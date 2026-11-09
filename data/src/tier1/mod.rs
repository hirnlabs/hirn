//! Tier 1 — canonical Markdown: parsing, projection, schema, block diffing.
//!
//! Owns `hyalo-core` and `hyalo-mdlint`. Nothing here knows about Loro or
//! SQLite; the only outward door is the [`MarkdownProjection`] trait.

use hyalo_core::links::extract_links_from_text;
use hyalo_core::schema::SchemaConfig;

use crate::error::{Error, Result};
use crate::projection::{DocumentSnapshot, MarkdownProjection};
use crate::types::{Block, BlockId, BlockKind, DocPath, PropertyValue};

pub mod diff;
pub mod links;
pub mod lint;
pub mod render;
pub mod schema;

/// The V1 projection: CommonMark + GFM with YAML frontmatter.
#[derive(Debug, Clone, Default)]
pub struct HyaloProjection {
    /// Schema used for validation, when a project was loaded.
    pub schema: Option<SchemaConfig>,
}

impl HyaloProjection {
    /// A projection with no schema attached.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a schema for validation.
    pub fn with_schema(schema: SchemaConfig) -> Self {
        Self {
            schema: Some(schema),
        }
    }
}

impl MarkdownProjection for HyaloProjection {
    fn parse(&self, doc: &DocPath, source: &str) -> Result<DocumentSnapshot> {
        parse_document(doc, source)
    }

    fn render(
        &self,
        doc: &DocPath,
        snapshot: &DocumentSnapshot,
        linked_blocks: &[String],
    ) -> Result<String> {
        render::render_document(doc, snapshot, linked_blocks)
    }
}

/// Parse a Markdown document into frontmatter and blocks.
///
/// Frontmatter is read with `hyalo-core`'s framing parser so byte offsets,
/// BOMs and CRLF files behave exactly as they do for the rest of the hyalo
/// toolchain.
pub fn parse_document(doc: &DocPath, source: &str) -> Result<DocumentSnapshot> {
    let _ = doc;
    let frame = hyalo_core::frontmatter::DocumentFrame::parse(source)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let frontmatter = match frame.yaml(source.as_bytes()) {
        Some(yaml) => {
            let text = std::str::from_utf8(yaml)
                .map_err(|e| Error::Parse(format!("frontmatter is not UTF-8: {e}")))?;
            hyalo_core::frontmatter::parse_yaml_map(text)
                .map_err(|e| Error::Parse(e.to_string()))?
                .into_iter()
                .map(|(key, value)| (key, PropertyValue::from_json(&value)))
                .collect()
        }
        None => Vec::new(),
    };

    let body = frame.body(source.as_bytes()).unwrap_or_default();
    let (stripped_body, anchor_map) = extract_block_anchors(&String::from_utf8_lossy(body));
    let mut blocks = parse_blocks(&stripped_body);
    for (idx, block) in blocks.iter_mut().enumerate() {
        if let Some(preserved_id) = anchor_map.get(&idx) {
            block.id = preserved_id.clone();
        }
    }
    Ok(DocumentSnapshot {
        frontmatter,
        blocks,
    })
}

/// Extract `<!-- ^block-id -->` comments paired with the preceding block index.
fn extract_block_anchors(body: &str) -> (String, std::collections::HashMap<usize, BlockId>) {
    let mut out = String::with_capacity(body.len());
    let mut anchors = std::collections::HashMap::new();
    let mut block_index = 0usize;
    let mut prev_was_empty = true;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<!-- ^") && trimmed.ends_with("-->") {
            let id_str = trimmed
                .strip_prefix("<!-- ^")
                .and_then(|s| s.strip_suffix("-->"))
                .unwrap_or("")
                .trim();
            if !id_str.is_empty() && block_index > 0 {
                anchors.insert(block_index - 1, BlockId::new(id_str));
            }
            continue;
        }

        if trimmed.is_empty() {
            prev_was_empty = true;
        } else if prev_was_empty {
            block_index += 1;
            prev_was_empty = false;
        }

        out.push_str(line);
        out.push('\n');
    }
    (out, anchors)
}

/// Split a Markdown body into blocks at blank-line boundaries.
///
/// Consecutive non-blank lines form one block, which is what the block-level
/// Myers diff in [`diff`] operates on. A heading and a fenced code block are
/// each their own block, because the two kinds the CRDT schema addresses most
/// precisely are those.
pub fn parse_blocks(body: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut fence: Option<&str> = None;

    let flush = |current: &mut Vec<&str>, blocks: &mut Vec<Block>| {
        if current.is_empty() {
            return;
        }
        let text = current.join("\n");
        blocks.push(classify_block(&text));
        current.clear();
    };

    for line in body.lines() {
        let trimmed = line.trim_start();
        if let Some(marker) = fence {
            current.push(line);
            if trimmed.starts_with(marker) {
                fence = None;
                flush(&mut current, &mut blocks);
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            flush(&mut current, &mut blocks);
            fence = Some(if trimmed.starts_with("```") {
                "```"
            } else {
                "~~~"
            });
            current.push(line);
            continue;
        }
        if trimmed.is_empty() {
            flush(&mut current, &mut blocks);
            continue;
        }
        if is_atx_heading(trimmed) {
            flush(&mut current, &mut blocks);
            current.push(line);
            flush(&mut current, &mut blocks);
            continue;
        }
        current.push(line);
    }
    flush(&mut current, &mut blocks);
    blocks
}

fn is_atx_heading(line: &str) -> bool {
    let level = line.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&level) && line[level..].starts_with(' ')
}

fn classify_block(text: &str) -> Block {
    let mut block = Block::new(BlockId::generate(), BlockKind::Paragraph, text.to_string());
    let first = text.lines().next().unwrap_or("");

    if is_atx_heading(first) {
        let level = first.chars().take_while(|c| *c == '#').count();
        block.kind = BlockKind::Heading;
        block
            .meta
            .insert("level".into(), serde_json::Value::from(level));
    } else if first.starts_with("```") || first.starts_with("~~~") {
        block.kind = BlockKind::Code;
        let info = first.trim_matches(|c| c == '`' || c == '~').trim();
        if !info.is_empty() {
            block
                .meta
                .insert("language".into(), serde_json::Value::from(info));
        }
    } else if first.starts_with('>') {
        block.kind = BlockKind::Quote;
    } else if let Some(checked) = task_state(first) {
        block.kind = BlockKind::Task;
        block
            .meta
            .insert("checked".into(), serde_json::Value::from(checked));
    } else if is_list_item(first) {
        block.kind = BlockKind::ListItem;
    } else if first.trim().chars().all(|c| matches!(c, '-' | '*' | '_')) && first.trim().len() >= 3
    {
        block.kind = BlockKind::ThematicBreak;
    } else if first.trim_start().starts_with('|') {
        block.kind = BlockKind::Table;
    }
    block
}

fn task_state(line: &str) -> Option<bool> {
    let rest = line.trim_start().strip_prefix(['-', '*', '+'])?;
    let rest = rest.trim_start();
    if let Some(after) = rest.strip_prefix("[ ]") {
        let _ = after;
        Some(false)
    } else if rest
        .strip_prefix("[x]")
        .or_else(|| rest.strip_prefix("[X]"))
        .is_some()
    {
        Some(true)
    } else {
        None
    }
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix(['-', '*', '+']) {
        return rest.starts_with(' ');
    }
    let digits = trimmed.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && trimmed[digits..].starts_with(['.', ')'])
}

/// Whether the document's frontmatter satisfies the attached schema.
///
/// Returns one message per violation; an empty vector means valid.
pub fn validate(schema: &SchemaConfig, doc: &DocPath, snapshot: &DocumentSnapshot) -> Vec<String> {
    let explicit_type = snapshot.property("type").and_then(|value| match value {
        PropertyValue::String(s) => Some(s.clone()),
        _ => None,
    });
    let type_name =
        explicit_type.or_else(|| schema.bind.type_for(doc.as_str()).map(str::to_string));
    match type_name {
        Some(type_name) => validate_for_type(schema, doc, snapshot, &type_name),
        None => Vec::new(),
    }
}

/// Validate a document against one explicit type, ignoring path binding.
///
/// `change_type` validates against the folder the document is moving *to*,
/// not the folder it came from.
pub fn validate_for_type(
    schema: &SchemaConfig,
    doc: &DocPath,
    snapshot: &DocumentSnapshot,
    type_name: &str,
) -> Vec<String> {
    let mut violations = Vec::new();
    let merged = schema.merged_schema_for_type(type_name);
    for required in &merged.required {
        let present = snapshot
            .property(required)
            .is_some_and(|value| !matches!(value, PropertyValue::Null));
        if !present {
            violations.push(format!(
                "{}: missing required property `{required}` for type `{type_name}`",
                doc.as_str()
            ));
        }
    }
    violations
}

/// Wikilinks and markdown links written in one document's body.
pub fn body_links(snapshot: &DocumentSnapshot) -> Vec<hyalo_core::links::Link> {
    let mut out = Vec::new();
    for block in &snapshot.blocks {
        extract_links_from_text(&block.text, &mut out);
    }
    out
}

/// Read and parse a Markdown file from disk.
pub fn read_document(root: &std::path::Path, doc: &DocPath) -> Result<DocumentSnapshot> {
    let path = doc.to_fs_path(root);
    let source = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    parse_document(doc, &source)
}

/// The `hyalo-core` framing helper, re-exported for callers that need the raw
/// frontmatter bytes without a full parse.
pub use hyalo_core::frontmatter::read_frame as read_frame_export;
