use clap::Parser;
use hirn_proxy::cli::{Cli, Commands};

// CLI shape: each README verb parses into its variant with args intact.
#[test]
fn parses_list() {
    let cli = Cli::try_parse_from(["hirn-proxy", "list"]).unwrap();
    assert!(matches!(cli.command, Commands::List));
}

#[test]
fn parses_run_with_tool_and_args() {
    let cli = Cli::try_parse_from(["hirn-proxy", "run", "my-tool", r#"{"a":1}"#]).unwrap();
    match cli.command {
        Commands::Run { tool, args } => {
            assert_eq!(tool, "my-tool");
            assert_eq!(args, r#"{"a":1}"#);
        }
        _ => panic!("expected Run"),
    }
}

#[test]
fn parses_skill_add_and_activation_verbs() {
    let cli = Cli::try_parse_from(["hirn-proxy", "skill", "code-review"]).unwrap();
    assert!(matches!(cli.command, Commands::Skill { .. }));

    let cli = Cli::try_parse_from(["hirn-proxy", "add", "owner/repo"]).unwrap();
    assert!(matches!(
        cli.command,
        Commands::Add { local: false, only: None, .. }
    ));

    let cli = Cli::try_parse_from(["hirn-proxy", "add", "--local", "--only", "tdd", "owner/repo"]).unwrap();
    match cli.command {
        Commands::Add { local, only, .. } => {
            assert!(local);
            assert_eq!(only.as_deref(), Some("tdd"));
        }
        _ => panic!("expected Add"),
    }

    let cli = Cli::try_parse_from(["hirn-proxy", "activate", "my-tool"]).unwrap();
    assert!(matches!(cli.command, Commands::Activate { .. }));

    let cli = Cli::try_parse_from(["hirn-proxy", "deactivate", "my-tool"]).unwrap();
    assert!(matches!(cli.command, Commands::Deactivate { .. }));
}

#[test]
fn global_flags_survive_subcommand_parsing() {
    let cli = Cli::try_parse_from(["hirn-proxy", "--json", "list"]).unwrap();
    assert!(cli.json);
    assert!(!cli.interactive);

    let cli = Cli::try_parse_from(["hirn-proxy", "--interactive", "list"]).unwrap();
    assert!(cli.interactive);
    assert!(!cli.json);
}
