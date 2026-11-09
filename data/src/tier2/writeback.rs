//! Write-back: CRDT edits reach disk through an atomic replace or a streaming
//! patch, and foreign edits are diffed into CRDT operations.
//!
//! Two distinct diffs:
//!
//! - **The merge** (Tier 2): the file against the last projection. Blocks are
//!   matched by id, then by text, then by position, so a block keeps its
//!   identity while its text changes.
//! - **The Myers block diff** (Tier 1): the block lists of two documents. Used
//!   to decide what changed, not how to merge it.

use std::path::Path;
use std::time::Duration;

use crate::error::Result;
use crate::projection::MarkdownProjection;
use crate::tier1::diff::diff_blocks;
use crate::types::DocPath;

use super::doc::LoroDocument;
use super::streaming::{apply_edits, diff_edits};

/// Flush edits to Markdown after this much inactivity.
pub const FLUSH_DEBOUNCE: Duration = Duration::from_millis(500);

/// The write-back debounce interval.
///
/// A host that owns a timer writes on this cadence; a burst of mutations then
/// becomes one file write instead of one per mutation.
pub fn flush_interval() -> Duration {
    FLUSH_DEBOUNCE
}

/// Write a document's Markdown to disk atomically.
pub fn atomic_write(path: &Path, contents: &str) -> Result<()> {
    super::store::write_atomic(path, contents.as_bytes())
}

/// Write a document, patching only what changed when possible.
///
/// `previous_source` is the file as it was last read or written. A changed
/// frontmatter block is patched through the frame's byte range; changed blocks
/// are patched as line ranges. When neither patch applies (a key was added or
/// removed, or the old text is gone) the whole document is rendered instead.
/// Returns the full text now on disk.
pub fn write_document<P: MarkdownProjection>(
    path: &Path,
    projection: &P,
    doc: &DocPath,
    previous_source: Option<&str>,
    snapshot: &crate::projection::DocumentSnapshot,
    linked_blocks: &[String],
) -> Result<String> {
    let rendered = projection.render(doc, snapshot, linked_blocks)?;
    let patched = previous_source.and_then(|source| {
        let previous = projection.parse(doc, source).ok()?;
        let frontmatter_changed = previous.frontmatter != snapshot.frontmatter;
        let blocks_changed = previous.blocks.len() != snapshot.blocks.len()
            || previous
                .blocks
                .iter()
                .zip(&snapshot.blocks)
                .any(|(old, new)| old.text != new.text);

        if !frontmatter_changed && !blocks_changed {
            return Some(source.to_string());
        }
        if frontmatter_changed {
            // Splice the rendered frontmatter over the old one, then patch the
            // body: a frontmatter change usually shifts the body by a few bytes,
            // and a full render is only the fallback when the body changed too.
            if !blocks_changed {
                return Some(splice_frontmatter(source, &rendered));
            }
            return Some(rendered.clone());
        }
        let edits = diff_edits(source, &previous.blocks, &snapshot.blocks, linked_blocks)?;
        if edits.is_empty() {
            return Some(source.to_string());
        }
        Some(apply_edits(source, &edits))
    });
    let contents = patched.unwrap_or(rendered);
    atomic_write(path, &contents)?;
    Ok(contents)
}

/// Replace the frontmatter block of `source` with the one in `rendered`.
///
/// `rendered` is the projection of a document whose body equals `source`'s, so
/// the first blank line after its closing `---` ends its frontmatter. A source
/// with no frontmatter is left alone: adding one changes more than the range.
fn splice_frontmatter(source: &str, rendered: &str) -> String {
    let Some(rendered_end) = frontmatter_end(rendered) else {
        return rendered.to_string();
    };
    let Some(source_end) = frontmatter_end(source) else {
        return rendered.to_string();
    };
    let mut out = String::with_capacity(source.len() + rendered_end);
    out.push_str(&rendered[..rendered_end]);
    out.push_str(&source[source_end..]);
    out
}

/// The byte offset just past a leading frontmatter block, delimiters included.
fn frontmatter_end(source: &str) -> Option<usize> {
    let rest = source.strip_prefix("---")?;
    if !rest.starts_with('\n') {
        return None;
    }
    let mut offset = 4;
    for line in source[offset..].split_inclusive('\n') {
        offset += line.len();
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(offset);
        }
    }
    None
}

/// Ingest an externally edited Markdown file into an open document.
///
/// Parses the file with the projection, merges it against the document's last
/// projection, and applies the result as fine-grained CRDT operations, so
/// concurrent edits converge instead of overwriting each other
/// (HIR-78 resolution).
pub fn apply_external_edit<P: MarkdownProjection>(
    document: &LoroDocument,
    projection: &P,
    doc: &DocPath,
    source: &str,
) -> Result<Vec<crate::tier1::diff::BlockOp>> {
    let parsed = projection.parse(doc, source)?;
    let current = document.snapshot()?;

    // What the block-level diff says changed, for the caller and for tests.
    let old: Vec<String> = current.blocks.iter().map(|b| b.text.clone()).collect();
    let new: Vec<String> = parsed.blocks.iter().map(|b| b.text.clone()).collect();
    let ops = diff_blocks(&old, &new);

    apply_snapshot(document, &current, &parsed)?;
    Ok(ops)
}

/// Fold `parsed` into `document`, keeping ids and merging text.
///
/// Preserves block identity:
/// 1. If a parsed block has an anchor comment (`<!-- ^id -->`), it matches that id.
/// 2. Otherwise it matches existing blocks by exact text, preserving position if moved.
/// 3. If a block's text changed, it keeps the id of the block at that position.
fn apply_snapshot(
    document: &LoroDocument,
    current: &crate::projection::DocumentSnapshot,
    parsed: &crate::projection::DocumentSnapshot,
) -> Result<()> {
    let mut survivors: Vec<crate::types::BlockId> = Vec::new();
    let mut mapped_blocks = Vec::new();

    for (pos, block) in parsed.blocks.iter().enumerate() {
        let matched_id = current
            .blocks
            .iter()
            .find(|existing| existing.id == block.id && !survivors.contains(&existing.id))
            .or_else(|| {
                current.blocks.iter().find(|existing| {
                    existing.text == block.text && !survivors.contains(&existing.id)
                })
            })
            .or_else(|| {
                current
                    .blocks
                    .get(pos)
                    .filter(|existing| !survivors.contains(&existing.id))
            })
            .map(|existing| existing.id.clone())
            .unwrap_or_else(crate::types::BlockId::generate);

        survivors.push(matched_id.clone());
        mapped_blocks.push((matched_id, block.clone()));
    }

    // Reconstruct blocks in document order with stable ids
    let blocks_container = document.blocks();
    blocks_container.clear()?;
    for (id, block) in mapped_blocks {
        document.insert_block_with_id_at(
            blocks_container.len(),
            id,
            block.kind,
            &block.text,
            block.meta,
        )?;
    }

    // Frontmatter: apply the file's values, drop the keys it removed.
    for (key, value) in &parsed.frontmatter {
        if current.property(key) != Some(value) {
            document.set_property(key, value)?;
        }
    }
    for (key, _) in &current.frontmatter {
        if parsed.property(key).is_none() {
            document.delete_property(key)?;
        }
    }
    document.commit();
    Ok(())
}
