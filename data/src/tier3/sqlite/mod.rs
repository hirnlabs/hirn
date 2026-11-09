//! The SQLite base tables, typed views, FTS5 index and SQL access.

use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use rusqlite::{Connection, OpenFlags, params};

use crate::error::{Error, Result};
use crate::tier3::schema::TypeFolder;
use crate::tier3::{FullTextHit, IndexDb};
use crate::types::{Block, DocPath, NodeHandle, PropertyValue, TypeName};

/// Install the extension once per process.
///
/// `sqlite-vec` registers as an auto-extension so every connection from this
/// process gets `vec0` without a second handle or an explicit load.
fn register_vec_extension() {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *mut std::os::raw::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> std::os::raw::c_int,
        >(
            sqlite_vec::sqlite3_vec_init as *const ()
        )));
    });
}

/// Open `<project>/.hirn/index.db`, creating `.hirn/` and the schema if absent.
pub fn open(project_root: impl AsRef<Path>) -> Result<IndexDb> {
    register_vec_extension();
    let root = project_root.as_ref();
    let hirn_dir = root.join(".hirn");
    if !hirn_dir.exists() {
        std::fs::create_dir_all(&hirn_dir).map_err(|e| Error::io(&hirn_dir, e))?;
    }
    let conn = Connection::open_with_flags(
        hirn_dir.join("index.db"),
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    init(&conn)?;
    Ok(IndexDb {
        conn: Arc::new(Mutex::new(conn)),
    })
}

/// Open a throwaway in-memory index.
pub fn open_in_memory() -> Result<IndexDb> {
    register_vec_extension();
    let conn = Connection::open_in_memory()?;
    init(&conn)?;
    Ok(IndexDb {
        conn: Arc::new(Mutex::new(conn)),
    })
}

fn init(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")
        .or_else(|_| Ok::<(), rusqlite::Error>(()))
        .ok();
    // Tier 3 is derived and recomputable: a torn commit is fixed by
    // `rebuild_index`, so full durability buys nothing but fsyncs per write.
    conn.pragma_update(None, "synchronous", "NORMAL")
        .or_else(|_| Ok::<(), rusqlite::Error>(()))
        .ok();
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS documents (
            doc_path   TEXT PRIMARY KEY,
            type_name  TEXT,
            title      TEXT,
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE TABLE IF NOT EXISTS properties (
            doc_path TEXT NOT NULL,
            key      TEXT NOT NULL,
            value    TEXT,
            type     TEXT NOT NULL,
            PRIMARY KEY (doc_path, key)
        ) WITHOUT ROWID;

        CREATE INDEX IF NOT EXISTS properties_key_value ON properties (key, value);

        CREATE TABLE IF NOT EXISTS blocks (
            doc_path TEXT NOT NULL,
            block_id TEXT NOT NULL,
            kind     TEXT NOT NULL,
            position INTEGER NOT NULL,
            text     TEXT NOT NULL,
            PRIMARY KEY (doc_path, block_id)
        ) WITHOUT ROWID;

        CREATE VIRTUAL TABLE IF NOT EXISTS blocks_fts USING fts5(
            doc_path UNINDEXED,
            block_id UNINDEXED,
            text,
            tokenize = 'unicode61'
        );

        -- `blocks_fts.doc_path` is UNINDEXED, so deleting by it scans the
        -- whole FTS table. This normal table maps a document's blocks to their
        -- FTS rowids and gives that lookup a real index.
        CREATE TABLE IF NOT EXISTS blocks_fts_map (
            doc_path TEXT NOT NULL,
            block_id TEXT NOT NULL,
            rowid    INTEGER NOT NULL,
            PRIMARY KEY (doc_path, block_id)
        ) WITHOUT ROWID;

        CREATE TABLE IF NOT EXISTS chunk_hashes (
            doc_path TEXT NOT NULL,
            block_id TEXT NOT NULL,
            hash     TEXT NOT NULL,
            PRIMARY KEY (doc_path, block_id)
        ) WITHOUT ROWID;

        CREATE VIRTUAL TABLE IF NOT EXISTS vec_blocks USING vec0(
            doc_path TEXT partition key,
            block_id TEXT,
            embedding float[384] distance_metric=cosine
        );
        ",
    )?;
    Ok(())
}

fn esc(value: &str) -> String {
    value.replace('\'', "''")
}

/// Create or replace every `type_<name>` view and its per-property indexes.
///
/// SQLite cannot index a view, so each declared property gets a partial index
/// on the underlying `properties (key, value)` table. Views and indexes are
/// dropped and recreated on `schema.toml` change, so a renamed or removed
/// property never leaves a stale view or index behind.
pub fn regenerate_views(db: &mut IndexDb, folders: &[TypeFolder]) -> Result<()> {
    let mut conn = db.lock()?;
    let existing_views: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'view' AND name LIKE 'type\\_%' ESCAPE '\\'",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    let existing_indexes: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx\\_type\\_%' ESCAPE '\\'",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    let tx = conn.transaction()?;
    for index in &existing_indexes {
        tx.execute_batch(&format!("DROP INDEX IF EXISTS \"{index}\""))?;
    }
    for view in &existing_views {
        tx.execute_batch(&format!("DROP VIEW IF EXISTS \"{view}\""))?;
    }

    for folder in folders {
        let props = folder.property_names();
        let view = folder.name.view_name();

        // `value` is stored as TEXT; a `number` or `boolean` property gets an
        // expression column so the view answers typed queries directly.
        let selected: Vec<String> = std::iter::once("d.doc_path AS doc_path".to_string())
            .chain(props.iter().enumerate().map(|(idx, prop)| {
                let column = format!("p{idx}.value");
                match folder.property_sql_type(prop) {
                    "REAL" => format!("CAST({column} AS REAL) AS \"{prop}\""),
                    "INTEGER" => format!("CAST({column} AS INTEGER) AS \"{prop}\""),
                    _ => format!("{column} AS \"{prop}\""),
                }
            }))
            .collect();
        let mut sql = format!("SELECT {}\nFROM documents d", selected.join(",\n       "));
        for (idx, prop) in props.iter().enumerate() {
            sql.push_str(&format!(
                "\n  LEFT JOIN properties p{idx} ON p{idx}.doc_path = d.doc_path AND p{idx}.key = '{}'",
                esc(prop)
            ));
        }
        sql.push_str(&format!(
            "\nWHERE d.type_name = '{}'",
            esc(folder.name.as_str())
        ));

        tx.execute_batch(&format!("DROP VIEW IF EXISTS \"{view}\""))?;
        tx.execute_batch(&format!("CREATE VIEW \"{view}\" AS {sql}"))?;

        for prop in &props {
            let idx_name = format!("idx_{}_{}", view, column_suffix(prop));
            tx.execute_batch(&format!(
                "CREATE INDEX \"{idx_name}\" ON properties (key, value) WHERE key = '{}'",
                esc(prop)
            ))?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn column_suffix(prop: &str) -> String {
    prop.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Encode a vector the way `sqlite-vec` expects: little-endian `f32` bytes.
fn encode_vector(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// Decode a `sqlite-vec` blob back into floats.
fn decode_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

/// Insert one chunk embedding into `vec_blocks`, replacing any older row.
pub fn upsert_vector(
    db: &mut IndexDb,
    doc: &DocPath,
    block_id: &str,
    embedding: &[f32],
) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM vec_blocks WHERE doc_path = ?1 AND block_id = ?2",
        params![doc.as_str(), block_id],
    )?;
    tx.execute(
        "INSERT INTO vec_blocks (doc_path, block_id, embedding) VALUES (?1, ?2, ?3)",
        params![doc.as_str(), block_id, encode_vector(embedding)],
    )?;
    tx.commit()?;
    Ok(())
}

/// Every `(block_id, embedding)` pair stored for one document.
pub fn read_vectors(db: &IndexDb, doc: &DocPath) -> Result<Vec<(String, Vec<f32>)>> {
    let conn = db.lock()?;
    let mut stmt =
        conn.prepare("SELECT block_id, embedding FROM vec_blocks WHERE doc_path = ?1")?;
    let rows = stmt.query_map(params![doc.as_str()], |row| {
        let block_id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((block_id, decode_vector(&blob)))
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// Nearest neighbours to `query` across every document.
pub fn search_vectors(db: &IndexDb, query: &[f32], k: usize) -> Result<Vec<(String, String, f32)>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT doc_path, block_id, distance
         FROM vec_blocks
         WHERE embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let rows = stmt.query_map(params![encode_vector(query), k as i64], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}

/// Remember a chunk's content hash so unchanged text skips re-embedding.
pub fn write_chunk_hash(db: &mut IndexDb, doc: &DocPath, block_id: &str, hash: &str) -> Result<()> {
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO chunk_hashes (doc_path, block_id, hash) VALUES (?1, ?2, ?3)
         ON CONFLICT (doc_path, block_id) DO UPDATE SET hash = excluded.hash",
        params![doc.as_str(), block_id, hash],
    )?;
    Ok(())
}

/// Drop one block's embedding and content hash.
pub fn delete_chunk(db: &mut IndexDb, doc: &DocPath, block_id: &str) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM vec_blocks WHERE doc_path = ?1 AND block_id = ?2",
        params![doc.as_str(), block_id],
    )?;
    tx.execute(
        "DELETE FROM chunk_hashes WHERE doc_path = ?1 AND block_id = ?2",
        params![doc.as_str(), block_id],
    )?;
    tx.commit()?;
    Ok(())
}

/// Drop the embeddings and hashes of one document.
pub fn delete_vectors(db: &mut IndexDb, doc: &DocPath) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM vec_blocks WHERE doc_path = ?1",
        params![doc.as_str()],
    )?;
    tx.execute(
        "DELETE FROM chunk_hashes WHERE doc_path = ?1",
        params![doc.as_str()],
    )?;
    tx.commit()?;
    Ok(())
}

/// The number of stored chunks.
pub fn count_vectors(db: &IndexDb) -> Result<usize> {
    let conn = db.lock()?;
    let count: i64 = conn.query_row("SELECT count(*) FROM vec_blocks", [], |row| row.get(0))?;
    Ok(usize::try_from(count).unwrap_or(0))
}

/// Replace one document's `blocks` rows and its `blocks_fts` rows.
pub fn replace_blocks(db: &mut IndexDb, doc: &DocPath, blocks: &[Block]) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    write_blocks(&tx, doc, blocks)?;
    tx.commit()?;
    Ok(())
}

/// Write the `documents`, `properties`, `blocks` and `blocks_fts` rows of one
/// document in a single transaction.
///
/// This is the instant half of the derivation: frontmatter and block text land
/// together, so a query never sees one without the other.
pub fn derive_document(
    db: &mut IndexDb,
    doc: &DocPath,
    type_name: Option<&TypeName>,
    snapshot: &crate::projection::DocumentSnapshot,
) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    write_frontmatter(&tx, doc, type_name, &snapshot.frontmatter)?;
    write_blocks(&tx, doc, &snapshot.blocks)?;
    tx.commit()?;
    Ok(())
}

fn write_frontmatter(
    tx: &rusqlite::Transaction<'_>,
    doc: &DocPath,
    type_name: Option<&TypeName>,
    frontmatter: &[(String, PropertyValue)],
) -> Result<()> {
    let title = frontmatter
        .iter()
        .find(|(k, _)| k == "title")
        .and_then(|(_, v)| match v {
            PropertyValue::String(s) => Some(s.clone()),
            _ => None,
        });
    tx.execute(
        "INSERT INTO documents (doc_path, type_name, title, updated_at)
         VALUES (?1, ?2, ?3, unixepoch())
         ON CONFLICT (doc_path) DO UPDATE SET
             type_name = excluded.type_name,
             title = excluded.title,
             updated_at = excluded.updated_at",
        params![doc.as_str(), type_name.map(TypeName::as_str), title],
    )?;
    tx.execute(
        "DELETE FROM properties WHERE doc_path = ?1",
        params![doc.as_str()],
    )?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO properties (doc_path, key, value, type) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (key, value) in frontmatter {
            let kind = format!("{:?}", value.type_name()).to_lowercase();
            let stored = match value {
                PropertyValue::Null => None,
                PropertyValue::Bool(v) => Some(i64::from(*v).to_string()),
                PropertyValue::Number(v) => Some(v.to_string()),
                PropertyValue::String(v) => Some(v.clone()),
                other => Some(serde_json::to_string(&other.to_sql()).unwrap_or_default()),
            };
            stmt.execute(params![doc.as_str(), key, stored, kind])?;
        }
    }
    Ok(())
}

/// Write `documents` and `properties` for one document in its own transaction.
pub fn write_frontmatter_tx(
    db: &mut IndexDb,
    doc: &DocPath,
    type_name: Option<&TypeName>,
    frontmatter: &[(String, PropertyValue)],
) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    write_frontmatter(&tx, doc, type_name, frontmatter)?;
    tx.commit()?;
    Ok(())
}

fn write_blocks(tx: &rusqlite::Transaction<'_>, doc: &DocPath, blocks: &[Block]) -> Result<()> {
    // Look the old rowids up in the indexed map. FTS5 cannot index its
    // `UNINDEXED` `doc_path`, so querying `blocks_fts` directly would scan the
    // whole vault's FTS table on every write.
    let stale_rowids: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT rowid FROM blocks_fts_map WHERE doc_path = ?1")?;
        let rows = stmt.query_map(params![doc.as_str()], |row| row.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    for rowid in stale_rowids {
        tx.execute("DELETE FROM blocks_fts WHERE rowid = ?1", params![rowid])?;
    }
    tx.execute(
        "DELETE FROM blocks_fts_map WHERE doc_path = ?1",
        params![doc.as_str()],
    )?;
    tx.execute(
        "DELETE FROM blocks WHERE doc_path = ?1",
        params![doc.as_str()],
    )?;
    {
        let mut block_stmt = tx.prepare(
            "INSERT INTO blocks (doc_path, block_id, kind, position, text)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        let mut fts_stmt =
            tx.prepare("INSERT INTO blocks_fts (doc_path, block_id, text) VALUES (?1, ?2, ?3)")?;
        let mut map_stmt = tx.prepare(
            "INSERT INTO blocks_fts_map (doc_path, block_id, rowid) VALUES (?1, ?2, ?3)",
        )?;
        for (position, block) in blocks.iter().enumerate() {
            block_stmt.execute(params![
                doc.as_str(),
                block.id.as_str(),
                block.kind.as_str(),
                position as i64,
                block.text
            ])?;
            if !block.text.trim().is_empty() {
                fts_stmt.execute(params![doc.as_str(), block.id.as_str(), block.text])?;
                let rowid = tx.last_insert_rowid();
                map_stmt.execute(params![doc.as_str(), block.id.as_str(), rowid])?;
            }
        }
    }
    Ok(())
}

/// Remove a document from every derived table.
pub fn delete_document(db: &mut IndexDb, doc: &DocPath) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    let stale_rowids: Vec<i64> = {
        let mut stmt = tx.prepare("SELECT rowid FROM blocks_fts_map WHERE doc_path = ?1")?;
        let rows = stmt.query_map(params![doc.as_str()], |row| row.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };
    for rowid in stale_rowids {
        tx.execute("DELETE FROM blocks_fts WHERE rowid = ?1", params![rowid])?;
    }
    for table in [
        "blocks_fts_map",
        "documents",
        "properties",
        "blocks",
        "vec_blocks",
        "chunk_hashes",
    ] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE doc_path = ?1"),
            params![doc.as_str()],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Remove every derived row, for a full rebuild.
pub fn clear(db: &mut IndexDb) -> Result<()> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    tx.execute_batch(
        "DELETE FROM blocks_fts;
         DELETE FROM blocks_fts_map;
         DELETE FROM blocks;
         DELETE FROM properties;
         DELETE FROM documents;
         DELETE FROM vec_blocks;
         DELETE FROM chunk_hashes;",
    )?;
    tx.commit()?;
    Ok(())
}

/// Run a `SELECT` and return rows as JSON objects.
///
/// Tier 3 is read-only: anything that modifies data or schema is refused.
/// Uses `Statement::readonly()` from SQLite to strictly reject mutations
/// (including those hidden in CTEs like `WITH x AS (SELECT 1) DELETE ...`).
pub fn query_sql(db: &IndexDb, sql: &str) -> Result<Vec<serde_json::Value>> {
    let trimmed = sql.trim();
    if trimmed.contains(';') {
        return Err(Error::Invalid(
            "Tier 3 SQL is read-only: multiple statements or trailing semicolons are not permitted"
                .into(),
        ));
    }
    let conn = db.lock()?;
    let mut stmt = conn.prepare(trimmed)?;
    if !stmt.readonly() {
        return Err(Error::Invalid(
            "Tier 3 SQL is read-only: statement modifies data or schema".into(),
        ));
    }
    let names: Vec<String> = stmt.column_names().map(str::to_string).collect();
    let mut rows = stmt.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let mut object = serde_json::Map::new();
        for (idx, name) in names.iter().enumerate() {
            let value = match row.get_ref(idx)? {
                rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                rusqlite::types::ValueRef::Integer(v) => serde_json::Value::from(v),
                rusqlite::types::ValueRef::Real(v) => serde_json::Value::from(v),
                rusqlite::types::ValueRef::Text(v) => {
                    serde_json::Value::from(String::from_utf8_lossy(v).to_string())
                }
                rusqlite::types::ValueRef::Blob(v) => serde_json::Value::from(
                    v.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                ),
            };
            object.insert(name.clone(), value);
        }
        out.push(serde_json::Value::Object(object));
    }
    Ok(out)
}

/// BM25 full-text search over `blocks_fts`.
pub fn search_full_text(db: &IndexDb, query: &str, limit: usize) -> Result<Vec<FullTextHit>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT doc_path, block_id, text, bm25(blocks_fts) AS score
         FROM blocks_fts
         WHERE blocks_fts MATCH ?1
         ORDER BY score
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![query, limit as i64], |row| {
        Ok(FullTextHit {
            handle: NodeHandle::block(
                DocPath::from(row.get::<_, String>(0)?),
                crate::types::BlockId::from(row.get::<_, String>(1)?),
            ),
            text: row.get(2)?,
            score: row.get(3)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Error::from)
}

/// The hash-tagged chunk table used for `blake3` caching.
pub fn read_chunk_hashes(
    db: &IndexDb,
    doc: &DocPath,
) -> Result<std::collections::HashMap<String, String>> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare("SELECT block_id, hash FROM chunk_hashes WHERE doc_path = ?1")?;
    let rows = stmt.query_map(params![doc.as_str()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    Ok(rows.collect::<std::result::Result<_, _>>()?)
}
