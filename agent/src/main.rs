use clap::Parser;
use std::env;
use std::fs;
use std::process::{self, Command, Stdio};

// Embed the raw ZIP/Tarball archive of Pi
const PI_ARCHIVE: &[u8] = include_bytes!(env!("PI_ARCHIVE_PATH"));

#[derive(Parser, Debug)]
#[command(name = "hirn")]
#[command(about = "Hirn Agent CLI and WebRTC ACP Relay Server", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Start the WebRTC signaling & ACP relay server
    Serve {
        /// Port to run the signaling HTTP/WS server on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Intercept serve subcommand or standard clap helper flags
    if args.len() > 1 && args[1] == "serve" {
        let cli = Cli::parse();
        match cli.command {
            Commands::Serve { port } => {
                println!("Starting Hirn WebRTC ACP Relay Server on port {}...", port);
                // The actual server execution logic will be implemented in later tickets.
                // For now, we block or simulate the serve process.
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
        println!("  serve                          Start the WebRTC signaling & ACP relay server");
        println!("                                 (Run 'hirn serve --help' for serve options)\n");
        println!("Hirn CLI Options & Usage:");
        println!("--------------------------------------------------------------------------------");

        // Dynamically get help from the extracted pi binary
        if let Ok(pi_path) = ensure_pi_extracted() {
            let home = dirs::home_dir().unwrap();
            let hirn_dir = home.join(".hirn");

            let output = Command::new(&pi_path)
                .arg("--help")
                .env("PI_CODING_AGENT_DIR", hirn_dir.join("agent"))
                .env("PI_CODING_AGENT_SESSION_DIR", hirn_dir.join("agent").join("sessions"))
                .output();

            if let Ok(out) = output {
                let raw_help = String::from_utf8_lossy(&out.stdout);
                let rebranded = rebrand_text(&raw_help);
                print!("{}", rebranded);
            }
        } else {
            println!("(Error: Could not extract bundled Pi CLI to load help menu)");
        }
    } else if args.len() > 1 && (args[1] == "-V" || args[1] == "--version") {
        println!("hirn 0.1.0");
        
        // Dynamically get version from the extracted pi binary
        if let Ok(pi_path) = ensure_pi_extracted() {
            let home = dirs::home_dir().unwrap();
            let hirn_dir = home.join(".hirn");

            let output = Command::new(&pi_path)
                .arg("--version")
                .env("PI_CODING_AGENT_DIR", hirn_dir.join("agent"))
                .env("PI_CODING_AGENT_SESSION_DIR", hirn_dir.join("agent").join("sessions"))
                .output();

            if let Ok(out) = output {
                let raw_ver = String::from_utf8_lossy(&out.stdout);
                print!("pi {}", raw_ver);
            }
        }
    } else {
        // Wrap and execute the bundled 'pi' CLI
        if let Err(e) = run_pi_wrapper() {
            eprintln!("Hirn Agent: Failed to wrap Pi CLI: {}", e);
            process::exit(1);
        }
    }
}

fn rebrand_text(text: &str) -> String {
    text.replace(" pi ", " hirn ")
        .replace(" pi\n", " hirn\n")
        .replace(" pi\r", " hirn\r")
        .replace(" pi\"", " hirn\"")
        .replace(" pi`", " hirn`")
        .replace("(default: ~/.pi/agent)", "(default: ~/.hirn/agent)")
        .replace("~/.pi/", "~/.hirn/")
        .replace(" PI_", " HIRN_")
        .replace("\nPI_", "\nHIRN_")
        .replace("\r\nPI_", "\r\nHIRN_")
        .replace("Pi ", "Hirn ")
        .replace("Pi:", "Hirn:")
}

fn ensure_pi_extracted() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // 1. Get user home directory and define ~/.hirn/bin/
    let home = dirs::home_dir().ok_or("Could not find user home directory")?;
    let hirn_dir = home.join(".hirn");
    let bin_dir = hirn_dir.join("bin");
    fs::create_dir_all(&bin_dir)?;

    // 2. Define pi executable path
    let pi_name = if cfg!(windows) { "pi.exe" } else { "pi" };
    let pi_path = bin_dir.join(pi_name);

    // 3. Extract the embedded Pi archive if not present
    if !pi_path.exists() {
        let is_windows = cfg!(windows);
        let archive_name = if is_windows { "pi_archive.tmp.zip" } else { "pi_archive.tmp.tar.gz" };
        let archive_path = bin_dir.join(archive_name);

        // Write the embedded archive bytes to a temp file
        fs::write(&archive_path, PI_ARCHIVE)?;

        // Execute extraction command
        if is_windows {
            let status = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(
                        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                        archive_path.display(),
                        bin_dir.display()
                    ),
                ])
                .status()?;
            if !status.success() {
                return Err("Failed to extract Pi ZIP archive using powershell".into());
            }
        } else {
            let status = Command::new("tar")
                .args(&[
                    "-xzf",
                    archive_path.to_str().unwrap(),
                    "-C",
                    bin_dir.to_str().unwrap(),
                ])
                .status()?;
            if !status.success() {
                return Err("Failed to extract Pi Tarball archive using tar".into());
            }
        }

        // Clean up the temp archive file
        let _ = fs::remove_file(archive_path)?;
    }

    Ok(pi_path)
}

fn run_pi_wrapper() -> Result<(), Box<dyn std::error::Error>> {
    let pi_path = ensure_pi_extracted()?;
    let home = dirs::home_dir().ok_or("Could not find user home directory")?;
    let hirn_dir = home.join(".hirn");

    let pass_args: Vec<String> = env::args().skip(1).collect();

    let mut cmd = Command::new(&pi_path);
    cmd.args(&pass_args);

    // Forward HIRN_ env variables to child PI_ env variables
    let env_vars = [
        ("HIRN_CODING_AGENT_DIR", "PI_CODING_AGENT_DIR"),
        ("HIRN_CODING_AGENT_SESSION_DIR", "PI_CODING_AGENT_SESSION_DIR"),
        ("HIRN_PACKAGE_DIR", "PI_PACKAGE_DIR"),
        ("HIRN_OFFLINE", "PI_OFFLINE"),
        ("HIRN_TELEMETRY", "PI_TELEMETRY"),
        ("HIRN_SHARE_VIEWER_URL", "PI_SHARE_VIEWER_URL"),
    ];

    for (hirn_var, pi_var) in env_vars.iter() {
        if let Ok(val) = env::var(hirn_var) {
            cmd.env(pi_var, val);
        }
    }

    // Set default isolated directories if not explicitly overridden by user
    if env::var("HIRN_CODING_AGENT_DIR").is_err() && env::var("PI_CODING_AGENT_DIR").is_err() {
        cmd.env("PI_CODING_AGENT_DIR", hirn_dir.join("agent"));
    }
    if env::var("HIRN_CODING_AGENT_SESSION_DIR").is_err() && env::var("PI_CODING_AGENT_SESSION_DIR").is_err() {
        cmd.env("PI_CODING_AGENT_SESSION_DIR", hirn_dir.join("agent").join("sessions"));
    }

    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    if let Some(code) = status.code() {
        process::exit(code);
    }

    Ok(())
}
