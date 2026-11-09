//! Tier 1 linting through `hyalo-mdlint`: schema, body rules, HYALO native rules.

use hirn_data::MarkdownProjection;
use hirn_data::tier1::HyaloProjection;
use hirn_data::tier1::lint::{lint_file, lint_source};
use hirn_data::types::DocPath;

const SCHEMA: &str = r#"
required = ["title"]

[properties.title]
type = "string"

[properties.status]
type = "enum"
values = ["draft", "done"]
"#;

fn schema() -> hyalo_core::schema::SchemaConfig {
    hyalo_core::schema::SchemaConfig {
        default: hyalo_core::schema::TypeSchema {
            required: vec!["title".to_string()],
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn clean_document_has_no_findings() {
    let dir = tempfile::tempdir().unwrap();
    let doc = DocPath::new("notes/clean.md");
    let path = dir.path().join("notes/clean.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = "---\ntitle: Clean\nstatus: draft\n---\n\n# Heading\n\nA tidy paragraph.\n";
    std::fs::write(&path, source).unwrap();

    let findings = lint_source(&doc, &path, source, Some(&schema())).unwrap();
    let rules: Vec<&str> = findings.iter().map(|f| f.rule.as_str()).collect();
    assert!(
        !rules.contains(&"title"),
        "a required `title` present means no schema finding: {findings:?}"
    );
}

#[test]
fn missing_required_property_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let doc = DocPath::new("notes/untitled.md");
    let path = dir.path().join("notes/untitled.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = "---\nstatus: draft\n---\n\nBody.\n";
    std::fs::write(&path, source).unwrap();

    let findings = lint_source(&doc, &path, source, Some(&schema())).unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("title")),
        "the missing required property is reported: {findings:?}"
    );
}

#[test]
fn bare_checkbox_is_reported_by_the_hyalo_rule() {
    let dir = tempfile::tempdir().unwrap();
    let doc = DocPath::new("notes/tasks.md");
    let path = dir.path().join("notes/tasks.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = "---\ntitle: Tasks\n---\n\n- [] not a checkbox\n";
    std::fs::write(&path, source).unwrap();

    let findings = lint_source(&doc, &path, source, None).unwrap();
    assert!(
        findings.iter().any(|finding| finding.rule == "HYALO001"),
        "bare `[]` is flagged as HYALO001: {findings:?}"
    );
    assert!(
        findings.iter().any(|finding| finding.line >= 2),
        "findings carry a body line number: {findings:?}"
    );
}

#[test]
fn linting_a_missing_file_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let doc = DocPath::new("notes/absent.md");
    let findings = lint_file(dir.path(), &doc, Some(&schema())).unwrap();
    assert!(
        findings.is_empty(),
        "a missing file has nothing to report: {findings:?}"
    );
}

#[test]
fn frontmatter_hyalo_rules_are_wired() {
    let dir = tempfile::tempdir().unwrap();
    let doc = DocPath::new("notes/dates.md");
    let path = dir.path().join("notes/dates.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let source = "---\ntitle:\n  - not\n  - scalar\ndate: not-a-date\npublished: not-a-datetime\n---\n\nBody.\n";
    std::fs::write(&path, source).unwrap();

    let mut config = hyalo_core::schema::SchemaConfig::default();
    config.types.insert(
        "notes".to_string(),
        hyalo_core::schema::TypeSchema {
            properties: [(
                "published".to_string(),
                hyalo_core::schema::PropertyConstraint::DateTime,
            )]
            .into(),
            ..Default::default()
        },
    );

    let findings = lint_source(&doc, &path, source, Some(&config)).unwrap();
    let rules: Vec<&str> = findings
        .iter()
        .map(|finding| finding.rule.as_str())
        .collect();
    assert!(
        rules.contains(&"HYALO007"),
        "a list title is not a scalar: {findings:?}"
    );
    assert!(
        rules.contains(&"HYALO003"),
        "a known date key with a bad value: {findings:?}"
    );
    assert!(
        rules.contains(&"HYALO004"),
        "a schema-declared datetime property with a bad value: {findings:?}"
    );
}

#[test]
fn projection_round_trips_a_linted_document() {
    // Linting must not be a precondition for writing: a document with a
    // finding still parses and renders.
    let projection = HyaloProjection::new();
    let doc = DocPath::new("notes/untitled.md");
    let source = "---\nstatus: draft\n---\n\n- [] not a checkbox\n";
    let parsed = projection.parse(&doc, source).unwrap();
    let rendered = projection.render(&doc, &parsed, &[]).unwrap();
    let reparsed = projection.parse(&doc, &rendered).unwrap();
    assert_eq!(parsed.blocks.len(), reparsed.blocks.len());
}

#[test]
fn the_fixture_schema_toml_is_the_one_the_project_uses() {
    // Guards the schema text above against drift from what type folders accept.
    let parsed: hirn_data::tier3::schema::RawTypeSchema = toml::from_str(SCHEMA).unwrap();
    assert_eq!(parsed.required, vec!["title".to_string()]);
    assert_eq!(parsed.properties.len(), 2);
}
