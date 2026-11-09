//! Shared contracts: document paths stay inside the project, and a
//! `schema.toml` declaration reaches `hyalo-core` intact.

use hirn_data::tier1::schema::schema_config;
use hirn_data::tier3::schema::discover;
use hirn_data::types::{DocPath, ProjectLayout};
use hyalo_core::schema::PropertyConstraint;

#[test]
fn doc_path_drops_escaping_and_absolute_components() {
    assert_eq!(DocPath::new("../../etc/passwd").as_str(), "etc/passwd");
    assert_eq!(
        DocPath::new("notes/../../outside.md").as_str(),
        "outside.md"
    );
    assert_eq!(
        DocPath::new("/absolute/note.md").as_str(),
        "absolute/note.md"
    );
    assert_eq!(
        DocPath::new("C:\\Users\\marten\\note.md").as_str(),
        "Users/marten/note.md"
    );
    assert_eq!(DocPath::new(".\\notes\\.\\a.md").as_str(), "notes/a.md");

    for raw in [
        "../escape.md",
        "a/../../b.md",
        "..\\..\\escape.md",
        "/root.md",
        "../../../",
    ] {
        let path = DocPath::new(raw);
        assert!(!path.as_str().contains(".."), "{raw} -> {path}");
        assert!(!path.as_str().contains('\\'), "{raw} -> {path}");
        assert!(!path.as_str().starts_with('/'), "{raw} -> {path}");
    }
}

#[test]
fn doc_path_deserialization_is_normalized() {
    // Deriving `Deserialize` would let a payload smuggle `..` or a drive
    // prefix into the inner string, so the payload goes through `new`.
    let path: DocPath = serde_json::from_str("\"../../escape.md\"").unwrap();
    assert_eq!(path.as_str(), "escape.md");
    assert_eq!(serde_json::to_string(&path).unwrap(), "\"escape.md\"");

    let nested: Vec<DocPath> =
        serde_json::from_str("[\"notes/./a.md\", \"C:\\\\x\\\\b.md\"]").unwrap();
    assert_eq!(nested[0].as_str(), "notes/a.md");
    assert_eq!(nested[1].as_str(), "x/b.md");
}

#[test]
fn resolved_paths_stay_below_their_root() {
    let root = std::path::PathBuf::from("C:/project");
    let doc = DocPath::new("../../escape.md");

    assert_eq!(doc.to_fs_path(&root), root.join("escape.md"));

    let layout = ProjectLayout::new(root);
    let loro = layout.loro_path(&doc);
    assert_eq!(loro, layout.docs_dir().join("escape.md.loro"));
    assert!(loro.starts_with(layout.docs_dir()));

    let nested = layout.loro_path(&DocPath::new("notes/deep/a.md"));
    assert_eq!(nested, layout.docs_dir().join("notes/deep/a.md.loro"));
}

#[test]
fn schema_toml_constraints_reach_hyalo_core() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("books")).unwrap();
    std::fs::write(
        dir.path().join("books/schema.toml"),
        r#"
required = ["title"]
filename = "{title}.md"

[properties.title]
type = "string"
pattern = "^[A-Z]"
min_length = 3
max_length = 80

[properties.pages]
type = "number"
minimum = 1
maximum = 2000

[properties.genre]
type = "enum"
values = ["fiction", "nonfiction"]

[properties.contributors]
type = "object-list"
required_keys = ["name"]
allowed_keys = ["name", "role"]
"#,
    )
    .unwrap();

    let folders = discover(dir.path()).unwrap();
    assert_eq!(folders.len(), 1);
    let config = schema_config(&folders);
    let books = config.types.get("books").expect("the type is declared");
    assert_eq!(books.required, vec!["title".to_string()]);
    assert_eq!(books.filename_template.as_deref(), Some("{title}.md"));

    match books.properties.get("title").unwrap() {
        PropertyConstraint::String {
            pattern,
            min_length,
            max_length,
        } => {
            assert_eq!(pattern.as_deref(), Some("^[A-Z]"));
            assert_eq!(*min_length, Some(3));
            assert_eq!(*max_length, Some(80));
        }
        other => panic!("expected a string constraint, got {other:?}"),
    }
    match books.properties.get("pages").unwrap() {
        PropertyConstraint::Number { minimum, maximum } => {
            assert_eq!(*minimum, Some(1.0));
            assert_eq!(*maximum, Some(2000.0));
        }
        other => panic!("expected a number constraint, got {other:?}"),
    }
    match books.properties.get("genre").unwrap() {
        PropertyConstraint::Enum { values } => {
            assert_eq!(values, &["fiction".to_string(), "nonfiction".to_string()]);
        }
        other => panic!("expected an enum constraint, got {other:?}"),
    }
    match books.properties.get("contributors").unwrap() {
        PropertyConstraint::ObjectList {
            required_keys,
            allowed_keys,
            ..
        } => {
            assert_eq!(required_keys, &["name".to_string()]);
            assert_eq!(
                allowed_keys.as_ref().unwrap(),
                &["name".to_string(), "role".to_string()]
            );
        }
        other => panic!("expected an object-list constraint, got {other:?}"),
    }
}
