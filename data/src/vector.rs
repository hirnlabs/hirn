//! The vector index seam.
//!
//! V1 has one implementation (`tier3::vector::SqliteVectorIndex`, backed by
//! `sqlite-vec`). The trait exists so the retrieval call sites in the RPC layer
//! never import the vector tier directly, and so a second engine can be added
//! without touching them.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::{BlockId, DocPath, NodeHandle};

/// Embedding width for V1 (`all-MiniLM-L6-v2`, `bge-small-en-v1.5`).
pub const EMBEDDING_DIM: usize = 384;

/// A chunk to embed: one block of one document.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorChunk {
    pub doc_path: DocPath,
    pub block_id: BlockId,
    pub text: String,
}

/// One nearest-neighbour hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorHit {
    pub handle: NodeHandle,
    /// Cosine distance; lower is closer.
    pub distance: f32,
}

/// Store and search block embeddings.
pub trait VectorIndex: Send + Sync {
    /// Replace the chunks of one document. Chunks whose `blake3` content hash
    /// is unchanged keep their embedding.
    fn upsert_document(&self, doc_path: &DocPath, chunks: &[VectorChunk]) -> Result<()>;

    /// Drop every chunk of one document.
    fn delete_document(&self, doc_path: &DocPath) -> Result<()>;

    /// The `k` nearest chunks to `query`.
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<VectorHit>>;

    /// Number of stored chunks.
    fn len(&self) -> Result<usize>;

    /// Whether the index holds no chunks.
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}
