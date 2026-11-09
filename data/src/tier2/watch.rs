//! The filesystem watcher: external edits, debounced, with echo suppression.
//!
//! `notify` watches the project root recursively, skipping `.hirn/` and
//! `.git/`. Raw events are coalesced for [`DEBOUNCE`] before the callback sees
//! them, so a single editor save that produces several filesystem events is
//! delivered once. Writes this process performed are hash-tracked in
//! [`EchoGuard`] so the watcher does not turn its own write-back into an
//! external edit (HIR-82 resolution).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher as NotifyWatcher};

use crate::error::{Error, Result};
use crate::types::DocPath;

/// One external change the watcher decided to report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEvent {
    /// Project-relative path of the changed file.
    pub path: DocPath,
    /// How the file changed.
    pub kind: FileEventKind,
}

/// The kind of change, reduced to what Tier 2 acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEventKind {
    Created,
    Modified,
    Removed,
}

/// External change debounce.
pub const DEBOUNCE: Duration = Duration::from_millis(300);

/// The watch debounce interval.
pub fn debounce_interval() -> Duration {
    DEBOUNCE
}

/// Hash of the bytes this process last wrote to a path.
///
/// The watcher compares every incoming change against this so its own
/// write-back never re-enters as an external edit.
#[derive(Debug, Clone, Default)]
pub struct EchoGuard {
    expected: Arc<Mutex<HashSet<String>>>,
}

impl EchoGuard {
    /// Record that this process is about to write `contents` to `path`.
    pub fn expect(&self, path: &Path, contents: &str) {
        let hash = blake3::hash(contents.as_bytes()).to_hex().to_string();
        let key = format!("{}#{}", path.display(), hash);
        if let Ok(mut guard) = self.expected.lock() {
            guard.insert(key);
        }
    }

    /// Whether the current file content matches an expected write.
    pub fn consume_echo(&self, path: &Path, contents: &str) -> bool {
        let hash = blake3::hash(contents.as_bytes()).to_hex().to_string();
        let key = format!("{}#{}", path.display(), hash);
        match self.expected.lock() {
            Ok(mut guard) => guard.remove(&key),
            Err(_) => false,
        }
    }
}

/// A running recursive watch over a project root.
pub struct Watcher {
    root: PathBuf,
    inner: notify::RecommendedWatcher,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Watcher {
    /// Start watching `root`, delivering debounced `.md` changes to `on_event`.
    ///
    /// The callback runs on the watcher's worker thread; it must not block.
    pub fn start(
        root: impl AsRef<Path>,
        on_event: impl Fn(FileEvent) + Send + 'static,
    ) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let base = root.clone();
        let (sender, receiver) = std::sync::mpsc::channel::<(PathBuf, FileEventKind)>();
        let mut inner = notify::recommended_watcher(move |result: notify::Result<Event>| {
            let Ok(event) = result else {
                return;
            };
            let kind = match event.kind {
                EventKind::Create(_) => FileEventKind::Created,
                EventKind::Remove(_) => FileEventKind::Removed,
                EventKind::Modify(_) => FileEventKind::Modified,
                _ => return,
            };
            for path in event.paths {
                if !is_markdown(&path) || is_ignored(&base, &path) {
                    continue;
                }
                let Ok(relative) = path.strip_prefix(&base) else {
                    continue;
                };
                let _ = sender.send((relative.to_path_buf(), kind));
            }
        })
        .map_err(|e| Error::Invalid(format!("watcher unavailable for {}: {e}", root.display())))?;
        inner
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| Error::Invalid(format!("cannot watch {}: {e}", root.display())))?;

        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            let mut pending: BTreeMap<PathBuf, FileEvent> = BTreeMap::new();
            loop {
                if worker_stop.load(Ordering::Relaxed) {
                    break;
                }
                match receiver.recv_timeout(DEBOUNCE) {
                    Ok((relative, kind)) => {
                        pending
                            .entry(relative.clone())
                            .and_modify(|event| event.kind = merge_kinds(event.kind, kind))
                            .or_insert_with(|| FileEvent {
                                path: DocPath::new(relative),
                                kind,
                            });
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        for (_, event) in std::mem::take(&mut pending) {
                            on_event(event);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            for (_, event) in pending {
                on_event(event);
            }
        });

        Ok(Self {
            root,
            inner,
            stop,
            worker: Some(worker),
        })
    }

    /// The watched root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Stop watching and join the debounce worker.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.inner.unwatch(&self.root);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Collapse the events one path emitted into the single kind worth reporting.
fn merge_kinds(previous: FileEventKind, next: FileEventKind) -> FileEventKind {
    match (previous, next) {
        (_, FileEventKind::Removed) => FileEventKind::Removed,
        (FileEventKind::Removed, FileEventKind::Created) => FileEventKind::Modified,
        (FileEventKind::Created, FileEventKind::Modified) => FileEventKind::Created,
        (_, next) => next,
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn is_ignored(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    relative.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        name == ".hirn" || name == ".git" || name == "target" || name == "node_modules"
    })
}
