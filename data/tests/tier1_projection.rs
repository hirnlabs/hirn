//! Tier 1 projection: parse, render, round-trip, block kinds and diffing.

use hirn_data::MarkdownProjection;
use hirn_data::tier1::HyaloProjection;
use hirn_data::tier1::diff::{BlockOp, diff_blocks};
use hirn_data::types::DocPath;

const SAMPLE: &str = "---
title: Meeting notes
tags:
  - work
  - planning
status: draft
---

# Agenda

Intro paragraph with a [[wiki link]] and a [normal link](https://example.com).

- [x] write the plan

- [ ] review it

```rust
fn main() {}
```
";

#[test]
fn parses_frontmatter_and_blocks() {
    let projection = HyaloProjection::new();
    let doc = DocPath::new("notes/meeting.md");
    let snapshot = projection.parse(&doc, SAMPLE).unwrap();

    let keys: Vec<&str> = snapshot
        .frontmatter
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();
    assert_eq!(keys, vec!["title", "tags", "status"]);

    let kinds: Vec<&str> = snapshot.blocks.iter().map(|b| b.kind.as_str()).collect();
    assert_eq!(
        kinds,
        vec!["heading", "paragraph", "task", "task", "code"],
        "blank lines separate blocks and the fence stays whole"
    );
    assert_eq!(snapshot.blocks[0].heading_level(), Some(1));
    assert_eq!(snapshot.blocks[2].task_checked(), Some(true));
    assert_eq!(snapshot.blocks[3].task_checked(), Some(false));
    assert!(
        snapshot.blocks[4].text.contains("fn main"),
        "code block keeps its body"
    );
}

#[test]
fn render_round_trips_through_parse() {
    let projection = HyaloProjection::new();
    let doc = DocPath::new("notes/meeting.md");
    let parsed = projection.parse(&doc, SAMPLE).unwrap();
    let rendered = projection.render(&doc, &parsed, &[]).unwrap();
    assert!(
        !rendered.contains("^"),
        "no block anchors when nothing links"
    );
    let reparsed = projection.parse(&doc, &rendered).unwrap();
    let original_texts: Vec<&str> = parsed.blocks.iter().map(|b| b.text.as_str()).collect();
    let reparsed_texts: Vec<&str> = reparsed.blocks.iter().map(|b| b.text.as_str()).collect();
    assert_eq!(original_texts, reparsed_texts);
    assert_eq!(parsed.frontmatter.len(), reparsed.frontmatter.len());
}

#[test]
fn linked_blocks_get_html_comment_anchors() {
    let projection = HyaloProjection::new();
    let doc = DocPath::new("note.md");
    let snapshot = projection.parse(&doc, "one\n\ntwo\n").unwrap();
    let first = snapshot.blocks[0].id.clone();
    let rendered = projection
        .render(&doc, &snapshot, &[first.to_string()])
        .unwrap();
    assert!(
        rendered.contains(&format!("<!-- ^{first} -->")),
        "anchored block is marked in the file"
    );
    // The anchor does not become part of the block text on reparse.
    let reparsed = projection.parse(&doc, &rendered).unwrap();
    assert_eq!(reparsed.blocks[0].text, "one");
    assert!(
        !reparsed.blocks.iter().any(|b| b.text.contains("<!--")),
        "anchors stay out of block text"
    );
}

#[test]
fn diff_reports_in_place_update() {
    let old = vec!["alpha".to_string(), "beta".to_string()];
    let new = vec!["alpha".to_string(), "gamma".to_string()];
    let ops = diff_blocks(&old, &new);
    assert_eq!(
        ops,
        vec![BlockOp::Update {
            index: 1,
            text: "gamma".to_string()
        }],
        "a changed block keeps its position as one update"
    );
}

#[test]
fn diff_reports_move() {
    let old = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let new = vec!["gamma".to_string(), "alpha".to_string(), "beta".to_string()];
    let ops = diff_blocks(&old, &new);
    assert!(
        ops.contains(&BlockOp::Move { from: 2, to: 0 }),
        "reordered text is a move: {ops:?}"
    );
    assert!(
        !ops.iter().any(|op| matches!(op, BlockOp::Update { .. })),
        "a move is not also an update"
    );
}

#[test]
fn diff_reports_pure_delete_and_insert() {
    let deleted = diff_blocks(
        &["alpha".to_string(), "beta".to_string()],
        &["alpha".to_string()],
    );
    assert_eq!(deleted, vec![BlockOp::Delete { index: 1 }]);

    let inserted = diff_blocks(
        &["alpha".to_string()],
        &["alpha".to_string(), "gamma".to_string()],
    );
    assert_eq!(
        inserted,
        vec![BlockOp::Insert {
            index: 1,
            text: "gamma".to_string()
        }]
    );
}

#[test]
fn diff_is_noop_for_identical_input() {
    let old = vec!["alpha".to_string()];
    let ops = diff_blocks(&old, &["alpha".to_string()]);
    assert!(ops.is_empty());
}
