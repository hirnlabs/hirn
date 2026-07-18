use std::fs;
use std::process::{self, Command, Stdio};
use crate::extract::ensure_goose_extracted;
use crate::rebrand::rebrand_tty_text;

fn process_stream<R: std::io::Read, W: std::io::Write>(reader: R, mut writer: W) {
    let mut reader = std::io::BufReader::new(reader);
    let mut carry = String::new();
    let mut buffer = [0u8; 1024];
    loop {
        match std::io::Read::read(&mut reader, &mut buffer) {
            Ok(0) => {
                if !carry.is_empty() {
                    let rebranded = rebrand_tty_text(&carry);
                    let _ = writer.write_all(rebranded.as_bytes());
                    let _ = writer.flush();
                }
                break;
            }
            Ok(n) => {
                let chunk_str = String::from_utf8_lossy(&buffer[..n]);
                let combined = format!("{}{}", carry, chunk_str);
                
                let mut split_idx = combined.len();
                let mut found = None;
                for (idx, c) in combined.char_indices().rev().take(30) {
                    if c.is_whitespace() || c == ')' || c == '>' || c == '\'' || c == '-' || c == '.' {
                        found = Some((idx, c));
                        break;
                    }
                }
                if let Some((idx, c)) = found {
                    split_idx = idx + c.len_utf8();
                }
                
                let to_process = &combined[..split_idx];
                carry = combined[split_idx..].to_string();
                
                let rebranded = rebrand_tty_text(to_process);
                let _ = writer.write_all(rebranded.as_bytes());
                let _ = writer.flush();
            }
            Err(_) => break,
        }
    }
}

pub fn run_goose_wrapper_with_args(mut pass_args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let goose_path = ensure_goose_extracted()?;
    let home = dirs::home_dir().ok_or("Could not find user home directory")?;
    let config_dir = home.join(".hirn").join("config");

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
        process_stream(child_stdout, std::io::stdout());
    });

    // Spawn thread to process stderr
    std::thread::spawn(move || {
        process_stream(child_stderr, std::io::stderr());
    });

    let status = child.wait()?;
    if let Some(code) = status.code() {
        process::exit(code);
    }

    Ok(())
}
