use hirn_proxy::caller::OutputMode;
use hirn_proxy::cli::Commands;
use hirn_proxy::commands::dispatch;

// JSON verdicts parse and carry command + status + payload.
#[test]
fn run_json_embeds_structured_args() {
    let out = dispatch(
        &Commands::Run {
            tool: "my-tool".into(),
            args: r#"{"a":1}"#.into(),
        },
        OutputMode::Json,
    );
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["command"], "run");
    assert_eq!(value["status"], "stub");
    assert_eq!(value["tool"], "my-tool");
    assert_eq!(value["args"]["a"], 1);
}

#[test]
fn run_json_keeps_unparsable_args_as_string() {
    let out = dispatch(
        &Commands::Run {
            tool: "my-tool".into(),
            args: "not-json".into(),
        },
        OutputMode::Json,
    );
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["args"], "not-json");
}

// Pretty verdicts are human-readable single lines.
#[test]
fn list_pretty_names_the_stub() {
    let out = dispatch(&Commands::List, OutputMode::Pretty);
    assert!(out.contains("list"));
    assert!(out.contains("stub"));
}

#[test]
fn pretty_run_shows_tool_and_args() {
    let out = dispatch(
        &Commands::Run {
            tool: "my-tool".into(),
            args: r#"{"a":1}"#.into(),
        },
        OutputMode::Pretty,
    );
    assert!(out.contains("my-tool"));
    assert!(out.contains(r#"{"a":1}"#));
}

// Every verb dispatches without dropping its payload.
//
// `Add` hits the network, so it is covered in `tests/add.rs` with a fake
// env — here it only asserts the failure verdict still names the source.
#[test]
fn each_verb_mentions_its_subject() {
    let cases = [
        (Commands::Skill { name: "code-review".into() }, "code-review"),
        (
            Commands::Add {
                source: "".into(),
                local: false,
                only: None,
            },
            "add",
        ),
        (Commands::Activate { name: "my-tool".into() }, "my-tool"),
        (Commands::Deactivate { name: "my-tool".into() }, "my-tool"),
    ];
    for (command, needle) in cases {
        for mode in [OutputMode::Json, OutputMode::Pretty] {
            assert!(dispatch(&command, mode).contains(needle), "{command:?} @ {mode:?}");
        }
    }
}
