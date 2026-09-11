use std::path::PathBuf;

use hirn_proxy::caller::OutputMode;
use hirn_proxy::commands::add::{AddEnv, run_with};
use hirn_proxy::skills::install::InstallError;
use hirn_proxy::skills::source::SkillSource;

struct FakeEnv {
    result: Result<(Vec<String>, PathBuf), String>,
}

impl AddEnv for FakeEnv {
    fn install(
        &self,
        _source: &SkillSource,
        _only: Option<&str>,
        _local: bool,
    ) -> Result<(Vec<String>, PathBuf), InstallError> {
        match &self.result {
            Ok((names, index)) => Ok((names.clone(), index.clone())),
            Err(msg) => Err(InstallError::Io {
                path: PathBuf::from("fake"),
                source: std::io::Error::other(msg.clone()),
            }),
        }
    }
}

// Success verdict carries installed names, active flag, and index path.
#[test]
fn success_json_lists_installed_skills_as_active() {
    let env = FakeEnv {
        result: Ok((
            vec!["tdd".to_string(), "grill-me".to_string()],
            PathBuf::from("/home/u/.hirn/tools.json"),
        )),
    };
    let out = run_with("mattpocock/skills", false, None, OutputMode::Json, &env);
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["command"], "add");
    assert_eq!(value["status"], "ok");
    assert_eq!(value["active"], true);
    assert_eq!(value["installed"], serde_json::json!(["tdd", "grill-me"]));
    assert!(value["index"].as_str().unwrap().ends_with("tools.json"));
}

#[test]
fn success_pretty_names_each_skill() {
    let env = FakeEnv {
        result: Ok((vec!["tdd".to_string()], PathBuf::from("/home/u/.hirn/tools.json"))),
    };
    let out = run_with("mattpocock/skills", false, Some("tdd"), OutputMode::Pretty, &env);
    assert!(out.contains("tdd"));
    assert!(out.contains("active"));
    assert!(out.contains("tools.json"));
}

// Failure verdicts name the source and the cause, in both modes.
#[test]
fn bad_source_short_circuits_before_install() {
    struct ExplodingEnv;
    impl AddEnv for ExplodingEnv {
        fn install(
            &self,
            _s: &SkillSource,
            _o: Option<&str>,
            _l: bool,
        ) -> Result<(Vec<String>, PathBuf), InstallError> {
            panic!("must not reach install on bad source");
        }
    }
    for mode in [OutputMode::Json, OutputMode::Pretty] {
        let out = run_with("just-a-word", false, None, mode, &ExplodingEnv);
        assert!(out.contains("just-a-word"), "{mode:?}: {out}");
    }
}

#[test]
fn install_failure_renders_error_verdict() {
    let env = FakeEnv {
        result: Err("connection refused".to_string()),
    };
    let out = run_with("mattpocock/skills", false, None, OutputMode::Json, &env);
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["status"], "error");
    assert_eq!(value["source"], "mattpocock/skills");
    assert!(value["error"].as_str().unwrap().contains("connection refused"));

    let out = run_with("mattpocock/skills", false, None, OutputMode::Pretty, &env);
    assert!(out.contains("connection refused"));
}
