//! Shared types for the Hirn data backend.
//!
//! Every tier and the RPC surface use these types. Nothing here may depend on
//! a tier: the tiers depend on this module, never the reverse.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A document's path, relative to the project root, using forward slashes.
///
/// The document identity is the file path (HIR-77 resolution). The type keeps
/// that contract in one place so no tier re-invents normalisation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DocPath(String);

impl DocPath {
    /// Normalise an arbitrary path to a project-relative, forward-slash path.
    ///
    /// Backslashes become forward slashes, `.` components are dropped, and a
    /// leading root or drive prefix is stripped. `..` pops the preceding
    /// component and is dropped when there is nothing to pop, so a path can
    /// never escape the project root.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let raw = path.as_ref().to_string_lossy().replace('\\', "/");
        let mut parts: Vec<&str> = Vec::new();
        for part in raw.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                _ if parts.is_empty() && is_windows_drive(part) => {}
                _ => parts.push(part),
            }
        }
        Self(parts.join("/"))
    }

    /// The path as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path components that are safe to join below a project root.
    fn relative_components(&self) -> impl Iterator<Item = &str> {
        self.0
            .split('/')
            .filter(|part| !part.is_empty() && *part != "." && *part != "..")
    }

    /// Resolve against a project root.
    ///
    /// The join is component-wise and drops any remaining `.`/`..`, so even a
    /// `DocPath` that arrived through a hand-written serde payload cannot walk
    /// outside `root`.
    pub fn to_fs_path(&self, root: &Path) -> PathBuf {
        let mut path = root.to_path_buf();
        for part in self.relative_components() {
            path.push(part);
        }
        path
    }

    /// File stem without directories, e.g. `note` for `folder/note.md`.
    pub fn stem(&self) -> &str {
        let file = self.0.rsplit('/').next().unwrap_or(&self.0);
        file.strip_suffix(".md").unwrap_or(file)
    }

    /// Directory part, empty when the document sits at the project root.
    pub fn parent_dir(&self) -> &str {
        match self.0.rfind('/') {
            Some(idx) => &self.0[..idx],
            None => "",
        }
    }
}

fn is_windows_drive(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

impl<'de> Deserialize<'de> for DocPath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Validate through `new`; deriving `Deserialize` would let a payload
        // write an absolute or `..` path straight into the inner string.
        let raw = String::deserialize(deserializer)?;
        Ok(Self::new(raw))
    }
}

impl fmt::Display for DocPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for DocPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for DocPath {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// A block's stable identity: a 10-character `nanoid` assigned on insertion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockId(String);

impl BlockId {
    /// Wrap an existing id, as read from a Loro block map or a `<!-- ^id -->`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Mint a fresh 10-character id.
    pub fn generate() -> Self {
        Self(nanoid::nanoid!(10))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for BlockId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for BlockId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A type name, derived from a type folder (its directory name).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypeName(String);

impl TypeName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The derived SQLite view name: `type_<name>`.
    pub fn view_name(&self) -> String {
        format!("type_{}", self.0)
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The kind of a block, mirroring the Markdown block types in the Loro schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Paragraph,
    Heading,
    Task,
    Code,
    Quote,
    ListItem,
    ThematicBreak,
    Table,
    Other(String),
}

impl BlockKind {
    /// Stable string used in the Loro `kind` field and in SQLite.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::Task => "task",
            Self::Code => "code",
            Self::Quote => "quote",
            Self::ListItem => "list_item",
            Self::ThematicBreak => "thematic_break",
            Self::Table => "table",
            Self::Other(raw) => raw,
        }
    }

    /// Parse the `kind` field, mapping anything unrecognised to `Other`.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "paragraph" => Self::Paragraph,
            "heading" => Self::Heading,
            "task" => Self::Task,
            "code" => Self::Code,
            "quote" => Self::Quote,
            "list_item" => Self::ListItem,
            "thematic_break" => Self::ThematicBreak,
            "table" => Self::Table,
            other => Self::Other(other.to_string()),
        }
    }
}

/// A block of a document as it flows through the tiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub kind: BlockKind,
    pub text: String,
    /// Block-level metadata: heading level, task state, list depth, code fence
    /// info string. Keys are free-form; known keys are listed in
    /// [`Block::heading_level`] and friends.
    pub meta: serde_json::Map<String, serde_json::Value>,
}

impl Block {
    /// Key holding the block's `LoroText`, per the HIR-77 schema.
    pub const TEXT_KEY: &'static str = "text";

    /// Create a block with empty metadata.
    pub fn new(id: BlockId, kind: BlockKind, text: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            text: text.into(),
            meta: serde_json::Map::new(),
        }
    }

    /// Heading level, when the block is a heading.
    pub fn heading_level(&self) -> Option<u8> {
        self.meta
            .get("level")
            .and_then(serde_json::Value::as_u64)
            .and_then(|level| u8::try_from(level).ok())
    }

    /// Task checkbox state, when the block is a task.
    pub fn task_checked(&self) -> Option<bool> {
        self.meta
            .get("checked")
            .and_then(serde_json::Value::as_bool)
    }

    /// The `<!-- ^id -->` comment that pins this block's id in Markdown.
    pub fn anchor(&self) -> String {
        format!("<!-- ^{} -->", self.id)
    }
}

/// A property value as stored in frontmatter.
///
/// Mirrors the scalar shapes `hyalo-core` infers, plus lists and maps, so the
/// tiers never carry raw YAML around.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<PropertyValue>),
    Map(serde_json::Map<String, serde_json::Value>),
}

impl PropertyValue {
    /// The declared SQLite/SQL type name for this value.
    pub fn type_name(&self) -> PropertyType {
        match self {
            Self::Null => PropertyType::Null,
            Self::Bool(_) => PropertyType::Boolean,
            Self::Number(_) => PropertyType::Number,
            Self::String(_) => PropertyType::String,
            Self::List(_) => PropertyType::List,
            Self::Map(_) => PropertyType::Map,
        }
    }

    /// The value as SQLite stores it: scalars as JSON, containers serialised.
    pub fn to_sql(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(v) => serde_json::Value::Bool(*v),
            Self::Number(v) => serde_json::Number::from_f64(*v)
                .map_or(serde_json::Value::Null, serde_json::Value::Number),
            Self::String(v) => serde_json::Value::String(v.clone()),
            Self::List(_) | Self::Map(_) => {
                serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
            }
        }
    }

    /// Rebuild from the JSON `hyalo-core` yields.
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(v) => Self::Bool(*v),
            serde_json::Value::Number(v) => Self::Number(v.as_f64().unwrap_or_default()),
            serde_json::Value::String(v) => Self::String(v.clone()),
            serde_json::Value::Array(items) => {
                Self::List(items.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(map) => Self::Map(map.clone()),
        }
    }
}

/// The type of a frontmatter property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Null,
    Boolean,
    Number,
    String,
    List,
    Map,
}

impl PropertyType {
    /// The SQLite storage class a property of this type maps to.
    pub fn sql_type(&self) -> &'static str {
        match self {
            Self::Null => "NULL",
            Self::Boolean => "INTEGER",
            Self::Number => "REAL",
            Self::String => "TEXT",
            Self::List | Self::Map => "TEXT",
        }
    }
}

/// The stable 3-tuple referencing a document, a block, or a frontmatter
/// property across tiers and queries (HIR-77 resolution).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeHandle {
    pub doc_path: DocPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<BlockId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_key: Option<String>,
}

impl NodeHandle {
    /// Handle for a whole document.
    pub fn document(doc_path: impl Into<DocPath>) -> Self {
        Self {
            doc_path: doc_path.into(),
            block_id: None,
            property_key: None,
        }
    }

    /// Handle for a block.
    pub fn block(doc_path: impl Into<DocPath>, block_id: impl Into<BlockId>) -> Self {
        Self {
            doc_path: doc_path.into(),
            block_id: Some(block_id.into()),
            property_key: None,
        }
    }

    /// Handle for a frontmatter property.
    pub fn property(doc_path: impl Into<DocPath>, key: impl Into<String>) -> Self {
        Self {
            doc_path: doc_path.into(),
            block_id: None,
            property_key: Some(key.into()),
        }
    }
}

/// Subscription identifier handed out by the RPC layer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The on-disk locations inside `<project>/.hirn/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLayout {
    root: PathBuf,
}

impl ProjectLayout {
    /// Wrap a project root. No I/O happens here.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The canonical project root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `<project>/.hirn/`.
    pub fn hirn_dir(&self) -> PathBuf {
        self.root.join(".hirn")
    }

    /// `<project>/.hirn/index.db`.
    pub fn index_db(&self) -> PathBuf {
        self.hirn_dir().join("index.db")
    }

    /// `<project>/.hirn/graph.grafeo`.
    pub fn graph_db(&self) -> PathBuf {
        self.hirn_dir().join("graph.grafeo")
    }

    /// `<project>/.hirn/docs/`.
    pub fn docs_dir(&self) -> PathBuf {
        self.hirn_dir().join("docs")
    }

    /// `<project>/.hirn/docs/<rel-path>.loro`.
    pub fn loro_path(&self, doc: &DocPath) -> PathBuf {
        let mut path = self.docs_dir();
        let mut components = doc.relative_components().peekable();
        while let Some(part) = components.next() {
            if components.peek().is_none() {
                path.push(format!("{part}.loro"));
            } else {
                path.push(part);
            }
        }
        path
    }

    /// `~/.hirn/models/`, the shared embedding model cache.
    pub fn models_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".hirn")
            .join("models")
    }
}
