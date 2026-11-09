//! The whole backend behind one handle: tiers, sync cycle and rebuild.
//!
//! This is the only module that imports every tier. Tiers do not import each
//! other, with one documented exception: Tier 3 consumes the shared link and
//! schema-folder types owned by Tier 1. They meet otherwise here and through
//! the crate-root trait seams.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::projection::{DocumentSnapshot, MarkdownProjection};
use crate::tier1::HyaloProjection;
use crate::tier2::watch::{DEBOUNCE, EchoGuard, Watcher};
use crate::tier2::{FileEvent, FileEventKind, LoroDocument, LoroStore};
use crate::tier3::IndexDb;
use crate::tier3::graph::GraphIndex;
use crate::tier3::schema::TypeFolder;
use crate::types::{DocPath, NodeHandle, ProjectLayout, TypeName};

/// A document as the API hands it out.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedDocument {
    pub path: DocPath,
    pub type_name: Option<TypeName>,
    pub snapshot: DocumentSnapshot,
}

impl LoadedDocument {
    /// The handle for a whole document.
    pub fn handle(&self) -> NodeHandle {
        NodeHandle::document(self.path.clone())
    }

    /// A handle per block, in document order.
    pub fn handles(&self) -> Vec<NodeHandle> {
        self.snapshot
            .blocks
            .iter()
            .map(|block| NodeHandle::block(self.path.clone(), block.id.clone()))
            .collect()
    }
}

/// The hirn data backend for one project directory.
///
/// Owns the three tiers and the sync cycle between them: the Loro documents
/// that are open, the filesystem watcher that turns external edits into CRDT
/// operations, and the debounced write-back that turns CRDT edits into files.
pub struct DataBackend {
    root: PathBuf,
    projection: HyaloProjection,
    store: LoroStore,
    index: IndexDb,
    graph: GraphIndex,
    types: HashMap<String, TypeName>,
    open: HashMap<DocPath, LoroDocument>,
    watcher: Option<Watcher>,
    echo: EchoGuard,
    /// The file text last read or written, per open document: the base of the
    /// three-way merge and of the streaming write.
    projected: HashMap<DocPath, String>,
}

impl DataBackend {
    /// Open (or initialise) the backend for a project root.
    ///
    /// Creates `<project>/.hirn/`, its SQLite index and its Grafeo graph, then
    /// discovers type folders and regenerates the typed views.
    pub fn open(project_root: impl AsRef<Path>) -> Result<Self> {
        let root = project_root.as_ref().to_path_buf();
        let schema = crate::tier1::schema::load_project_schema(&root)?;
        let mut index = IndexDb::open(&root)?;
        let folders = crate::tier3::schema::discover(&root)?;
        index.regenerate_views(&folders)?;
        let graph = GraphIndex::open(&root)?;
        graph.ensure_indexes()?;
        Ok(Self {
            store: LoroStore::new(&root),
            projection: HyaloProjection::with_schema(schema),
            index,
            graph,
            types: folders
                .iter()
                .map(|TypeFolder { dir, name, .. }| (dir.clone(), name.clone()))
                .collect(),
            open: HashMap::new(),
            watcher: None,
            echo: EchoGuard::default(),
            projected: HashMap::new(),
            root,
        })
    }

    /// The project root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The projection in use.
    pub fn projection(&self) -> &HyaloProjection {
        &self.projection
    }

    /// The derived SQLite index.
    pub fn index(&self) -> &IndexDb {
        &self.index
    }

    /// The derived graph.
    pub fn graph(&self) -> &GraphIndex {
        &self.graph
    }

    /// The type name for a document, from its folder's `schema.toml`.
    pub fn type_for(&self, doc: &DocPath) -> Option<TypeName> {
        self.types.get(doc.parent_dir()).cloned()
    }

    /// The watcher's debounce, so a host can align its own timers with it.
    pub fn debounce(&self) -> Duration {
        DEBOUNCE
    }

    /// Start watching the project for external Markdown edits.
    ///
    /// The callback runs on the watcher thread for every debounced change to a
    /// `.md` file. It must not block. The change is *not* applied here: the
    /// host calls [`DataBackend::apply_external_edit`] with the file's new
    /// text, which keeps the one write path through the CRDT. Writes this
    /// backend performs are suppressed by the echo guard, so the write-back
    /// never re-enters as an external edit.
    pub fn start_watching(
        &mut self,
        on_change: impl Fn(FileEvent) + Send + Sync + 'static,
    ) -> Result<()> {
        let root = self.root.clone();
        let echo = self.echo.clone();
        let callback = std::sync::Arc::new(on_change);
        let event_root = root.clone();
        let watcher = Watcher::start(&root, move |event: FileEvent| {
            let path = event.path.to_fs_path(&event_root);
            if event.kind != FileEventKind::Removed
                && let Ok(contents) = std::fs::read_to_string(&path)
                && echo.consume_echo(&path, &contents)
            {
                return;
            }
            callback(event);
        })?;
        self.watcher = Some(watcher);
        Ok(())
    }

    /// Stop watching.
    pub fn stop_watching(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            watcher.stop();
        }
    }

    /// Load a document, reconciling the Markdown file with its Loro state.
    ///
    /// With no Loro state the file is parsed into a fresh document. With Loro
    /// state the file is folded in when it differs, so edits made while the
    /// document was closed are not lost. A document that exists in neither
    /// place is [`Error::NotFound`].
    ///
    /// A document that is already open is returned without touching Tier 3: the
    /// indices are kept current by [`DataBackend::flush`], which every mutation
    /// path calls, so re-deriving here would double the cost of each `set`.
    pub fn load(&mut self, doc: &DocPath) -> Result<LoadedDocument> {
        let was_open = self.open.contains_key(doc);
        if !was_open {
            let path = doc.to_fs_path(&self.root);
            let source = if path.exists() {
                Some(std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?)
            } else {
                None
            };
            let loaded = self.store.load(doc)?;
            let has_state = loaded.raw().len_ops() > 0;
            let document = match (has_state, source.as_deref()) {
                (false, Some(source)) => {
                    let snapshot = self.projection.parse(doc, source)?;
                    LoroDocument::from_snapshot(&snapshot)?
                }
                (false, None) => return Err(Error::NotFound(doc.to_string())),
                (true, Some(source)) => {
                    // Loro is authoritative while open, but a file edited while
                    // the document was closed is the newer state. Fold it in
                    // before the document is served.
                    let current = loaded.snapshot()?;
                    let parsed = self.projection.parse(doc, source)?;
                    if !snapshots_agree(&current, &parsed) {
                        crate::tier2::writeback::apply_external_edit(
                            &loaded,
                            &self.projection,
                            doc,
                            source,
                        )?;
                    }
                    loaded
                }
                // The Loro state survives a lost file; the next flush restores
                // the Markdown.
                (true, None) => loaded,
            };
            self.store.save(doc, &document)?;
            if let Some(source) = source {
                self.projected.insert(doc.clone(), source);
            }
            self.open.insert(doc.clone(), document);
        }

        let snapshot = self.snapshot_of(doc)?;
        let type_name = self.resolve_type(doc, &snapshot);
        if !was_open {
            self.derive(doc, type_name.as_ref(), &snapshot)?;
        }
        Ok(LoadedDocument {
            path: doc.clone(),
            type_name,
            snapshot,
        })
    }

    /// Write the Markdown file for an open document and re-derive its indices.
    ///
    /// Uses the streaming writer: only the blocks that changed since the last
    /// projection are rewritten, so an editor's cursor and undo history survive.
    pub fn flush(&mut self, doc: &DocPath) -> Result<()> {
        self.persist(doc)
    }

    /// Write a document whose only change is frontmatter.
    ///
    /// Skips the block half of the derivation: `blocks` and `blocks_fts` are
    /// untouched, and the graph is only re-extracted when the property could
    /// carry a link. This is the hot path for `data.mutate { op: "set" }`.
    pub fn flush_frontmatter(&mut self, doc: &DocPath) -> Result<()> {
        let snapshot = self.snapshot_of(doc)?;
        self.write_file(doc, &snapshot)?;
        let type_name = self.resolve_type(doc, &snapshot);
        self.index
            .update_frontmatter(doc, type_name.as_ref(), &snapshot.frontmatter)?;
        // Re-extract even when the new frontmatter has no links: that is how
        // removed links, a cleared title, or a changed type reach the graph.
        let links = crate::tier1::links::document_links(&snapshot);
        let title = snapshot.string_property("title");
        self.graph.reindex_document(
            doc,
            type_name.as_ref(),
            title.as_deref(),
            &snapshot.blocks,
            &links,
        )?;
        Ok(())
    }

    /// Write the file, the Loro state and the derived indices for one document.
    fn persist(&mut self, doc: &DocPath) -> Result<()> {
        let snapshot = self.snapshot_of(doc)?;
        self.write_file(doc, &snapshot)?;
        let type_name = self.resolve_type(doc, &snapshot);
        self.derive(doc, type_name.as_ref(), &snapshot)
    }

    /// Render and write one document's Markdown, then save its Loro state.
    fn write_file(&mut self, doc: &DocPath, snapshot: &DocumentSnapshot) -> Result<()> {
        let previous_source = self.projected.get(doc).cloned();
        let path = doc.to_fs_path(&self.root);
        let contents = crate::tier2::writeback::write_document(
            &path,
            &self.projection,
            doc,
            previous_source.as_deref(),
            snapshot,
            &[],
        )?;
        self.echo.expect(&path, &contents);
        self.projected.insert(doc.clone(), contents);
        let document = self
            .open
            .get(doc)
            .ok_or_else(|| Error::NotFound(doc.to_string()))?;
        self.store.save(doc, document)
    }

    /// Flush and forget an open document.
    pub fn close(&mut self, doc: &DocPath) -> Result<()> {
        if self.open.contains_key(doc) {
            self.flush(doc)?;
            self.open.remove(doc);
            self.projected.remove(doc);
        }
        Ok(())
    }

    /// Apply an external Markdown edit to an open or persisted document.
    pub fn apply_external_edit(&mut self, doc: &DocPath, source: &str) -> Result<()> {
        self.load(doc)?;
        let previous = self.projected.get(doc).cloned();
        let snapshot = {
            let document = self
                .open
                .get(doc)
                .ok_or_else(|| Error::NotFound(doc.to_string()))?;
            crate::tier2::writeback::apply_external_edit(document, &self.projection, doc, source)?;
            document.snapshot()?
        };
        // The file the editor wrote is now the base for the next patch, unless
        // the merge changed more than the editor did.
        let merged = self.projection.render(doc, &snapshot, &[])?;
        self.projected.insert(
            doc.clone(),
            if previous.as_deref() == Some(source) {
                source.to_string()
            } else {
                merged
            },
        );
        let type_name = self.resolve_type(doc, &snapshot);
        self.derive(doc, type_name.as_ref(), &snapshot)?;
        let document = self
            .open
            .get(doc)
            .ok_or_else(|| Error::NotFound(doc.to_string()))?;
        self.store.save(doc, document)
    }

    /// Move a document to a new path, keeping its block ids and updating every
    /// tier in one go.
    pub fn rename(&mut self, doc: &DocPath, to: &DocPath) -> Result<()> {
        self.move_document(doc, to, None)
    }

    /// Move a document into another type folder and stamp its `type`.
    ///
    /// Returns the violations the document still has against the target
    /// folder's schema; they are reported, not enforced.
    pub fn change_type(&mut self, doc: &DocPath, target: &TypeName) -> Result<Vec<String>> {
        let to = self.path_in_type(doc, target)?;

        let loaded = self.load(doc)?;
        let violations = self
            .projection
            .schema
            .as_ref()
            .map(|schema| {
                crate::tier1::validate_for_type(schema, &to, &loaded.snapshot, target.as_str())
            })
            .unwrap_or_default();
        self.move_document(doc, &to, Some(target.clone()))?;
        Ok(violations)
    }

    /// Where a document would live inside a type folder.
    pub fn path_in_type(&self, doc: &DocPath, target: &TypeName) -> Result<DocPath> {
        let folder = crate::tier3::schema::discover(&self.root)?
            .into_iter()
            .find(|folder| &folder.name == target)
            .ok_or_else(|| Error::NotFound(format!("type folder `{target}`")))?;
        let file_name = doc.as_str().rsplit('/').next().unwrap_or(doc.as_str());
        Ok(DocPath::new(format!("{}/{file_name}", folder.dir)))
    }

    /// Delete a document from every tier and from disk.
    pub fn delete(&mut self, doc: &DocPath) -> Result<()> {
        let path = doc.to_fs_path(&self.root);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
        }
        self.open.remove(doc);
        self.projected.remove(doc);
        self.store.remove(doc)?;
        self.index.delete_document(doc)?;
        self.graph.delete_document(doc)
    }

    /// Rebuild every derived index from the Markdown files on disk.
    ///
    /// Tier 3 is derived and recomputable: this is the recovery path after a
    /// schema change or a damaged index. Existing rows are cleared first, so a
    /// deleted or moved file cannot leave a stale row behind.
    pub fn rebuild_index(&mut self) -> Result<usize> {
        let schema = crate::tier1::schema::load_project_schema(&self.root)?;
        self.projection = HyaloProjection::with_schema(schema);
        let folders = crate::tier3::schema::discover(&self.root)?;
        self.index.clear()?;
        self.graph.clear()?;
        self.index.regenerate_views(&folders)?;
        self.types = folders
            .iter()
            .map(|TypeFolder { dir, name, .. }| (dir.clone(), name.clone()))
            .collect();

        let mut count = 0;
        for path in markdown_files(&self.root)? {
            let relative = path
                .strip_prefix(&self.root)
                .map_err(|e| Error::Invalid(e.to_string()))?;
            let doc = DocPath::new(relative);
            let source = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            let snapshot = self.projection.parse(&doc, &source)?;
            let type_name = self.resolve_type(&doc, &snapshot);
            self.derive(&doc, type_name.as_ref(), &snapshot)?;
            count += 1;
        }
        Ok(count)
    }

    /// Run a read-only SQL query against Tier 3.
    pub fn query_sql(&self, sql: &str) -> Result<Vec<serde_json::Value>> {
        self.index.query_sql(sql)
    }

    /// Lint a document on disk and return its findings.
    ///
    /// Findings are reported, not enforced: a document that violates its
    /// schema is still written and still indexed.
    pub fn lint(&self, doc: &DocPath) -> Result<Vec<crate::tier1::lint::LintFinding>> {
        crate::tier1::lint::lint_file(&self.root, doc, self.projection.schema.as_ref())
    }

    /// Run an openCypher query against the graph.
    pub fn query_graph(&self, cypher: &str) -> Result<Vec<serde_json::Value>> {
        self.graph.query(cypher)
    }

    /// BM25 full-text search.
    pub fn query_full_text(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::tier3::FullTextHit>> {
        self.index.search_full_text(query, limit)
    }

    /// Run a query against Tier 3: `sql` when it starts with `SELECT`/`WITH`,
    /// `graph` when it starts with `MATCH`.
    pub fn query(&self, request: &str) -> Result<Vec<serde_json::Value>> {
        if request
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("match")
        {
            return self.query_graph(request);
        }
        self.query_sql(request)
    }

    /// Mutable access to the graph, for per-document re-extraction.
    pub fn graph_mut(&mut self) -> &mut GraphIndex {
        &mut self.graph
    }

    /// Mutable access to the SQLite index (vector tier writes).
    pub fn index_mut(&mut self) -> &mut IndexDb {
        &mut self.index
    }

    /// An open document, for the RPC mutation surface.
    pub fn document(&self, doc: &DocPath) -> Option<&LoroDocument> {
        self.open.get(doc)
    }

    /// The current snapshot of an open document.
    pub fn snapshot_of(&self, doc: &DocPath) -> Result<DocumentSnapshot> {
        self.open
            .get(doc)
            .ok_or_else(|| Error::NotFound(doc.to_string()))?
            .snapshot()
    }

    /// The file text the document was last written from.
    pub fn projected_source(&self, doc: &DocPath) -> Option<&str> {
        self.projected.get(doc).map(String::as_str)
    }

    /// Replace the vector tier's view of a document. Callers own the embedder.
    pub fn derive_vectors<V: crate::vector::VectorIndex>(
        &mut self,
        vectors: &V,
        doc: &DocPath,
        snapshot: &DocumentSnapshot,
    ) -> Result<()> {
        let chunks: Vec<crate::vector::VectorChunk> = snapshot
            .blocks
            .iter()
            .filter(|block| !block.text.trim().is_empty())
            .map(|block| crate::vector::VectorChunk {
                doc_path: doc.clone(),
                block_id: block.id.clone(),
                text: block.text.clone(),
            })
            .collect();
        vectors.upsert_document(doc, &chunks)
    }

    /// Resolve a handle to the document it names, erroring when it is gone.
    pub fn resolve(&self, handle: &NodeHandle) -> Result<LoadedDocument> {
        self.load_existing(&handle.doc_path)
    }

    /// The current snapshot of a document, open or on disk, without deriving.
    ///
    /// This is the read path for mutations: it never touches Tier 3, so a
    /// `set` costs one CRDT write rather than a full re-derivation.
    pub fn snapshot_readonly(&self, doc: &DocPath) -> Result<DocumentSnapshot> {
        self.load_existing(doc).map(|loaded| loaded.snapshot)
    }

    /// Load a document without deriving indices, for read paths.
    pub fn load_existing(&self, doc: &DocPath) -> Result<LoadedDocument> {
        let snapshot = if self.open.contains_key(doc) {
            self.snapshot_of(doc)?
        } else {
            let loaded = self.store.load(doc)?;
            if loaded.raw().len_ops() == 0 {
                return Err(Error::NotFound(doc.to_string()));
            }
            loaded.snapshot()?
        };
        Ok(LoadedDocument {
            path: doc.clone(),
            type_name: self.type_for(doc),
            snapshot,
        })
    }

    fn resolve_type(&self, doc: &DocPath, snapshot: &DocumentSnapshot) -> Option<TypeName> {
        snapshot
            .string_property("type")
            .map(TypeName::new)
            .or_else(|| self.type_for(doc))
    }

    /// Move a document and keep every tier consistent.
    fn move_document(
        &mut self,
        doc: &DocPath,
        to: &DocPath,
        new_type: Option<TypeName>,
    ) -> Result<()> {
        if let Some(new_type) = &new_type {
            self.load(doc)?;
            let document = self
                .open
                .get(doc)
                .ok_or_else(|| Error::NotFound(doc.to_string()))?;
            document.set_property(
                "type",
                &crate::types::PropertyValue::String(new_type.as_str().into()),
            )?;
        }
        if self.open.contains_key(doc) {
            self.flush(doc)?;
            self.open.remove(doc);
            self.projected.remove(doc);
        }

        let from_path = doc.to_fs_path(&self.root);
        let to_path = to.to_fs_path(&self.root);
        if let Some(parent) = to_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        let from_loro = self.store.path_for(doc);
        let to_loro = self.store.path_for(to);
        if let Some(parent) = to_loro.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        if from_path.exists() {
            std::fs::rename(&from_path, &to_path).map_err(|e| Error::io(&from_path, e))?;
        }
        if from_loro.exists() {
            std::fs::rename(&from_loro, &to_loro).map_err(|e| Error::io(&from_loro, e))?;
        }
        self.store.remove(doc)?;
        self.index.delete_document(doc)?;
        self.graph.delete_document(doc)?;
        self.rebuild_index()?;
        self.load(to)?;
        Ok(())
    }

    /// Write every tier for one document.
    ///
    /// SQLite holds `documents`, `properties`, `blocks` and `blocks_fts` in a
    /// single transaction, so a failure leaves no half-derived state. The graph
    /// is a separate engine and re-extracts per document.
    fn derive(
        &mut self,
        doc: &DocPath,
        type_name: Option<&TypeName>,
        snapshot: &DocumentSnapshot,
    ) -> Result<()> {
        self.index.derive_document(doc, type_name, snapshot)?;
        let links = crate::tier1::links::document_links(snapshot);
        let title = snapshot.string_property("title");
        self.graph
            .reindex_document(doc, type_name, title.as_deref(), &snapshot.blocks, &links)
    }
}

/// Whether two snapshots carry the same content, ignoring freshly minted ids.
fn snapshots_agree(left: &DocumentSnapshot, right: &DocumentSnapshot) -> bool {
    left.frontmatter == right.frontmatter
        && left.blocks.len() == right.blocks.len()
        && left.blocks.iter().zip(&right.blocks).all(|(left, right)| {
            left.kind == right.kind && left.text == right.text && left.meta == right.meta
        })
}

/// A projection plus its schema, for callers that only need parsing.
pub fn projection_for(root: impl AsRef<Path>) -> Result<HyaloProjection> {
    let schema = crate::tier1::schema::load_project_schema(root.as_ref())?;
    Ok(HyaloProjection::with_schema(schema))
}

/// Every Markdown file under `root`, skipping hidden and generated directories.
fn markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|e| Error::io(&dir, e))? {
            let entry = entry.map_err(|e| Error::io(&dir, e))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored_dir(&name) {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if name.to_ascii_lowercase().ends_with(".md") {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Directories no tier walks into.
pub(crate) fn is_ignored_dir(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules"
}

/// The project's `.hirn/` layout.
pub fn layout(root: impl AsRef<Path>) -> ProjectLayout {
    ProjectLayout::new(root.as_ref().to_path_buf())
}
