use clap::Parser;
use std::env;
use std::fs;
use std::process::{self, Command};

mod rebrand;
mod extract;
mod wrapper;

use rebrand::rebrand_text;
use extract::ensure_goose_extracted;
use wrapper::run_goose_wrapper_with_args;

#[derive(Parser, Debug)]
#[command(name = "hirn")]
#[command(about = "Hirn Agent CLI and WebRTC ACP Relay Server", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Start the WebRTC ACP relay server
    Relay {
        /// Port to run the WebRTC ACP relay server on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Intercept relay subcommand
    if args.len() > 1 && args[1] == "relay" {
        let cli = Cli::parse();
        match cli.command {
            Commands::Relay { port } => {
                println!("Starting Hirn WebRTC ACP Relay Server on port {}...", port);
                // The actual server execution logic will be implemented in later tickets.
                // For now, we block or simulate the relay process.
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(60));
                }
            }
        }
    } else if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        // Print Agent help header
        println!("Hirn Agent CLI and WebRTC ACP Relay Server\n");
        println!("Usage: hirn [COMMAND] [OPTIONS]\n");
        println!("Hirn Agent Commands:");
        println!("  relay                          Start the WebRTC ACP relay server");
        println!("                                 (Run 'hirn relay --help' for options)\n");
        println!("Hirn CLI Options & Usage (forwarded to Goose):");
        println!("--------------------------------------------------------------------------------");

        let home = dirs::home_dir().unwrap();
        let config_dir = home.join(".config").join("hirn");
        let bin_dir = home.join(".hirn").join("bin");
        let help_cache_path = bin_dir.join(".help_cache");

        if help_cache_path.exists() {
            if let Ok(cached_help) = fs::read_to_string(&help_cache_path) {
                print!("{}", cached_help);
                return;
            }
        }

        // Dynamically get help from the extracted goose binary
        if let Ok(goose_path) = ensure_goose_extracted() {
            let output = Command::new(&goose_path)
                .arg("--help")
                .env("GOOSE_CONFIG_DIR", &config_dir)
                .output();

            if let Ok(out) = output {
                let raw_help = String::from_utf8_lossy(&out.stdout);
                let rebranded = rebrand_text(&raw_help);
                print!("{}", rebranded);
                if let Err(e) = fs::write(&help_cache_path, &rebranded) {
                    eprintln!("Error writing help cache: {:?}", e);
                }
            }
        } else {
            println!("(Error: Could not extract bundled Goose CLI to load help menu)");
        }
    } else if args.len() > 1 && (args[1] == "-V" || args[1] == "--version") {
        println!("hirn 0.1.0");

        let home = dirs::home_dir().unwrap();
        let config_dir = home.join(".config").join("hirn");
        let bin_dir = home.join(".hirn").join("bin");
        let version_cache_path = bin_dir.join(".version_cache");

        if version_cache_path.exists() {
            if let Ok(cached_ver) = fs::read_to_string(&version_cache_path) {
                print!("{}", cached_ver);
                return;
            }
        }
        
        // Dynamically get version from the extracted goose binary
        if let Ok(goose_path) = ensure_goose_extracted() {
            let output = Command::new(&goose_path)
                .arg("--version")
                .env("GOOSE_CONFIG_DIR", &config_dir)
                .output();

            if let Ok(out) = output {
                let raw_ver = String::from_utf8_lossy(&out.stdout);
                let rebranded_ver = rebrand_text(&raw_ver);
                print!("{}", rebranded_ver);
                if let Err(e) = fs::write(&version_cache_path, &rebranded_ver) {
                    eprintln!("Error writing version cache: {:?}", e);
                }
            }
        }
    } else {
        // Forward arguments to Goose wrapper
        let pass_args: Vec<String> = env::args().skip(1).collect();

        if let Err(e) = run_goose_wrapper_with_args(pass_args) {
            eprintln!("Hirn Agent: Failed to wrap Goose CLI: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rebrand::rebrand_tty_text;

    #[test]
    fn test_rebrand_text() {
        assert_eq!(rebrand_text("Usage: goose.exe [COMMAND]"), "Usage: hirn [COMMAND]");
        assert_eq!(rebrand_text("Run goose configure"), "Run hirn configure");
    }

    #[test]
    fn test_rebrand_tty_text() {
        assert_eq!(rebrand_tty_text("    __( O)>"), "    .--@~^~@-.");
        assert_eq!(rebrand_tty_text("   \\____)"), "   ( @)(@ )(@ )");
        assert_eq!(rebrand_tty_text("     L L"), "    `-.@_@.-'");
    }

    #[test]
    fn test_cli_parse_relay() {
        let args = vec!["hirn", "relay"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(matches!(cli.command, Commands::Relay { port: 8080 }));
    }
}
