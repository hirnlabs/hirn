//! Markdown rendering: Loro-independent projection of a [`DocumentSnapshot`]
//! back to CommonMark + GFM with YAML frontmatter.

use hyalo_core::frontmatter::emit_yaml_map;
use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::projection::DocumentSnapshot;
use crate::types::{Block, BlockKind, DocPath};

/// Render a snapshot to Markdown source.
///
/// Frontmatter is written first when present. Blocks are separated by one blank
/// line. Block ids appear as `<!-- ^id -->` only for blocks in `linked_blocks`,
/// so ordinary files stay clean (HIR-78 resolution).
///
/// Rendering is idempotent: block text already carries its Markdown syntax
/// (the block's `text` is the source range, not a stripped payload), so a
/// rendered document parses back to the same block texts.
pub fn render_document(
    doc: &DocPath,
    snapshot: &DocumentSnapshot,
    linked_blocks: &[String],
) -> Result<String> {
    let _ = doc;
    let mut out = String::new();

    if !snapshot.frontmatter.is_empty() {
        let map: IndexMap<String, serde_json::Value> = snapshot
            .frontmatter
            .iter()
            .map(|(key, value)| (key.clone(), value.to_sql()))
            .collect();
        let yaml = emit_yaml_map(&map).map_err(|e| Error::Parse(e.to_string()))?;
        let yaml = compact_sequences(&yaml);
        out.push_str("---\n");
        out.push_str(yaml.trim_end());
        out.push_str("\n---\n\n");
    }

    for (index, block) in snapshot.blocks.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&render_block(block));
        out.push('\n');
        if linked_blocks.iter().any(|id| id == block.id.as_str()) {
            out.push_str(&format!("\n{}\n", block.anchor()));
        }
    }
    Ok(out)
}

fn render_block(block: &Block) -> String {
    match &block.kind {
        BlockKind::Task => {
            // The block text already contains the `- [x] ` marker.
            let checked = block.task_checked().unwrap_or(false);
            let wanted = if checked { "[x]" } else { "[ ]" };
            let trimmed = block.text.trim_start();
            if let Some(marker) = trimmed.get(..2)
                && matches!(marker, "- " | "* " | "+ ")
                && matches!(
                    trimmed.get(2..6),
                    Some("[ ] ") | Some("[x] ") | Some("[X] ")
                )
            {
                let after = trimmed.get(6..).unwrap_or_default();
                return format!("{marker}{wanted} {after}");
            }
            format!("- {wanted} {}", block.text)
        }
        _ => block.text.clone(),
    }
}

/// Pull indented sequences up under their key.
///
/// `hyalo-core` emits `key:\n  - item`; Hyalo-authored files and Obsidian both
/// use the flush form `key:\n- item`, and preserving the authored style keeps
/// a round trip from rewriting every list in the project.
fn compact_sequences(yaml: &str) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        out.push(line.to_string());
        let is_key_line = !line.starts_with([' ', '\t', '-']) && line.trim_end().ends_with(':');
        index += 1;
        if is_key_line {
            while index < lines.len() {
                let candidate = lines[index];
                if candidate.trim_start().starts_with("- ") && candidate.starts_with("  ") {
                    out.push(candidate[2..].to_string());
                    index += 1;
                } else {
                    break;
                }
            }
        }
    }
    let mut joined = out.join("\n");
    joined.push('\n');
    joined
}
