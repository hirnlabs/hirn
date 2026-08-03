use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::io::{BufRead, BufReader, Write};
use tauri::{AppHandle, Emitter, State};
use std::collections::HashMap;

struct ProcessState {
    children: Mutex<HashMap<u32, std::process::ChildStdin>>,
}

#[tauri::command]
fn spawn_acp_agent(app: AppHandle, state: State<'_, ProcessState>, command: String, args: Vec<String>) -> Result<u32, String> {
    let mut child = Command::new(&command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ACP binary '{}': {}", command, e))?;

    let pid = child.id();
    let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to open stderr")?;

    state.children.lock().unwrap().insert(pid, stdin);

    let app_handle_out = app.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line_text) = line {
                let _ = app_handle_out.emit("acp-stdout", (pid, line_text));
            }
        }
    });

    let app_handle_err = app.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line_text) = line {
                let _ = app_handle_err.emit("acp-stderr", (pid, line_text));
            }
        }
    });

    Ok(pid)
}

#[tauri::command]
fn send_acp_stdin(state: State<'_, ProcessState>, pid: u32, message: String) -> Result<(), String> {
    let mut lock = state.children.lock().unwrap();
    if let Some(stdin) = lock.get_mut(&pid) {
        writeln!(stdin, "{}", message).map_err(|e| e.to_string())?;
        stdin.flush().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Process {} not found", pid))
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProcessState {
            children: Mutex::new(HashMap::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, spawn_acp_agent, send_acp_stdin])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
