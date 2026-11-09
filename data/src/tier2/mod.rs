//! Tier 2 — the Loro CRDT engine.
//!
//! One Loro document per Markdown file (HIR-72 resolution), persisted at
//! `<project>/.hirn/docs/<rel-path>.loro`. The document schema is
//! `frontmatter` (`LoroMap`) plus `blocks` (`LoroMovableList` of
//! `{ id, kind, text, meta }`), with `text` a `LoroText` per block (HIR-77
//! resolution).

pub mod doc;
pub mod store;
pub mod streaming;
pub mod watch;
pub mod writeback;

pub use doc::{LoroDocument, doc_from_snapshot, snapshot_from_doc, snapshot_from_json};
pub use store::LoroStore;
pub use watch::{EchoGuard, FileEvent, FileEventKind, Watcher, debounce_interval};
pub use writeback::{atomic_write, flush_interval};
