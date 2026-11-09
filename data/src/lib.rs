//! `hirn-data` — the Hirn data backend.
//!
//! Three tiers behind one API:
//!
//! - **Tier 1** (`tier1`): canonical Markdown files, parsed and rendered through
//!   `hyalo-core`, with `schema.toml` type folders and Myers block diffing.
//! - **Tier 2** (`tier2`): a Loro CRDT per document, the file watcher, atomic
//!   write-back and snapshot compaction.
//! - **Tier 3** (`tier3`): fully derived, recomputable indices — SQLite
//!   (`index.db`) for typed views, FTS5 and `sqlite-vec`, Grafeo
//!   (`graph.grafeo`) for links.
//!
//! `rpc` exposes the whole thing as JSON-RPC 2.0 over any transport.
//!
//! The trait seams live at the crate root: [`MarkdownProjection`] between
//! Tier 1 and Tier 2, [`VectorIndex`] for retrieval. Only `backend` imports all
//! three tiers. Tier 1 owns the shared schema-folder and link types; Tier 3
//! consumes them, because a type folder is both a Markdown convention and a SQL
//! view, and a wikilink is both a body construct and a graph edge.

pub mod backend;
pub mod error;
pub mod projection;
pub mod rpc;
pub mod tier1;
pub mod tier2;
pub mod tier3;
pub mod types;
pub mod vector;

pub use backend::{DataBackend, LoadedDocument, projection_for};

pub use error::{Error, Result};
pub use projection::{DocumentSnapshot, MarkdownProjection};
pub use tier2::{FileEvent, FileEventKind, Watcher};
pub use types::{
    Block, BlockId, BlockKind, DocPath, NodeHandle, ProjectLayout, PropertyType, PropertyValue,
    SubscriptionId, TypeName,
};
pub use vector::{EMBEDDING_DIM, VectorChunk, VectorHit, VectorIndex};
