use serde_json::json;

use crate::caller::OutputMode;
use crate::skills::{install as install_mod, source as source_mod, source::SkillSource};

/// Environment seam for `add`: network + filesystem + home dir.
///
/// `dispatch` passes [`RealAddEnv`]; tests inject a fake. The seam is the
/// whole side-effect boundary — decision logic in `run_with` stays pure.
pub trait AddEnv {
    fn install(
        &self,
        source: &SkillSource,
        only: Option<&str>,
        local: bool,
    ) -> Result<(Vec<String>, std::path::PathBuf), install_mod::InstallError>;
}

/// Production environment: real GitHub client, cwd, and home dir.
pub struct RealAddEnv;

impl AddEnv for RealAddEnv {
    fn install(
        &self,
        source: &SkillSource,
        only: Option<&str>,
        local: bool,
    ) -> Result<(Vec<String>, std::path::PathBuf), install_mod::InstallError> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let client = install_mod::GitHubClient::new();
        install_mod::install(&client, source, only, local, dirs::home_dir(), &cwd)
    }
}

/// Fetch skills from a GitHub-like source and install them active.
///
/// Thin wrapper over [`run_with`] with the real environment.
pub fn run(source: &str, local: bool, only: Option<&str>, mode: OutputMode, env: &impl AddEnv) -> String {
    run_with(source, local, only, mode, env)
}

/// Decision logic: parse → install → render. All I/O via `env`.
pub fn run_with(
    source: &str,
    local: bool,
    only: Option<&str>,
    mode: OutputMode,
    env: &impl AddEnv,
) -> String {
    let parsed = match source_mod::parse_source(source) {
        Ok(parsed) => parsed,
        Err(e) => return error_string(source, &e.to_string(), mode),
    };
    match env.install(&parsed, only, local) {
        Ok((installed, index_path)) => success_string(&parsed.to_string(), &installed, &index_path, mode),
        Err(e) => error_string(&parsed.to_string(), &e.to_string(), mode),
    }
}

fn success_string(
    source: &str,
    installed: &[String],
    index_path: &std::path::Path,
    mode: OutputMode,
) -> String {
    match mode {
        OutputMode::Json => json!({
            "command": "add",
            "status": "ok",
            "source": source,
            "installed": installed,
            "active": true,
            "index": index_path.display().to_string(),
        })
        .to_string(),
        OutputMode::Pretty => {
            let mut out = format!("added {} skill(s) from {source} (active):\n", installed.len());
            for name in installed {
                out.push_str(&format!("  + {name}\n"));
            }
            out.push_str(&format!("index: {}", index_path.display()));
            out
        }
    }
}

fn error_string(source: &str, message: &str, mode: OutputMode) -> String {
    match mode {
        OutputMode::Json => json!({
            "command": "add",
            "status": "error",
            "source": source,
            "error": message,
        })
        .to_string(),
        OutputMode::Pretty => format!("add failed for {source}: {message}"),
    }
}
