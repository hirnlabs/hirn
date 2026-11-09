//! Persistence of Loro documents at `<project>/.hirn/docs/<path>.loro`.
//!
//! Deltas append as separate files next to the snapshot; at 100 operations the
//! log compacts into a single snapshot that keeps the complete version vector
//! (HIR-82 resolution).

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::types::{DocPath, ProjectLayout};

use super::doc::LoroDocument;

/// Compact once the op log passes this many operations.
pub const COMPACTION_THRESHOLD: usize = 100;

/// Loads and saves per-document Loro state.
pub struct LoroStore {
    layout: ProjectLayout,
}

impl LoroStore {
    /// Wrap a project root.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            layout: ProjectLayout::new(root.as_ref().to_path_buf()),
        }
    }

    /// The project layout this store writes into.
    pub fn layout(&self) -> &ProjectLayout {
        &self.layout
    }

    /// Where a document's snapshot lives.
    pub fn path_for(&self, doc: &DocPath) -> PathBuf {
        self.layout.loro_path(doc)
    }

    /// Load a document, replaying any appended delta files.
    ///
    /// A missing snapshot yields an empty document; the caller decides whether
    /// that means "not yet projected" or "deleted".
    pub fn load(&self, doc: &DocPath) -> Result<LoroDocument> {
        let path = self.path_for(doc);
        let document = if path.exists() {
            let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
            LoroDocument::import(&bytes)?
        } else {
            LoroDocument::new()
        };
        for delta in self.delta_paths(doc)? {
            let bytes = std::fs::read(&delta).map_err(|e| Error::io(&delta, e))?;
            document.apply_updates(&bytes)?;
        }
        Ok(document)
    }

    /// Persist a document, compacting when the op log has grown past
    /// [`COMPACTION_THRESHOLD`].
    pub fn save(&self, doc: &DocPath, document: &LoroDocument) -> Result<()> {
        let path = self.path_for(doc);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        let frontier = document.version_vector();
        let last_compacted = self.last_compacted_ops(doc)?;
        let current_ops = document.oplog_len();
        let ops_since_compact = current_ops.saturating_sub(last_compacted);

        if ops_since_compact >= COMPACTION_THRESHOLD || !path.exists() {
            let bytes = document.export_snapshot()?;
            write_atomic(&path, &bytes)?;
            for delta in self.delta_paths(doc)? {
                let _ = std::fs::remove_file(delta);
            }
            self.remember_compacted_ops(doc, current_ops)?;
            return self.remember_version(doc, &frontier);
        }
        // Each delta holds only the operations written since the last save,
        // which is what keeps the log small between compactions.
        let delta_path = self.next_delta_path(doc)?;
        let bytes = document.export_updates(&self.saved_version(doc)?)?;
        write_atomic(&delta_path, &bytes)?;
        self.remember_version(doc, &frontier)
    }

    /// Delete a document's snapshot and deltas.
    pub fn remove(&self, doc: &DocPath) -> Result<()> {
        let path = self.path_for(doc);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| Error::io(&path, e))?;
        }
        for delta in self.delta_paths(doc)? {
            let _ = std::fs::remove_file(delta);
        }
        let _ = std::fs::remove_file(self.version_path(doc));
        let _ = std::fs::remove_file(self.compacted_ops_path(doc));
        Ok(())
    }

    fn last_compacted_ops(&self, doc: &DocPath) -> Result<usize> {
        let path = self.compacted_ops_path(doc);
        if !path.exists() {
            return Ok(0);
        }
        let content = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        Ok(content.trim().parse::<usize>().unwrap_or(0))
    }

    fn remember_compacted_ops(&self, doc: &DocPath, ops: usize) -> Result<()> {
        let path = self.compacted_ops_path(doc);
        write_atomic(&path, ops.to_string().as_bytes())
    }

    fn compacted_ops_path(&self, doc: &DocPath) -> PathBuf {
        self.path_for(doc).with_extension("loro.compacted_ops")
    }

    /// The version already persisted for a document: the snapshot's frontier
    /// plus every delta written after it.
    fn saved_version(&self, doc: &DocPath) -> Result<loro::VersionVector> {
        let path = self.version_path(doc);
        if !path.exists() {
            return Ok(loro::VersionVector::default());
        }
        let bytes = std::fs::read(&path).map_err(|e| Error::io(&path, e))?;
        Ok(loro::VersionVector::decode(&bytes)?)
    }

    fn remember_version(&self, doc: &DocPath, version: &loro::VersionVector) -> Result<()> {
        let path = self.version_path(doc);
        write_atomic(&path, &version.encode())
    }

    fn version_path(&self, doc: &DocPath) -> PathBuf {
        self.path_for(doc).with_extension("loro.vv")
    }

    fn next_delta_path(&self, doc: &DocPath) -> Result<PathBuf> {
        let base = self.path_for(doc);
        let mut counter = 0usize;
        loop {
            let candidate = base.with_extension(format!("loro.{counter}.delta"));
            if !candidate.exists() {
                return Ok(candidate);
            }
            counter += 1;
        }
    }

    fn delta_paths(&self, doc: &DocPath) -> Result<Vec<PathBuf>> {
        let base = self.path_for(doc);
        let Some(parent) = base.parent() else {
            return Ok(Vec::new());
        };
        if !parent.exists() {
            return Ok(Vec::new());
        }
        let prefix = base
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let mut deltas = Vec::new();
        for entry in std::fs::read_dir(parent).map_err(|e| Error::io(parent, e))? {
            let entry = entry.map_err(|e| Error::io(parent, e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(".delta") {
                deltas.push(entry.path());
            }
        }
        deltas.sort();
        Ok(deltas)
    }
}

/// Write bytes to `path` through a temporary file, fsync and rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    let tmp = path.with_extension("tmp");
    {
        let mut file = std::fs::File::create(&tmp).map_err(|e| Error::io(&tmp, e))?;
        file.write_all(bytes).map_err(|e| Error::io(&tmp, e))?;
        file.sync_all().map_err(|e| Error::io(&tmp, e))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| Error::io(path, e))?;
    Ok(())
}
