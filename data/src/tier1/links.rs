//! Link extraction from Markdown, for the graph tier.
//!
//! Delegates the syntax to `hyalo-core` (which already handles code fences,
//! inline code, and inert zones) and normalises the targets into graph edges.
//! The extracted shape lives here, in Tier 1, so the graph tier consumes it
//! instead of defining a second copy.

use crate::projection::DocumentSnapshot;
use crate::types::{Block, PropertyValue};

/// One extracted link, from the body or from a frontmatter property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedLink {
    /// Wikilink target, or the frontmatter reference value.
    pub target: String,
    /// `#anchor` or `^block-id` fragment, without the leading `#`.
    pub anchor: Option<String>,
    /// `[[target|alias]]` display text.
    pub alias: Option<String>,
    /// Edge type: `LINKS_TO` for wikilinks, `AUTHORED_BY` etc. for frontmatter.
    pub relation: String,
}

/// Extract wikilinks and frontmatter references from a document.
pub fn extract_links(
    frontmatter: &[(String, PropertyValue)],
    blocks: &[Block],
) -> Vec<ExtractedLink> {
    let mut links = Vec::new();
    for block in blocks {
        let mut found = Vec::new();
        hyalo_core::links::extract_links_from_text(&block.text, &mut found);
        for link in found {
            if hyalo_core::links::is_external_target(&link.target) {
                continue;
            }
            links.push(ExtractedLink {
                target: link.target,
                anchor: link.fragment,
                alias: link.label,
                relation: "LINKS_TO".to_string(),
            });
        }
    }
    for (key, value) in frontmatter {
        let relation = key_to_relation(key);
        collect_property_links(value, &relation, &mut links);
    }
    links
}

fn key_to_relation(key: &str) -> String {
    match key {
        "author" => "AUTHORED_BY".to_string(),
        "related" => "RELATED_TO".to_string(),
        "implements" => "IMPLEMENTS".to_string(),
        "parent" => "PARENT_OF".to_string(),
        "tags" => "TAGGED".to_string(),
        other => other.to_uppercase().replace('-', "_"),
    }
}

fn collect_property_links(value: &PropertyValue, relation: &str, out: &mut Vec<ExtractedLink>) {
    match value {
        PropertyValue::String(raw) => {
            if let Some(target) = wikilink_target(raw) {
                out.push(ExtractedLink {
                    target,
                    anchor: None,
                    alias: None,
                    relation: relation.to_string(),
                });
            }
        }
        PropertyValue::List(items) => {
            for item in items {
                collect_property_links(item, relation, out);
            }
        }
        _ => {}
    }
}

/// `[[Target]]` or `[[Target|alias]]` -> `Target`.
///
/// Only explicit `[[wikilinks]]` are extracted as links. Plain strings in
/// frontmatter are properties, not link edges (prevents plain text like
/// `title: Foo` from generating unintended ghost nodes).
fn wikilink_target(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(inner) = trimmed
        .strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
    {
        let target = inner.split('|').next().unwrap_or(inner);
        let target = target.split('#').next().unwrap_or(target);
        return Some(target.trim().to_string());
    }
    None
}

/// `[[folder/note]]` and `[[note]]` both resolve to a project-relative `.md`.
pub(crate) fn normalize_doc_path(target: &str) -> String {
    let trimmed = target.trim().trim_matches('"');
    if trimmed.ends_with(".md") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.md")
    }
}

/// Every `[[wikilink]]` written in a document's body or frontmatter.
pub fn document_links(snapshot: &DocumentSnapshot) -> Vec<ExtractedLink> {
    extract_links(&snapshot.frontmatter, &snapshot.blocks)
}
