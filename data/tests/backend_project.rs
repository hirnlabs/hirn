//! End-to-end: a fixture project through the backend — load, round-trip, apply
//! an external edit, and answer frontmatter, full-text, vector and graph queries.

use std::path::Path;

use hirn_data::backend::DataBackend;
use hirn_data::types::{BlockId, DocPath, NodeHandle, PropertyValue, TypeName};
use hirn_data::vector::{VectorChunk, VectorHit, VectorIndex};
use hirn_data::{Error, Result};

const NOTE_SCHEMA: &str = r#"
required = ["title"]

[properties.title]
type = "string"

[properties.status]
type = "string"
"#;

const FIRST_NOTE: &str = "---
title: First note
status: draft
related:
  - '[[Second Note]]'
---

# Embedded storage

The index lives beside the project in a single SQLite database.

It also holds a vector table.
";

const SECOND_NOTE: &str = "---
title: Second note
status: done
---

A plain note with no heading.
";

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "notes/schema.toml", NOTE_SCHEMA);
    write_file(dir.path(), "notes/first.md", FIRST_NOTE);
    write_file(dir.path(), "notes/second.md", SECOND_NOTE);
    dir
}

/// A stand-in embedder: counts calls so caching can be asserted without ONNX.
#[derive(Default)]
struct StubEmbedder {
    embeds: std::sync::atomic::AtomicUsize,
}

impl VectorIndex for StubEmbedder {
    fn upsert_document(&self, _doc: &DocPath, chunks: &[VectorChunk]) -> Result<()> {
        self.embeds
            .fetch_add(chunks.len(), std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    fn delete_document(&self, _doc: &DocPath) -> Result<()> {
        Ok(())
    }
    fn search(&self, _query: &[f32], _k: usize) -> Result<Vec<VectorHit>> {
        Ok(Vec::new())
    }
    fn len(&self) -> Result<usize> {
        Ok(0)
    }
}

#[test]
fn backend_loads_a_document_and_derives_every_index() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();

    let loaded = backend.load(&DocPath::new("notes/first.md")).unwrap();
    assert_eq!(loaded.type_name.unwrap().as_str(), "notes");
    assert_eq!(backed_props(&loaded.snapshot).as_str(), "First note");
    assert_eq!(
        loaded.snapshot.blocks[0].kind.as_str(),
        "heading",
        "first block is the heading"
    );

    // Frontmatter query through the generated view.
    let rows = backend
        .query_sql("SELECT title, status FROM type_notes ORDER BY doc_path")
        .unwrap();
    assert_eq!(rows.len(), 1, "only the loaded document is indexed");
    assert_eq!(rows[0]["title"], "First note");

    // Full-text query.
    let hits = backend.query_full_text("vector", 5).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].handle.doc_path, DocPath::new("notes/first.md"));

    // Graph query: the frontmatter reference became an edge to a ghost node.
    let rows = backend
        .query("MATCH (d:Document)-[:RELATED_TO]->(t) RETURN t.path AS target")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["target"], "Second Note.md");

    // Vector derivation goes through the seam, so a stub counts the chunks.
    let vectors = StubEmbedder::default();
    backend
        .derive_vectors(&vectors, &DocPath::new("notes/first.md"), &loaded.snapshot)
        .unwrap();
    assert_eq!(
        vectors.embeds.load(std::sync::atomic::Ordering::Relaxed),
        3,
        "one chunk per non-empty block"
    );
}

fn backed_props(snapshot: &hirn_data::projection::DocumentSnapshot) -> String {
    match snapshot.property("title") {
        Some(PropertyValue::String(title)) => title.clone(),
        other => panic!("unexpected title {other:?}"),
    }
}

#[test]
fn a_frontmatter_set_leaves_blocks_and_fts_alone() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/first.md");
    backend.load(&doc).unwrap();

    let before: i64 = backend
        .query_sql("SELECT count(*) AS n FROM blocks_fts")
        .unwrap()[0]["n"]
        .as_i64()
        .unwrap();

    let document = backend.document(&doc).unwrap();
    document
        .set_property("status", &PropertyValue::String("review".into()))
        .unwrap();
    backend.flush_frontmatter(&doc).unwrap();

    // The property is live in the view immediately.
    let rows = backend.query_sql("SELECT status FROM type_notes").unwrap();
    assert_eq!(rows[0]["status"], "review");

    // The block half of Tier 3 is untouched, so FTS is not re-tokenised.
    let after: i64 = backend
        .query_sql("SELECT count(*) AS n FROM blocks_fts")
        .unwrap()[0]["n"]
        .as_i64()
        .unwrap();
    assert_eq!(before, after, "frontmatter writes leave FTS rows alone");
    assert_eq!(backend.query_full_text("vector", 5).unwrap().len(), 1);

    // A second load of an open document does not re-derive either.
    backend.load(&doc).unwrap();
    let written = std::fs::read_to_string(dir.path().join("notes/first.md")).unwrap();
    assert!(written.contains("status: review"), "file is canonical");
}

#[test]
fn backend_rebuilds_every_document_from_markdown() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let indexed = backend.rebuild_index().unwrap();
    assert_eq!(indexed, 2, "both Markdown files are indexed");

    let rows = backend
        .query_sql("SELECT doc_path, title FROM documents ORDER BY doc_path")
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1]["title"], "Second note");

    // The bind glob gives each note its folder's type even without `type:`.
    let rows = backend
        .query_sql("SELECT count(*) AS n FROM type_notes")
        .unwrap();
    assert_eq!(rows[0]["n"], 2);
}

#[test]
fn open_document_writes_back_and_absorbs_external_edits() {
    let dir = fixture();
    let first = dir.path().join("notes/first.md");
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/first.md");

    backend.load(&doc).unwrap();
    let document = backend.document(&doc).unwrap();
    let id: BlockId = document
        .append_block(hirn_data::BlockKind::Paragraph, "Added in the CRDT.")
        .unwrap();
    assert_eq!(id.as_str().len(), 10);
    backend.flush(&doc).unwrap();

    let written = std::fs::read_to_string(&first).unwrap();
    assert!(written.contains("Added in the CRDT."), "edit reached disk");
    assert!(
        written.starts_with("---\n") && written.contains("title: First note"),
        "frontmatter kept"
    );

    // An outside editor rewrites the file; the backend folds it in as CRDT ops.
    let external = format!("{written}\nA line from an external editor.\n");
    backend.apply_external_edit(&doc, &external).unwrap();
    let loaded = backend.load_existing(&doc).unwrap();
    assert!(
        loaded
            .snapshot
            .blocks
            .iter()
            .any(|block| block.text == "A line from an external editor."),
        "external text is now part of the document"
    );
    assert!(
        loaded
            .snapshot
            .blocks
            .iter()
            .any(|block| block.text == "Added in the CRDT."),
        "the earlier CRDT edit survived the external edit"
    );
}

#[test]
fn load_folds_in_edits_made_while_the_document_was_closed() {
    let dir = fixture();
    let doc = DocPath::new("notes/second.md");
    {
        let mut backend = DataBackend::open(dir.path()).unwrap();
        let loaded = backend.load(&doc).unwrap();
        assert!(
            loaded
                .snapshot
                .blocks
                .iter()
                .any(|block| block.text == "A plain note with no heading.")
        );
    }

    // An editor changes the file while no backend holds the document.
    let edited = SECOND_NOTE.replace("A plain note with no heading.", "Edited while closed.");
    std::fs::write(dir.path().join("notes/second.md"), edited).unwrap();

    let mut backend = DataBackend::open(dir.path()).unwrap();
    let loaded = backend.load(&doc).unwrap();
    let texts: Vec<String> = loaded
        .snapshot
        .blocks
        .iter()
        .map(|block| block.text.clone())
        .collect();
    assert!(
        texts.iter().any(|text| text == "Edited while closed."),
        "the disk edit is folded into the existing Loro state: {texts:?}"
    );
}

#[test]
fn loading_a_document_that_exists_nowhere_is_not_found() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    match backend.load(&DocPath::new("notes/absent.md")) {
        Err(Error::NotFound(name)) => assert!(name.contains("absent.md")),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn change_type_validates_against_the_target_type() {
    let dir = fixture();
    write_file(
        dir.path(),
        "journal/schema.toml",
        "required = [\"summary\"]\n",
    );
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/first.md");

    let violations = backend
        .change_type(&doc, &TypeName::new("journal"))
        .unwrap();
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("summary")),
        "the target type's required property is reported: {violations:?}"
    );
    assert!(
        !violations
            .iter()
            .any(|violation| violation.contains("title")),
        "the source type's required property is not applied: {violations:?}"
    );
    assert!(dir.path().join("journal/first.md").exists());
}

#[test]
fn rebuild_index_drops_rows_for_deleted_files() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    assert_eq!(backend.rebuild_index().unwrap(), 2);
    std::fs::remove_file(dir.path().join("notes/second.md")).unwrap();

    assert_eq!(backend.rebuild_index().unwrap(), 1);
    let rows = backend
        .query_sql("SELECT count(*) AS n FROM documents")
        .unwrap();
    assert_eq!(rows[0]["n"], 1, "the deleted file's rows are gone");
    let rows = backend
        .query_sql("SELECT count(*) AS n FROM type_notes")
        .unwrap();
    assert_eq!(rows[0]["n"], 1);
    let rows = backend
        .query("MATCH (d:Document {is_ghost: false}) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(rows[0]["n"], 1, "the deleted file's graph node is gone");
}

#[test]
fn clearing_the_last_link_removes_the_graph_edge() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/first.md");
    backend.load(&doc).unwrap();

    let rows = backend
        .query("MATCH ()-[r:RELATED_TO]->() RETURN count(r) AS n")
        .unwrap();
    assert_eq!(rows[0]["n"], 1, "the frontmatter link is indexed first");

    backend
        .document(&doc)
        .unwrap()
        .delete_property("related")
        .unwrap();
    backend.flush_frontmatter(&doc).unwrap();

    let rows = backend
        .query("MATCH ()-[r:RELATED_TO]->() RETURN count(r) AS n")
        .unwrap();
    assert_eq!(
        rows[0]["n"], 0,
        "clearing the last link removes the stale edge"
    );
}

#[test]
fn deleting_a_document_clears_every_tier() {
    let dir = fixture();
    let mut backend = DataBackend::open(dir.path()).unwrap();
    backend.rebuild_index().unwrap();
    let doc = DocPath::new("notes/second.md");

    std::fs::remove_file(dir.path().join("notes/second.md")).unwrap();
    backend.index_mut().delete_document(&doc).unwrap();
    backend.graph_mut().delete_document(&doc).unwrap();

    let rows = backend
        .query_sql("SELECT count(*) AS n FROM documents WHERE doc_path = 'notes/second.md'")
        .unwrap();
    assert_eq!(rows[0]["n"], 0);
    let rows = backend
        .query("MATCH (d:Document {path: 'notes/second.md'}) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(rows[0]["n"], 0);
}

#[test]
fn missing_documents_report_not_found() {
    let dir = fixture();
    let backend = DataBackend::open(dir.path()).unwrap();
    let handled = NodeHandle::block(DocPath::new("notes/absent.md"), BlockId::new("missing000"));
    match backend.resolve(&handled) {
        Err(Error::NotFound(name)) => assert!(name.contains("absent.md")),
        other => panic!("expected NotFound, got {other:?}"),
    }
}
