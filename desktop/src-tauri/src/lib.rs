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
    let mut cmd_to_run = command.clone();
    let final_args = args.clone();

    if command == "hirn" || command == "hirn.exe" {
        let local_debug_bin = std::path::Path::new("agent/target/debug/hirn.exe");
        let local_debug_bin_parent = std::path::Path::new("../agent/target/debug/hirn.exe");
        if local_debug_bin.exists() {
            cmd_to_run = local_debug_bin.to_string_lossy().to_string();
        } else if local_debug_bin_parent.exists() {
            cmd_to_run = local_debug_bin_parent.to_string_lossy().to_string();
        }
    }

    let mut child = Command::new(&cmd_to_run)
        .args(&final_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ACP binary '{}': {}", cmd_to_run, e))?;

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

fn ensure_extension(name: &str, default_ext: &str) -> String {
    if name.ends_with(".yaml") || name.ends_with(".json") {
        name.to_string()
    } else if default_ext.starts_with('.') {
        format!("{}{}", name, default_ext)
    } else {
        format!("{}.{}", name, default_ext)
    }
}

#[tauri::command]
fn save_named_config(name: String, content: String) -> Result<(), String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let config_dir = std::path::Path::new(&home).join(".hirn").join("config");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create directory ~/.hirn/config: {}", e))?;

    let filename = ensure_extension(&name, ".yaml");

    let file_path = config_dir.join(filename);
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write config file '{:?}': {}", file_path, e))?;

    Ok(())
}

#[tauri::command]
fn load_named_config(name: String) -> Result<String, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let config_dir = std::path::Path::new(&home).join(".hirn").join("config");

    let possible_names = if name.ends_with(".yaml") || name.ends_with(".json") {
        vec![name.clone()]
    } else {
        vec![ensure_extension(&name, ".yaml"), ensure_extension(&name, ".json")]
    };

    for fname in possible_names {
        let file_path = config_dir.join(&fname);
        if file_path.exists() {
            return std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read config file '{:?}': {}", file_path, e));
        }
    }

    Err("File not found".to_string())
}

#[tauri::command]
fn save_agent_config(agent_id: String, content: String) -> Result<(), String> {
    save_named_config(format!("{}.yaml", agent_id), content)
}

#[tauri::command]
fn load_agent_config(agent_id: String) -> Result<String, String> {
    load_named_config(format!("{}.yaml", agent_id))
}

#[tauri::command]
fn save_session_file(id: String, content: String) -> Result<(), String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let sessions_dir = std::path::Path::new(&home).join(".hirn").join("sessions");
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create directory ~/.hirn/sessions: {}", e))?;

    let filename = ensure_extension(&id, ".json");

    let file_path = sessions_dir.join(filename);
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write session file '{:?}': {}", file_path, e))?;

    Ok(())
}

#[tauri::command]
fn load_session_file(id: String) -> Result<String, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let sessions_dir = std::path::Path::new(&home).join(".hirn").join("sessions");
    let possible_names = if id.ends_with(".yaml") || id.ends_with(".json") {
        vec![id.clone()]
    } else {
        vec![ensure_extension(&id, ".json"), ensure_extension(&id, ".yaml")]
    };

    for fname in possible_names {
        let file_path = sessions_dir.join(&fname);
        if file_path.exists() {
            return std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read session file '{:?}': {}", file_path, e));
        }
    }

    Err("Session file not found".to_string())
}

#[tauri::command]
fn list_session_files() -> Result<Vec<String>, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let sessions_dir = std::path::Path::new(&home).join(".hirn").join("sessions");
    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut contents = Vec::new();
    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read directory ~/.hirn/sessions: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read session file '{:?}': {}", path, e))?;
            contents.push(text);
        }
    }

    Ok(contents)
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
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            spawn_acp_agent,
            send_acp_stdin,
            save_agent_config,
            load_agent_config,
            save_named_config,
            load_named_config,
            save_session_file,
            load_session_file,
            list_session_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


