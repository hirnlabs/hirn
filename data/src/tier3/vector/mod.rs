//! Embedding-backed vector tier: `fastembed-rs` on `sqlite-vec`.
//!
//! Models are cached in `~/.hirn/models/`, chunks are keyed by their `blake3`
//! content hash, and unchanged chunks keep their existing embedding
//! (HIR-76 resolution).

use std::path::PathBuf;
use std::sync::Mutex;

use blake3::Hasher;
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

use crate::error::{Error, Result};
use crate::tier3::{IndexDb, sqlite};
use crate::types::{BlockId, DocPath};
use crate::vector::{EMBEDDING_DIM, VectorChunk, VectorHit, VectorIndex};

/// The V1 embedding model, 384 dimensions.
pub const MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

/// The `sqlite-vec` implementation of [`VectorIndex`].
///
/// The connection sits behind a `Mutex` because `rusqlite::Connection` is not
/// `Sync`; the embedder needs `&mut self` for the same reason. Both are already
/// serialised, so the lock costs nothing real.
pub struct SqliteVectorIndex {
    db: Mutex<IndexDb>,
    embedder: Mutex<TextEmbedding>,
}

impl SqliteVectorIndex {
    /// Open the index over an existing [`IndexDb`] and load the embedding model
    /// from `~/.hirn/models/`, downloading it on first use.
    ///
    /// `IndexDb` clones share one SQLite connection, so a backend passes
    /// `backend.index().clone()` rather than opening a second handle.
    pub fn open(db: IndexDb) -> Result<Self> {
        Self::with_model_cache(db, crate::types::ProjectLayout::models_dir())
    }

    /// Open with an explicit model cache directory, for tests and offline use.
    pub fn with_model_cache(db: IndexDb, cache_dir: PathBuf) -> Result<Self> {
        let mut options = TextInitOptions::default();
        options.model_name = MODEL;
        options.cache_dir = cache_dir;
        options.show_download_progress = false;
        let embedder =
            TextEmbedding::try_new(options).map_err(|e| Error::Embedding(e.to_string()))?;
        Ok(Self {
            db: Mutex::new(db),
            embedder: Mutex::new(embedder),
        })
    }

    /// Embed one text into a 384-dimensional vector.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut embedder = self
            .embedder
            .lock()
            .map_err(|_| Error::Embedding("embedder lock poisoned".into()))?;
        let mut embeddings = embedder
            .embed([text], Some(1))
            .map_err(|e| Error::Embedding(e.to_string()))?;
        embeddings
            .pop()
            .ok_or_else(|| Error::Embedding("model returned no embedding".into()))
    }

    fn lock_db(&self) -> Result<std::sync::MutexGuard<'_, IndexDb>> {
        self.db
            .lock()
            .map_err(|_| Error::Embedding("index lock poisoned".into()))
    }
}

/// `blake3` of a chunk's text, hex-encoded.
pub fn chunk_hash(text: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(text.as_bytes());
    hasher.finalize().to_hex().to_string()
}

impl VectorIndex for SqliteVectorIndex {
    fn upsert_document(&self, doc: &DocPath, chunks: &[VectorChunk]) -> Result<()> {
        let known = {
            let db = self.lock_db()?;
            sqlite::read_chunk_hashes(&db, doc)?
        };
        let mut embedder = self
            .embedder
            .lock()
            .map_err(|_| Error::Embedding("embedder lock poisoned".into()))?;

        for chunk in chunks {
            let hash = chunk_hash(&chunk.text);
            if known.get(chunk.block_id.as_str()) == Some(&hash) {
                continue;
            }
            let mut vectors = embedder
                .embed([chunk.text.as_str()], Some(1))
                .map_err(|e| Error::Embedding(e.to_string()))?;
            let vector = vectors
                .pop()
                .ok_or_else(|| Error::Embedding("model returned no embedding".into()))?;
            if vector.len() != EMBEDDING_DIM {
                return Err(Error::Embedding(format!(
                    "expected {EMBEDDING_DIM} dimensions, got {}",
                    vector.len()
                )));
            }
            let mut db = self.lock_db()?;
            sqlite::upsert_vector(&mut db, doc, chunk.block_id.as_str(), &vector)?;
            sqlite::write_chunk_hash(&mut db, doc, chunk.block_id.as_str(), &hash)?;
        }

        // Blocks removed since the last derivation must not keep a stale
        // embedding or a hash that would make a later re-add skip embedding.
        let live: std::collections::HashSet<&str> =
            chunks.iter().map(|chunk| chunk.block_id.as_str()).collect();
        let mut db = self.lock_db()?;
        for stale in known.keys().filter(|id| !live.contains(id.as_str())) {
            sqlite::delete_chunk(&mut db, doc, stale)?;
        }
        Ok(())
    }

    fn delete_document(&self, doc: &DocPath) -> Result<()> {
        let mut db = self.lock_db()?;
        sqlite::delete_vectors(&mut db, doc)
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<VectorHit>> {
        let db = self.lock_db()?;
        let rows = sqlite::search_vectors(&db, query, k)?;
        Ok(rows
            .into_iter()
            .map(|(doc, block, distance)| VectorHit {
                handle: crate::types::NodeHandle::block(doc, BlockId::from(block)),
                distance,
            })
            .collect())
    }

    fn len(&self) -> Result<usize> {
        let db = self.lock_db()?;
        db.vector_count()
    }
}
