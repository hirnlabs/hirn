use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One skill entry in `tools.json`: GitHub origin plus activation flag.
///
/// Skills share the index file with tools (`tools.json` is the generated
/// `tools/list`-shaped index); `active` is user-owned and survives re-sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    /// `owner/repo` the skill was fetched from.
    pub source: String,
    /// Always `github` in v1.
    #[serde(rename = "sourceType")]
    pub source_type: String,
    /// Repo-relative path to the skill's `SKILL.md`.
    #[serde(rename = "skillPath")]
    pub skill_path: String,
    /// `false` hides the skill from `list` and refuses `skill`.
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

/// The on-disk index: `{ "version": 1, "skills": { name: entry } }`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillsIndex {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub skills: BTreeMap<String, SkillEntry>,
}

fn default_version() -> u32 {
    1
}

/// What went wrong while reading or writing `tools.json`.
#[derive(Debug)]
pub enum IndexError {
    /// Filesystem failure with the path attached.
    Io { path: PathBuf, source: std::io::Error },
    /// Existing `tools.json` was not valid JSON.
    Corrupt { path: PathBuf, source: serde_json::Error },
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            IndexError::Corrupt { path, source } => {
                write!(f, "{} is not valid JSON: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for IndexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IndexError::Io { source, .. } => Some(source),
            IndexError::Corrupt { source, .. } => Some(source),
        }
    }
}

/// Load `tools.json`, tolerating a missing file as an empty index.
pub fn load(index_path: &Path) -> Result<SkillsIndex, IndexError> {
    if !index_path.exists() {
        return Ok(SkillsIndex::default());
    }
    let raw = fs::read_to_string(index_path).map_err(|source| IndexError::Io {
        path: index_path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|source| IndexError::Corrupt {
        path: index_path.to_path_buf(),
        source,
    })
}

/// Persist the index (creating parent dirs), preserving unrelated keys.
///
/// Unknown top-level keys (e.g. a future `tools` section) round-trip
/// untouched; only `version` and `skills` are rewritten.
pub fn save(index_path: &Path, index: &SkillsIndex) -> Result<(), IndexError> {
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).map_err(|source| IndexError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut root: serde_json::Value = if index_path.exists() {
        let raw = fs::read_to_string(index_path).map_err(|source| IndexError::Io {
            path: index_path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&raw).map_err(|source| IndexError::Corrupt {
            path: index_path.to_path_buf(),
            source,
        })?
    } else {
        serde_json::json!({})
    };
    if !root.is_object() {
        root = serde_json::json!({});
    }
    root["version"] = serde_json::json!(index.version);
    root["skills"] = serde_json::to_value(&index.skills).expect("skills serialize");
    let pretty = serde_json::to_string_pretty(&root).expect("index serialize");
    fs::write(index_path, format!("{pretty}\n")).map_err(|source| IndexError::Io {
        path: index_path.to_path_buf(),
        source,
    })
}

/// Insert or replace an entry; new skills arrive active by default.
pub fn upsert(index: &mut SkillsIndex, name: String, mut entry: SkillEntry) -> bool {
    let is_new = !index.skills.contains_key(&name);
    if is_new {
        entry.active = true;
    } else if let Some(existing) = index.skills.get(&name) {
        // Re-add preserves the user's activation choice.
        entry.active = existing.active;
    }
    index.skills.insert(name, entry);
    is_new
}
