use serde_json::{Value, json};

use crate::caller::OutputMode;

/// Render a single-field stub verdict in the caller's mode.
pub fn single_stub(command: &str, detail_key: &str, detail_value: &str, mode: OutputMode) -> String {
    match mode {
        OutputMode::Json => json!({
            "command": command,
            "status": "stub",
            detail_key: detail_value,
        })
        .to_string(),
        OutputMode::Pretty => format!("{command} (stub): {detail_key}={detail_value}"),
    }
}

/// Render the `list` stub verdict.
pub fn list_stub(mode: OutputMode) -> String {
    match mode {
        OutputMode::Json => r#"{"command":"list","status":"stub","tools":[]}"#.to_string(),
        OutputMode::Pretty => "list (stub): no tools".to_string(),
    }
}

/// Render the `run` stub verdict.
///
/// `args` arrives as a raw JSON string; when it parses, the JSON value is
/// embedded so agents see structured args, otherwise the raw string is kept.
pub fn run_stub(tool: &str, args: &str, mode: OutputMode) -> String {
    match mode {
        OutputMode::Json => {
            let parsed: Value = serde_json::from_str(args).unwrap_or(Value::String(args.to_string()));
            json!({
                "command": "run",
                "status": "stub",
                "tool": tool,
                "args": parsed,
            })
            .to_string()
        }
        OutputMode::Pretty => format!("run (stub): tool={tool}\nargs: {args}"),
    }
}
