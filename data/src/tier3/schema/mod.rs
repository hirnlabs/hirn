//! Tier 3's view of the shared type-folder schema.
//!
//! Discovery and the raw `schema.toml` types live in [`crate::tier1::schema`];
//! this module only re-exports them for the SQLite derivation and adds the
//! `IndexDb`-aware refresh that regenerates the `type_<name>` views.

use std::path::Path;

use crate::error::Result;
use crate::tier3::IndexDb;
use crate::types::TypeName;

pub use crate::tier1::schema::{
    RawProperty, RawTypeSchema, TypeFolder, discover, load_project_schema, schema_config,
};

/// Rescan the project, regenerate every `type_<name>` view, and return the
/// type names now in force.
pub fn refresh(db: &mut IndexDb, project_root: &Path) -> Result<Vec<TypeName>> {
    let folders = discover(project_root)?;
    let names: Vec<TypeName> = folders.iter().map(|f| f.name.clone()).collect();
    db.regenerate_views(&folders)?;
    Ok(names)
}
