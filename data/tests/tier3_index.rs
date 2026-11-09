//! Tier 3: the derived SQLite index, FTS5, vectors and the Grafeo graph.

use std::io::Write;

use hirn_data::tier3::IndexDb;
use hirn_data::tier3::graph::{GraphIndex, extract_links};
use hirn_data::tier3::schema::{TypeFolder, discover, schema_config};
use hirn_data::types::{Block, BlockId, BlockKind, DocPath, PropertyValue, TypeName};

fn write_project_file(root: &std::path::Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

const NOTE_SCHEMA: &str = r#"
required = ["title"]

[properties.title]
type = "string"

[properties.status]
type = "string"

[properties.priority]
type = "number"
"#;

fn note() -> DocPath {
    DocPath::new("notes/first.md")
}

fn notes_type() -> TypeName {
    TypeName::new("notes")
}

fn frontmatter() -> Vec<(String, PropertyValue)> {
    vec![
        (
            "title".to_string(),
            PropertyValue::String("First note".into()),
        ),
        ("status".to_string(), PropertyValue::String("draft".into())),
        ("priority".to_string(), PropertyValue::Number(2.0)),
    ]
}

#[test]
fn type_folders_are_discovered_with_their_schema() {
    let dir = tempfile::tempdir().unwrap();
    write_project_file(dir.path(), "notes/schema.toml", NOTE_SCHEMA);
    write_project_file(dir.path(), "notes/a.md", "---\ntitle: a\n---\n");
    write_project_file(dir.path(), ".hirn/schema.toml", "ignored = true\n");

    let folders = discover(dir.path()).unwrap();
    assert_eq!(folders.len(), 1, "dot directories are not type folders");
    let folder: &TypeFolder = &folders[0];
    assert_eq!(folder.name.as_str(), "notes");
    assert_eq!(folder.property_names(), vec!["priority", "status", "title"]);

    let config = schema_config(&folders);
    assert_eq!(
        config.bind.type_for("notes/a.md"),
        Some("notes"),
        "documents bind to their folder's type"
    );
}

#[test]
fn frontmatter_round_trips_through_documents_and_properties() {
    let mut db = IndexDb::open_in_memory().unwrap();
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();

    let rows = db
        .query_sql("SELECT doc_path, type_name, title FROM documents")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["doc_path"], "notes/first.md");
    assert_eq!(rows[0]["type_name"], "notes");
    assert_eq!(rows[0]["title"], "First note");

    let properties = db
        .query_sql("SELECT key, value, type FROM properties ORDER BY key")
        .unwrap();
    assert_eq!(properties.len(), 3);
    assert_eq!(properties[0]["key"], "priority");
    assert_eq!(properties[0]["value"], "2");

    // A second upsert replaces the previous rows rather than duplicating them.
    let updated = vec![("title".to_string(), PropertyValue::String("Renamed".into()))];
    db.upsert_document(&note(), Some(&notes_type()), &updated)
        .unwrap();
    let rows = db.query_sql("SELECT title FROM documents").unwrap();
    assert_eq!(rows[0]["title"], "Renamed");
    let count = db
        .query_sql("SELECT count(*) AS n FROM properties")
        .unwrap();
    assert_eq!(count[0]["n"], 1);
}

#[test]
fn generated_view_exposes_declared_properties() {
    let dir = tempfile::tempdir().unwrap();
    write_project_file(dir.path(), "notes/schema.toml", NOTE_SCHEMA);

    let mut db = IndexDb::open_in_memory().unwrap();
    let types = db.refresh_schema(dir.path()).unwrap();
    assert_eq!(types, vec![notes_type()]);

    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let rows = db
        .query_sql("SELECT doc_path, title, status, priority FROM type_notes")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["status"], "draft");
    assert_eq!(rows[0]["priority"], 2.0);

    // Documents of another type stay out of the view.
    db.upsert_document(
        &DocPath::new("notes/second.md"),
        Some(&TypeName::new("other")),
        &frontmatter(),
    )
    .unwrap();
    let rows = db
        .query_sql("SELECT count(*) AS n FROM type_notes")
        .unwrap();
    assert_eq!(rows[0]["n"], 1);

    // A partial index is in place for every declared property.
    let indexes = db
        .query_sql(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_type_notes%'",
        )
        .unwrap();
    assert_eq!(indexes.len(), 3);
}

#[test]
fn blocks_and_full_text_search_stay_in_sync() {
    let mut db = IndexDb::open_in_memory().unwrap();
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let blocks = vec![
        Block::new(BlockId::new("aaaa"), BlockKind::Heading, "# Embeddings"),
        Block::new(
            BlockId::new("bbbb"),
            BlockKind::Paragraph,
            "local first storage with vectors",
        ),
    ];
    db.replace_blocks(&note(), &blocks).unwrap();

    let hits = db.search_full_text("vectors", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].handle.block_id, Some(BlockId::new("bbbb")));
    assert_eq!(hits[0].handle.doc_path, note());

    // Replacing blocks replaces the FTS rows too.
    db.replace_blocks(&note(), &blocks[..1]).unwrap();
    assert!(db.search_full_text("vectors", 10).unwrap().is_empty());
    assert_eq!(db.search_full_text("Embeddings", 10).unwrap().len(), 1);
}

#[test]
fn sql_is_read_only() {
    let db = IndexDb::open_in_memory().unwrap();
    for statement in [
        "DELETE FROM documents",
        "INSERT INTO documents (doc_path) VALUES ('x')",
        "SELECT 1; DROP TABLE documents",
    ] {
        assert!(
            db.query_sql(statement).is_err(),
            "`{statement}` must be refused"
        );
    }
    assert!(db.query_sql("SELECT 1 AS one").is_ok());
}

#[test]
fn vectors_round_trip_through_sqlite_vec() {
    let mut db = IndexDb::open_in_memory().unwrap();
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let blocks = vec![
        Block::new(BlockId::new("aaaa"), BlockKind::Paragraph, "about rust"),
        Block::new(BlockId::new("bbbb"), BlockKind::Paragraph, "about cooking"),
    ];
    db.replace_blocks(&note(), &blocks).unwrap();

    let scalar = |first: f32, second: f32| {
        let mut vector = vec![0.0f32; hirn_data::EMBEDDING_DIM];
        vector[0] = first;
        vector[1] = second;
        vector
    };
    hirn_data::tier3::sqlite::upsert_vector(&mut db, &note(), "aaaa", &scalar(1.0, 0.0)).unwrap();
    hirn_data::tier3::sqlite::upsert_vector(&mut db, &note(), "bbbb", &scalar(0.0, 1.0)).unwrap();

    let hits = hirn_data::tier3::sqlite::search_vectors(&db, &scalar(1.0, 0.0), 2).unwrap();
    assert_eq!(hits.len(), 2, "k=2 returns both chunks");
    let ids: Vec<&str> = hits.iter().map(|(_, id, _)| id.as_str()).collect();
    assert!(ids.contains(&"aaaa") && ids.contains(&"bbbb"));

    let hits = hirn_data::tier3::sqlite::search_vectors(&db, &scalar(1.0, 0.0), 1).unwrap();
    assert_eq!(hits.len(), 1, "k caps the result set");

    let query = scalar(1.0, 0.0);

    // Deleting the document clears vectors and hashes.
    hirn_data::tier3::sqlite::delete_vectors(&mut db, &note()).unwrap();
    assert!(
        hirn_data::tier3::sqlite::search_vectors(&db, &query, 2)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn links_come_from_body_and_frontmatter() {
    let blocks = vec![Block::new(
        BlockId::new("aaaa"),
        BlockKind::Paragraph,
        "See [[Other Note]] and [[folder/third|alias]] but not [web](https://example.com).",
    )];
    let frontmatter = vec![(
        "related".to_string(),
        PropertyValue::List(vec![PropertyValue::String("[[Fourth]]".into())]),
    )];
    let links = extract_links(&frontmatter, &blocks);
    let targets: Vec<&str> = links.iter().map(|link| link.target.as_str()).collect();
    assert_eq!(targets, vec!["Other Note", "folder/third", "Fourth"]);
    assert_eq!(links[1].alias.as_deref(), Some("alias"));
    assert_eq!(links[2].relation, "RELATED_TO");
}

#[test]
fn fts_rowid_map_tracks_replacements_and_deletes() {
    let mut db = IndexDb::open_in_memory().unwrap();
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let blocks = vec![
        Block::new(BlockId::new("aaaa"), BlockKind::Paragraph, "alpha beta"),
        Block::new(BlockId::new("bbbb"), BlockKind::Paragraph, "gamma delta"),
    ];
    db.replace_blocks(&note(), &blocks).unwrap();

    let rows = db
        .query_sql("SELECT count(*) AS n FROM blocks_fts_map")
        .unwrap();
    assert_eq!(rows[0]["n"], 2);

    // Replacing with fewer blocks drops the stale rowid mapping, and the FTS
    // row with it, even though `doc_path` is UNINDEXED there.
    db.replace_blocks(&note(), &blocks[..1]).unwrap();
    let rows = db
        .query_sql("SELECT count(*) AS n FROM blocks_fts_map")
        .unwrap();
    assert_eq!(rows[0]["n"], 1);
    let rows = db
        .query_sql("SELECT count(*) AS n FROM blocks_fts")
        .unwrap();
    assert_eq!(rows[0]["n"], 1, "the stale FTS row goes with its map row");
    assert!(db.search_full_text("delta", 10).unwrap().is_empty());

    db.delete_document(&note()).unwrap();
    for table in ["blocks_fts_map", "blocks_fts", "blocks"] {
        let rows = db
            .query_sql(&format!("SELECT count(*) AS n FROM {table}"))
            .unwrap();
        assert_eq!(rows[0]["n"], 0, "{table} is cleared");
    }
}

#[test]
fn reindexing_keeps_inbound_edges_and_drops_orphan_ghosts() {
    let mut graph = GraphIndex::in_memory();
    let first = DocPath::new("first.md");
    let second = DocPath::new("second.md");

    let blocks = vec![Block::new(
        BlockId::new("aaaa"),
        BlockKind::Paragraph,
        "See [[second]] and [[missing]].",
    )];
    let links = extract_links(&[], &blocks);
    graph
        .reindex_document(&first, None, Some("First"), &blocks, &links)
        .unwrap();
    let owned = graph.query("MATCH (b:Block) RETURN count(b) AS n").unwrap();
    assert_eq!(owned[0]["n"], 1, "the block gets a :Block node");
    let contains = graph
        .query("MATCH ()-[r:CONTAINS]->() RETURN count(r) AS n")
        .unwrap();
    assert_eq!(contains[0]["n"], 1, "the document owns the block");

    // Re-index the target: the source's backlink survives, and the target is
    // updated in place rather than recreated.
    graph
        .reindex_document(&second, None, Some("Second"), &[], &[])
        .unwrap();
    let rows = graph
        .query(
            "MATCH (s:Document {path: 'first.md'})-[:LINKS_TO]->(t:Document {path: 'second.md'}) RETURN t.is_ghost AS ghost",
        )
        .unwrap();
    assert_eq!(rows.len(), 1, "the inbound edge survives: {rows:?}");
    assert_eq!(rows[0]["ghost"], false);
    let nodes = graph
        .query("MATCH (d:Document {path: 'second.md'}) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(nodes[0]["n"], 1, "the target is one node, updated in place");

    // Re-index the source without its links: the stale edges and the
    // now-unreferenced ghost disappear.
    graph
        .reindex_document(&first, None, Some("First"), &[], &[])
        .unwrap();
    let edges = graph
        .query("MATCH (s:Document {path: 'first.md'})-[r]->() RETURN count(r) AS n")
        .unwrap();
    assert_eq!(edges[0]["n"], 0, "outgoing edges are replaced");
    let ghosts = graph
        .query("MATCH (d:Document {is_ghost: true}) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(ghosts[0]["n"], 0, "an orphan ghost is cleaned up");
    let owned = graph.query("MATCH (b:Block) RETURN count(b) AS n").unwrap();
    assert_eq!(owned[0]["n"], 0, "the old :Block node is deleted");
    let contains = graph
        .query("MATCH ()-[r:CONTAINS]->() RETURN count(r) AS n")
        .unwrap();
    assert_eq!(contains[0]["n"], 0, "and so is its :CONTAINS edge");
}

#[test]
fn upsert_document_leaves_blocks_and_fts_untouched() {
    let mut db = IndexDb::open_in_memory().unwrap();
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let blocks = vec![Block::new(
        BlockId::new("aaaa"),
        BlockKind::Paragraph,
        "body text",
    )];
    db.replace_blocks(&note(), &blocks).unwrap();

    // `upsert_document` is documented as the frontmatter-only write.
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let rows = db.query_sql("SELECT count(*) AS n FROM blocks").unwrap();
    assert_eq!(rows[0]["n"], 1, "a frontmatter upsert leaves blocks alone");
    let rows = db
        .query_sql("SELECT count(*) AS n FROM blocks_fts")
        .unwrap();
    assert_eq!(rows[0]["n"], 1, "a frontmatter upsert leaves FTS alone");
    assert_eq!(db.search_full_text("body", 5).unwrap().len(), 1);
}

#[test]
fn cloned_index_handles_share_one_connection() {
    let mut db = IndexDb::open_in_memory().unwrap();
    let mut clone = db.clone();

    // A write through the original is visible through the clone.
    db.upsert_document(&note(), Some(&notes_type()), &frontmatter())
        .unwrap();
    let rows = clone.query_sql("SELECT title FROM documents").unwrap();
    assert_eq!(rows.len(), 1, "the clone shares the connection");

    // And a write through the clone is visible through the original.
    clone
        .upsert_document(
            &DocPath::new("notes/second.md"),
            Some(&notes_type()),
            &frontmatter(),
        )
        .unwrap();
    let rows = db.query_sql("SELECT count(*) AS n FROM documents").unwrap();
    assert_eq!(rows[0]["n"], 2, "there is one database, not two");
}

#[test]
fn deleting_one_chunk_leaves_the_rest() {
    let mut db = IndexDb::open_in_memory().unwrap();
    let vector = vec![0.0f32; hirn_data::EMBEDDING_DIM];
    for id in ["aaaa", "bbbb"] {
        hirn_data::tier3::sqlite::upsert_vector(&mut db, &note(), id, &vector).unwrap();
        hirn_data::tier3::sqlite::write_chunk_hash(&mut db, &note(), id, "hash").unwrap();
    }
    assert_eq!(db.vector_count().unwrap(), 2);

    hirn_data::tier3::sqlite::delete_chunk(&mut db, &note(), "aaaa").unwrap();
    assert_eq!(db.vector_count().unwrap(), 1);
    let hashes = hirn_data::tier3::sqlite::read_chunk_hashes(&db, &note()).unwrap();
    assert!(!hashes.contains_key("aaaa"));
    assert!(hashes.contains_key("bbbb"));
}

#[test]
fn plain_title_does_not_spawn_a_ghost() {
    let mut graph = GraphIndex::in_memory();
    let frontmatter = vec![(
        "title".to_string(),
        PropertyValue::String("Foo".to_string()),
    )];
    let blocks = vec![Block::new(
        BlockId::new("aaaa"),
        BlockKind::Paragraph,
        "No links here.",
    )];
    let links = extract_links(&frontmatter, &blocks);
    graph
        .reindex_document(&note(), Some(&notes_type()), Some("First"), &blocks, &links)
        .unwrap();

    let ghosts = graph
        .query("MATCH (d:Document {is_ghost: true}) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(ghosts[0]["n"], 0, "a plain title is not a link");
    let nodes = graph
        .query("MATCH (d:Document) RETURN count(d) AS n")
        .unwrap();
    assert_eq!(nodes[0]["n"], 1, "only the real document exists");
}

#[test]
fn graph_indexes_documents_edges_and_ghosts() {
    let mut graph = GraphIndex::in_memory();
    let blocks = vec![Block::new(
        BlockId::new("aaaa"),
        BlockKind::Paragraph,
        "Links to [[second]].",
    )];
    let links = extract_links(&[], &blocks);
    graph
        .reindex_document(&note(), Some(&notes_type()), Some("First"), &blocks, &links)
        .unwrap();

    let rows = graph
        .query("MATCH (d:Document)-[:LINKS_TO]->(t) RETURN d.path AS source, t.path AS target, t.is_ghost AS ghost")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["source"], "notes/first.md");
    assert_eq!(rows[0]["target"], "second.md");
    assert_eq!(rows[0]["ghost"], true, "an unresolved link is a ghost node");

    // Re-indexing the same document replaces, rather than duplicates, its edges.
    graph
        .reindex_document(&note(), Some(&notes_type()), Some("First"), &blocks, &[])
        .unwrap();
    let rows = graph
        .query("MATCH (d:Document)-[:LINKS_TO]->(t) RETURN d.path AS source, t.path AS target")
        .unwrap();
    assert!(rows.is_empty());
}
