use hirn_proxy::skills::index;
use hirn_proxy::skills::source::parse_source;

// Round-trip: save → load preserves entries and unknown top-level keys.
#[test]
fn index_round_trips_through_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tools.json");
    let mut index = index::SkillsIndex::default();
    index::upsert(
        &mut index,
        "tdd".to_string(),
        index::SkillEntry {
            source: "mattpocock/skills".to_string(),
            source_type: "github".to_string(),
            skill_path: "skills/engineering/tdd/SKILL.md".to_string(),
            active: true,
        },
    );
    index::save(&path, &index).unwrap();
    let loaded = index::load(&path).unwrap();
    assert_eq!(loaded.skills["tdd"].skill_path, "skills/engineering/tdd/SKILL.md");
    assert!(loaded.skills["tdd"].active);
}

// New skills arrive active; re-add keeps the user's deactivation.
#[test]
fn upsert_defaults_active_but_preserves_deactivation() {
    let mut index = index::SkillsIndex::default();
    let entry = || index::SkillEntry {
        source: "mattpocock/skills".to_string(),
        source_type: "github".to_string(),
        skill_path: "skills/engineering/tdd/SKILL.md".to_string(),
        active: true,
    };
    assert!(index::upsert(&mut index, "tdd".to_string(), entry()));
    index.skills.get_mut("tdd").unwrap().active = false;
    assert!(!index::upsert(&mut index, "tdd".to_string(), entry()));
    assert!(!index.skills["tdd"].active);
}

// Missing file is an empty index; corrupt file is an error, not a panic.
#[test]
fn load_tolerates_missing_but_rejects_corrupt() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope.json");
    assert!(index::load(&missing).unwrap().skills.is_empty());

    let corrupt = dir.path().join("tools.json");
    std::fs::write(&corrupt, "{oops").unwrap();
    assert!(matches!(index::load(&corrupt), Err(index::IndexError::Corrupt { .. })));
}

// Source shapes: shorthand, nested path, tree URL, and rejections.
#[test]
fn parses_shorthand_and_nested_path() {
    let s = parse_source("mattpocock/skills").unwrap();
    assert_eq!(s.owner, "mattpocock");
    assert_eq!(s.repo, "skills");
    assert_eq!(s.path, None);

    let s = parse_source("mattpocock/skills/skills/engineering/tdd").unwrap();
    assert_eq!(s.path.as_deref(), Some("skills/engineering/tdd"));
}

#[test]
fn parses_github_tree_url() {
    let s = parse_source("https://github.com/mattpocock/skills/tree/main/skills/engineering/tdd").unwrap();
    assert_eq!(s.owner, "mattpocock");
    assert_eq!(s.repo, "skills");
    assert_eq!(s.path.as_deref(), Some("skills/engineering/tdd"));
}

#[test]
fn rejects_bare_word_and_foreign_host() {
    assert!(parse_source("just-a-word").is_err());
    assert!(parse_source("https://gitlab.com/owner/repo").is_err());
    assert!(parse_source("").is_err());
}
