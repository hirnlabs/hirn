use std::fs;
use std::io::Write;
use std::process::{self, Command, Stdio};
use crate::extract::ensure_goose_extracted;
use crate::rebrand::rebrand_tty_text;

pub fn run_goose_wrapper_with_args(mut pass_args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let goose_path = ensure_goose_extracted()?;
    let home = dirs::home_dir().ok_or("Could not find user home directory")?;
    let config_dir = home.join(".config").join("hirn");

    // If no arguments were passed, default to running goose in ACP stdio mode
    if pass_args.is_empty() {
        pass_args.push("acp".to_string());
        pass_args.push("--with-builtin".to_string());
        pass_args.push("developer".to_string());
    }

    let mut cmd = Command::new(&goose_path);
    cmd.args(&pass_args);

    cmd.env("GOOSE_CONFIG_DIR", &config_dir);

    // Decouple temp directory to prevent global EPERM conflicts
    let temp_dir = home.join(".hirn").join("agent").join("temp");
    let _ = fs::create_dir_all(&temp_dir);
    cmd.env("TEMP", &temp_dir);
    cmd.env("TMP", &temp_dir);
    cmd.env("TMPDIR", &temp_dir);

    let mut child = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    // Spawn thread to process stdout
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(child_stdout);
        let mut buffer = [0u8; 4096];
        let mut out = std::io::stdout();
        loop {
            match std::io::Read::read(&mut reader, &mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buffer[..n];
                    let text = String::from_utf8_lossy(chunk);
                    let rebranded = rebrand_tty_text(&text);
                    let _ = out.write_all(rebranded.as_bytes());
                    let _ = out.flush();
                }
                Err(_) => break,
            }
        }
    });

    // Spawn thread to process stderr
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(child_stderr);
        let mut buffer = [0u8; 4096];
        let mut err = std::io::stderr();
        loop {
            match std::io::Read::read(&mut reader, &mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = &buffer[..n];
                    let text = String::from_utf8_lossy(chunk);
                    let rebranded = rebrand_tty_text(&text);
                    let _ = err.write_all(rebranded.as_bytes());
                    let _ = err.flush();
                }
                Err(_) => break,
            }
        }
    });

    let status = child.wait()?;
    if let Some(code) = status.code() {
        process::exit(code);
    }

    Ok(())
}
