//! Tier 2: the Loro document schema, persistence, compaction and merge.

use hirn_data::MarkdownProjection;
use hirn_data::projection::DocumentSnapshot;
use hirn_data::tier1::HyaloProjection;
use hirn_data::tier2::writeback::apply_external_edit;
use hirn_data::tier2::{LoroDocument, LoroStore, doc_from_snapshot, snapshot_from_doc};
use hirn_data::types::{BlockId, BlockKind, DocPath, PropertyValue};

const SAMPLE: &str = "---
title: Schema
status: draft
---

# Heading

First paragraph.

Second paragraph.
";

fn sample_snapshot() -> DocumentSnapshot {
    HyaloProjection::new()
        .parse(&DocPath::new("note.md"), SAMPLE)
        .unwrap()
}

#[test]
fn document_holds_frontmatter_and_blocks() {
    let snapshot = sample_snapshot();
    let document = doc_from_snapshot(&snapshot).unwrap();
    let round_tripped = snapshot_from_doc(document.raw()).unwrap();

    assert_eq!(round_tripped.frontmatter.len(), snapshot.frontmatter.len());
    assert_eq!(round_tripped.blocks.len(), snapshot.blocks.len());
    let kinds: Vec<&str> = round_tripped
        .blocks
        .iter()
        .map(|b| b.kind.as_str())
        .collect();
    assert_eq!(kinds, vec!["heading", "paragraph", "paragraph"]);
    assert_eq!(round_tripped.blocks[0].heading_level(), Some(1));
    assert_eq!(round_tripped.blocks[2].text, "Second paragraph.");
}

#[test]
fn moving_and_deleting_blocks_keeps_ids_stable() {
    let snapshot = sample_snapshot();
    let document = doc_from_snapshot(&snapshot).unwrap();
    let second_id = snapshot.blocks[1].id.clone();

    document.move_block(1, 0).unwrap();
    let after_move = snapshot_from_doc(document.raw()).unwrap();
    assert_eq!(after_move.blocks[0].id, second_id, "move keeps the id");

    document.delete_block(&second_id).unwrap();
    let after_delete = snapshot_from_doc(document.raw()).unwrap();
    assert_eq!(after_delete.blocks.len(), 2);
    assert!(!after_delete.blocks.iter().any(|b| b.id == second_id));
}

#[test]
fn property_updates_replace_previous_value() {
    let document = LoroDocument::from_snapshot(&sample_snapshot()).unwrap();
    document
        .set_property("status", &PropertyValue::String("done".into()))
        .unwrap();
    document
        .set_property("priority", &PropertyValue::Number(2.0))
        .unwrap();
    let snapshot = document.snapshot().unwrap();
    assert_eq!(
        snapshot.property("status"),
        Some(&PropertyValue::String("done".into()))
    );
    assert_eq!(
        snapshot.property("priority"),
        Some(&PropertyValue::Number(2.0))
    );

    document.delete_property("status").unwrap();
    assert_eq!(document.snapshot().unwrap().property("status"), None);
}

#[test]
fn two_peers_converge_through_updates() {
    let a = LoroDocument::from_snapshot(&sample_snapshot()).unwrap();
    let b = LoroDocument::import(&a.export_snapshot().unwrap()).unwrap();

    a.append_block(BlockKind::Paragraph, "from a").unwrap();
    b.append_block(BlockKind::Paragraph, "from b").unwrap();

    // Exchange deltas both ways, as the sync transport would.
    let from_a = a.export_updates(&b.version_vector()).unwrap();
    b.apply_updates(&from_a).unwrap();
    let from_b = b.export_updates(&a.version_vector()).unwrap();
    a.apply_updates(&from_b).unwrap();

    let ma = a.snapshot().unwrap();
    let mb = b.snapshot().unwrap();
    assert_eq!(ma.blocks.len(), mb.blocks.len());
    assert!(
        ma.blocks.iter().any(|block| block.text == "from a")
            && ma.blocks.iter().any(|block| block.text == "from b"),
        "concurrent appends both survive"
    );
}

#[test]
fn external_edit_becomes_crdt_ops() {
    let document = LoroDocument::from_snapshot(&sample_snapshot()).unwrap();
    let projection = HyaloProjection::new();
    let doc = DocPath::new("note.md");
    let edited = SAMPLE.replace("Second paragraph.", "Rewritten paragraph.");
    let ops = apply_external_edit(&document, &projection, &doc, &edited).unwrap();
    assert!(!ops.is_empty(), "the text change is detected");
    let snapshot = document.snapshot().unwrap();
    assert!(
        snapshot
            .blocks
            .iter()
            .any(|b| b.text == "Rewritten paragraph."),
        "the file's new text is in the CRDT"
    );
    assert!(
        !snapshot
            .blocks
            .iter()
            .any(|b| b.text == "Second paragraph."),
        "the old text is gone"
    );
}

#[test]
fn append_then_edit_keeps_other_blocks() {
    let document = LoroDocument::from_snapshot(&sample_snapshot()).unwrap();
    let projection = HyaloProjection::new();
    let doc = DocPath::new("note.md");

    let rendered = projection
        .render(&doc, &document.snapshot().unwrap(), &[])
        .unwrap();
    let edited = format!("{rendered}\nAppended externally.\n");
    apply_external_edit(&document, &projection, &doc, &edited).unwrap();
    let snapshot = document.snapshot().unwrap();
    assert_eq!(snapshot.blocks.len(), 4);
    assert_eq!(snapshot.blocks[3].text, "Appended externally.");
}

#[test]
fn store_compacts_after_threshold_and_reloads() {
    let dir = tempfile::tempdir().unwrap();
    let store = LoroStore::new(dir.path());
    let doc = DocPath::new("folder/note.md");
    let document = LoroDocument::from_snapshot(&sample_snapshot()).unwrap();

    store.save(&doc, &document).unwrap();
    let path = store.path_for(&doc);
    assert!(path.exists(), "snapshot file written at .hirn/docs/...");

    let reloaded = store.load(&doc).unwrap();
    assert_eq!(
        snapshot_from_doc(reloaded.raw()).unwrap().blocks.len(),
        3,
        "reload restores the document"
    );

    // Push the op log past the compaction threshold and save again.
    for index in 0..hirn_data::tier2::store::COMPACTION_THRESHOLD {
        document
            .append_block(BlockKind::Paragraph, &format!("block {index}"))
            .unwrap();
    }
    store.save(&doc, &document).unwrap();
    let delta_files: Vec<_> = std::fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".delta"))
        .collect();
    assert!(delta_files.is_empty(), "compaction removes appended deltas");

    let reloaded = store.load(&doc).unwrap();
    assert_eq!(
        snapshot_from_doc(reloaded.raw()).unwrap().blocks.len(),
        3 + hirn_data::tier2::store::COMPACTION_THRESHOLD
    );
}

#[test]
fn block_ids_are_ten_characters_and_unique() {
    let first = BlockId::generate();
    let second = BlockId::generate();
    assert_eq!(first.as_str().len(), 10);
    assert_ne!(first, second);
    assert!("abc" != first.as_str());
}
