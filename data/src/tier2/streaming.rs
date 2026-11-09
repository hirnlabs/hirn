//! Streaming writer: renders a document once, then writes only the block text
//! that changed since the last projection.
//!
//! An external editor usually touches one block. Replacing the whole file for
//! that would move the editor's cursor and invalidate its undo history, so the
//! writer patches the changed lines in place and leaves the rest of the file
//! byte-identical.

use std::ops::Range;

use crate::types::Block;

/// One contiguous edit: replace `bytes` of the old source with `replacement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub bytes: Range<usize>,
    pub replacement: String,
}

/// The block's text as it will be written, including any `<!-- ^id -->` line.
fn rendered_block(block: &Block, linked: bool) -> String {
    let mut text = block.text.clone();
    if linked {
        text.push_str("\n\n");
        text.push_str(&block.anchor());
    }
    text
}

/// Line spans of `source`, as `(start, end)` byte offsets without the newline.
fn line_spans(source: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut start = 0;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            spans.push(start..index);
            start = index + 1;
        }
    }
    if start < source.len() {
        spans.push(start..source.len());
    }
    spans
}

/// Find `needle` as whole lines inside `haystack`.
///
/// Returns `None` when the block's exact text no longer sits in the file, which
/// is the signal to fall back to a whole-file write.
fn find_block(source: &str, needle: &str) -> Option<Range<usize>> {
    if needle.is_empty() {
        return None;
    }
    let spans = line_spans(source);
    let needle_lines: Vec<&str> = needle.lines().collect();
    if needle_lines.len() > spans.len() {
        return None;
    }
    'outer: for start in 0..=spans.len() - needle_lines.len() {
        for (offset, expected) in needle_lines.iter().enumerate() {
            let span = &spans[start + offset];
            if source.get(span.clone()) != Some(*expected) {
                continue 'outer;
            }
        }
        let first = spans[start].start;
        let last = spans[start + needle_lines.len() - 1].end;
        return Some(first..last);
    }
    None
}

/// Compute the minimal edits that turn `source` into the document's blocks.
///
/// Blocks whose text is unchanged produce no edit. A block whose text changed
/// produces one edit over its lines. Returns `None` when the old text of any
/// changed block is no longer present, meaning the caller must write the whole
/// rendered document instead.
pub fn diff_edits(
    source: &str,
    old_blocks: &[Block],
    new_blocks: &[Block],
    linked_blocks: &[String],
) -> Option<Vec<Edit>> {
    let mut edits = Vec::new();
    for (index, block) in new_blocks.iter().enumerate() {
        let linked = linked_blocks.iter().any(|id| id == block.id.as_str());
        let previous = old_blocks.get(index);
        if previous.is_some_and(|old| old.text == block.text) {
            continue;
        }
        let rendered = rendered_block(block, linked);
        let Some(old) = previous else {
            // A block with no predecessor: append it after the last block.
            let anchor = old_blocks.last()?;
            let range = find_block(source, &anchor.text)?;
            let separator = if range.end >= source.len() {
                "\n\n"
            } else {
                "\n\n\n"
            };
            edits.push(Edit {
                bytes: range.end..range.end,
                replacement: format!("{separator}{rendered}"),
            });
            continue;
        };
        let range = find_block(source, &old.text)?;
        edits.push(Edit {
            bytes: range,
            replacement: rendered,
        });
    }
    // Deleted blocks: any old block past the new length.
    for block in old_blocks
        .iter()
        .skip(new_blocks.len().min(old_blocks.len()))
    {
        if new_blocks.iter().any(|new| new.id == block.id) {
            continue;
        }
        let range = find_block(source, &block.text)?;
        let mut start = range.start;
        // Take the blank line that separated it, so the file stays tidy.
        if source[..start].ends_with("\n\n") {
            start -= 1;
        }
        edits.push(Edit {
            bytes: start..range.end,
            replacement: String::new(),
        });
    }

    edits.sort_by_key(|edit| edit.bytes.start);
    // Overlapping edits cannot be applied in order; fall back to a full write.
    for pair in edits.windows(2) {
        if pair[0].bytes.end > pair[1].bytes.start {
            return None;
        }
    }
    let length = source.len();
    if edits.iter().any(|edit| edit.bytes.end > length) {
        return None;
    }
    Some(edits)
}

/// Apply edits back-to-front so earlier offsets stay valid.
///
/// Edits are rebased onto `base` by offset: the caller splices a different text
/// over the same frontmatter bytes, so an edit at `offset` lands at the same
/// block in either text.
pub fn apply_edits(base: &str, edits: &[Edit]) -> String {
    let mut out = base.to_string();
    for edit in edits.iter().rev() {
        if edit.bytes.end > out.len() {
            continue;
        }
        out.replace_range(edit.bytes.clone(), &edit.replacement);
    }
    out
}

/// The smallest set of line spans that differ between two texts, for callers
/// that only have the two sources.
pub fn changed_span(old: &str, new: &str) -> Option<Range<usize>> {
    let prefix = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let start = old[..prefix].rfind('\n').map_or(0, |index| index + 1);
    let suffix = old
        .bytes()
        .rev()
        .zip(new.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(old.len() - start);
    let end = old.len() - suffix;
    if end < start {
        return None;
    }
    Some(start..end)
}
