//! Write throughput: N frontmatter `set`s per second, over notes of several sizes.
//!
//! Run with `cargo test --release --test bench_mutate -- --nocapture`.
//! The numbers are wall-clock per mutation on this machine; they are meant to
//! catch regressions and show where the cost sits, not to be a benchmark suite.

use std::path::Path;
use std::time::{Duration, Instant};

use hirn_data::backend::DataBackend;
use hirn_data::types::{DocPath, PropertyValue};

const SCHEMA: &str = r#"
required = ["title"]

[properties.title]
type = "string"

[properties.status]
type = "string"
"#;

fn write_file(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// A note with `blocks` body paragraphs plus a two-key frontmatter.
fn note_with_blocks(blocks: usize) -> String {
    let mut note = String::from("---\ntitle: Bench note\nstatus: draft\n---\n\n");
    for index in 0..blocks {
        note.push_str(&format!("Paragraph {index} of the benchmark note.\n\n"));
    }
    note
}

fn fixture(blocks: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_file(dir.path(), "notes/schema.toml", SCHEMA);
    write_file(dir.path(), "notes/bench.md", &note_with_blocks(blocks));
    dir
}

/// Time `count` frontmatter sets through the backend's flush path.
fn measure(blocks: usize, count: usize) -> Duration {
    let dir = fixture(blocks);
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/bench.md");
    backend.load(&doc).unwrap();

    let started = Instant::now();
    for index in 0..count {
        let document = backend.document(&doc).unwrap();
        document
            .set_property(
                "status",
                &PropertyValue::String(format!("iteration-{index}")),
            )
            .unwrap();
        backend.flush_frontmatter(&doc).unwrap();
    }
    started.elapsed()
}

#[test]
fn report_set_throughput() {
    const COUNT: usize = 20;
    for blocks in [1usize, 10, 50] {
        let elapsed = measure(blocks, COUNT);
        let per_set = elapsed / COUNT as u32;
        let per_second = 1.0 / per_set.as_secs_f64();
        println!(
            "{blocks:>3} blocks: {per_set:?} per set, {per_second:.1} sets/s over {COUNT} sets"
        );
    }
}

#[test]
fn frontmatter_only_set_reaches_the_file() {
    let dir = fixture(1);
    let mut backend = DataBackend::open(dir.path()).unwrap();
    let doc = DocPath::new("notes/bench.md");
    backend.load(&doc).unwrap();

    let before = std::fs::read_to_string(dir.path().join("notes/bench.md")).unwrap();
    let body_before = before
        .split_once("\n---\n")
        .map(|(_, body)| body.to_string())
        .expect("the fixture has a frontmatter block");

    let document = backend.document(&doc).unwrap();
    document
        .set_property("status", &PropertyValue::String("published".into()))
        .unwrap();
    backend.flush_frontmatter(&doc).unwrap();

    let written = std::fs::read_to_string(dir.path().join("notes/bench.md")).unwrap();
    assert!(
        written.contains("status: published"),
        "a frontmatter-only change must reach the canonical file:\n{written}"
    );
    let body_after = written
        .split_once("\n---\n")
        .map(|(_, body)| body.to_string())
        .expect("the written file still has a frontmatter block");
    assert_eq!(
        body_after, body_before,
        "a frontmatter-only flush leaves the body byte-identical"
    );
}
