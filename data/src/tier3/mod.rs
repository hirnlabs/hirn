//! Tier 3 — one derived SQLite database at `<project>/.hirn/index.db`.

pub mod graph;
pub mod schema;
pub mod sqlite;
pub mod vector;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{Error, Result};
use crate::types::{Block, DocPath, NodeHandle, PropertyValue, TypeName};

/// The single connection every Tier 3 index shares.
///
/// `sqlite-vec`, FTS5 and the typed views live in the same database (HIR-76
/// resolution), so there is exactly one handle: never open a second one.
/// `IndexDb` is cheap to clone and every clone shares the same connection,
/// which is what lets the vector tier and the backend write through one
/// handle.
#[derive(Clone)]
pub struct IndexDb {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl IndexDb {
    /// Open or create `<project>/.hirn/index.db` and install the base tables.
    pub fn open(project_root: impl AsRef<Path>) -> Result<Self> {
        sqlite::open(project_root)
    }

    /// Lock the shared connection. Every entry point that touches SQL goes
    /// through here, so clones serialise on one handle rather than opening a
    /// second connection to the same file.
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| Error::Sqlite("index connection lock poisoned".into()))
    }

    /// Open an in-memory database, for tests.
    pub fn open_in_memory() -> Result<Self> {
        sqlite::open_in_memory()
    }

    /// Load `schema.toml` files for every type folder below `project_root` and
    /// regenerate the `type_<name>` views.
    pub fn refresh_schema(&mut self, project_root: impl AsRef<Path>) -> Result<Vec<TypeName>> {
        schema::refresh(self, project_root.as_ref())
    }

    /// Create or replace the `type_<name>` views from already-loaded folders.
    pub fn regenerate_views(&mut self, folders: &[schema::TypeFolder]) -> Result<()> {
        sqlite::regenerate_views(self, folders)
    }

    /// Upsert a document's `documents` row and its `properties` rows.
    pub fn upsert_document(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        frontmatter: &[(String, PropertyValue)],
    ) -> Result<()> {
        sqlite::write_frontmatter_tx(self, doc, type_name, frontmatter)
    }

    /// Write only the frontmatter half of a document: its `documents` row and
    /// its `properties` rows, in one transaction. Leaves `blocks` and
    /// `blocks_fts` untouched, which is what a frontmatter-only edit needs.
    pub fn update_frontmatter(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        frontmatter: &[(String, PropertyValue)],
    ) -> Result<()> {
        sqlite::write_frontmatter_tx(self, doc, type_name, frontmatter)
    }

    /// Write a document's frontmatter rows and block rows in one transaction.
    pub fn derive_document(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        snapshot: &crate::projection::DocumentSnapshot,
    ) -> Result<()> {
        sqlite::derive_document(self, doc, type_name, snapshot)
    }

    /// Replace a document's rows in `blocks` and `blocks_fts`.
    pub fn replace_blocks(&mut self, doc: &DocPath, blocks: &[Block]) -> Result<()> {
        sqlite::replace_blocks(self, doc, blocks)
    }

    /// Remove a document from every derived table.
    pub fn delete_document(&mut self, doc: &DocPath) -> Result<()> {
        sqlite::delete_document(self, doc)
    }

    /// Remove every derived row, for a full rebuild.
    pub fn clear(&mut self) -> Result<()> {
        sqlite::clear(self)
    }

    /// Number of stored chunk embeddings.
    pub fn vector_count(&self) -> Result<usize> {
        sqlite::count_vectors(self)
    }

    /// Run a read-only `SELECT` and return the rows as JSON objects.
    pub fn query_sql(&self, sql: &str) -> Result<Vec<serde_json::Value>> {
        sqlite::query_sql(self, sql)
    }

    /// BM25 full-text search over block text.
    pub fn search_full_text(&self, query: &str, limit: usize) -> Result<Vec<FullTextHit>> {
        sqlite::search_full_text(self, query, limit)
    }
}

/// One FTS5 hit.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FullTextHit {
    pub handle: NodeHandle,
    pub text: String,
    /// BM25 score; more negative is a better match in SQLite's `bm25()`.
    pub score: f64,
}
