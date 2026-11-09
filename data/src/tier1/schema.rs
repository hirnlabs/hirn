//! Type folder discovery and `schema.toml` loading.
//!
//! A type folder is a project directory holding a `schema.toml`. Its directory
//! name is the type name, and every declared property gets a generated
//! `type_<name>` SQLite view plus a partial index on `properties` (HIR-79).
//!
//! These types live in Tier 1 because a type folder is a Markdown convention
//! first; Tier 3 consumes them when it builds its views.

use std::path::{Path, PathBuf};

use hyalo_core::schema::{PropertyConstraint, SchemaBind, SchemaConfig, TypeSchema};
use indexmap::IndexMap;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::types::TypeName;

/// The raw `schema.toml` shape.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawTypeSchema {
    /// Property keys that must be present in every document of this type.
    #[serde(default)]
    pub required: Vec<String>,
    /// Filename template for new documents of this type.
    #[serde(default)]
    pub filename: Option<String>,
    /// Default values applied to new documents.
    #[serde(default)]
    pub defaults: std::collections::BTreeMap<String, String>,
    /// Per-property declarations.
    #[serde(default)]
    pub properties: std::collections::BTreeMap<String, RawProperty>,
}

/// One declared property, including its optional validation constraints.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawProperty {
    /// One of `string`, `number`, `boolean`, `date`, `datetime`, `datetime-tz`,
    /// `list`, `enum`, `object-list`.
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Allowed values for an `enum` property.
    #[serde(default)]
    pub values: Vec<String>,
    /// Regex a `string` value must match.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Inclusive minimum length of a `string` value.
    #[serde(default)]
    pub min_length: Option<usize>,
    /// Inclusive maximum length of a `string` value.
    #[serde(default)]
    pub max_length: Option<usize>,
    /// Inclusive minimum of a `number` value.
    #[serde(default)]
    pub minimum: Option<f64>,
    /// Inclusive maximum of a `number` value.
    #[serde(default)]
    pub maximum: Option<f64>,
    /// Keys every `object-list` item must carry.
    #[serde(default)]
    pub required_keys: Vec<String>,
    /// When set, the only keys an `object-list` item may carry.
    #[serde(default)]
    pub allowed_keys: Option<Vec<String>>,
    /// Per-key regexes applied to `object-list` item values.
    #[serde(default)]
    pub key_patterns: IndexMap<String, String>,
}

/// A loaded type folder.
#[derive(Debug, Clone)]
pub struct TypeFolder {
    pub name: TypeName,
    /// Directory, relative to the project root, forward-slashed.
    pub dir: String,
    pub schema: RawTypeSchema,
}

impl TypeFolder {
    /// Parse `<dir>/schema.toml`.
    pub fn load(project_root: &Path, dir: &str) -> Result<Self> {
        let path = project_root.join(dir).join("schema.toml");
        let raw = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        let schema: RawTypeSchema =
            toml::from_str(&raw).map_err(|e| Error::Schema(format!("{}: {e}", path.display())))?;
        let name = PathBuf::from(dir)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| Error::Schema(format!("type folder has no name: {dir}")))?;
        Ok(Self {
            name: TypeName::new(name),
            dir: dir.to_string(),
            schema,
        })
    }

    /// Declared property names, sorted.
    pub fn property_names(&self) -> Vec<String> {
        self.schema.properties.keys().cloned().collect()
    }

    /// The SQLite storage class for one declared property.
    pub fn property_sql_type(&self, property: &str) -> &'static str {
        match self
            .schema
            .properties
            .get(property)
            .and_then(|p| p.kind.as_deref())
        {
            Some("number") => "REAL",
            Some("boolean") => "INTEGER",
            _ => "TEXT",
        }
    }
}

/// Every type folder below `project_root`, ignoring `.hirn/` and `.git/`.
pub fn discover(project_root: &Path) -> Result<Vec<TypeFolder>> {
    let mut found = Vec::new();
    let mut stack = vec![project_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| Error::io(&dir, e))?;
        for entry in entries {
            let entry = entry.map_err(|e| Error::io(&dir, e))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            if path.join("schema.toml").is_file() {
                let rel = path
                    .strip_prefix(project_root)
                    .map_err(|e| Error::Schema(e.to_string()))?
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push(TypeFolder::load(project_root, &rel)?);
            }
            stack.push(path);
        }
    }
    found.sort_by(|a, b| a.dir.cmp(&b.dir));
    Ok(found)
}

/// Build the `hyalo-core` schema config from the discovered type folders.
///
/// Each type folder's directory is bound by glob, so a document without an
/// explicit `type:` still resolves to its folder's type.
pub fn schema_config(folders: &[TypeFolder]) -> SchemaConfig {
    let mut types = std::collections::HashMap::new();
    let mut bind = Vec::new();
    for folder in folders {
        let mut type_schema = TypeSchema {
            required: folder.schema.required.clone(),
            filename_template: folder.schema.filename.clone(),
            defaults: folder
                .schema
                .defaults
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            properties: std::collections::HashMap::new(),
            required_sections: Vec::new(),
        };
        for (key, prop) in &folder.schema.properties {
            if let Some(constraint) = constraint_for(prop) {
                type_schema.properties.insert(key.clone(), constraint);
            }
        }
        types.insert(folder.name.as_str().to_string(), type_schema);
        bind.push((
            format!("{}/**/*.md", folder.dir),
            folder.name.as_str().to_string(),
        ));
    }
    SchemaConfig {
        default: TypeSchema::default(),
        types,
        exempt: Default::default(),
        bind: SchemaBind::new(bind),
    }
}

fn constraint_for(prop: &RawProperty) -> Option<PropertyConstraint> {
    Some(match prop.kind.as_deref()? {
        "string" => PropertyConstraint::String {
            pattern: prop.pattern.clone(),
            min_length: prop.min_length,
            max_length: prop.max_length,
        },
        "number" => PropertyConstraint::Number {
            minimum: prop.minimum,
            maximum: prop.maximum,
        },
        "boolean" => PropertyConstraint::Boolean,
        "date" => PropertyConstraint::Date,
        "datetime" => PropertyConstraint::DateTime,
        "datetime-tz" => PropertyConstraint::DateTimeTz,
        "list" => PropertyConstraint::List,
        "enum" => PropertyConstraint::Enum {
            values: prop.values.clone(),
        },
        "object-list" => PropertyConstraint::ObjectList {
            required_keys: prop.required_keys.clone(),
            allowed_keys: prop.allowed_keys.clone(),
            key_patterns: prop.key_patterns.clone(),
        },
        _ => return None,
    })
}

/// Discover every type folder and build one merged `hyalo-core` schema.
pub fn load_project_schema(project_root: &Path) -> Result<SchemaConfig> {
    let folders = discover(project_root)?;
    Ok(schema_config(&folders))
}
