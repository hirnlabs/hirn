use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::source::SkillSource;

/// Where a skill lands on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    /// `~/.hirn` or `<project>/.agents`, depending on `--local`.
    pub root: PathBuf,
    /// `<root>/skills/<name>/`.
    pub skill_dir: PathBuf,
    /// `<root>/tools.json`.
    pub index_path: PathBuf,
}

/// Resolve global (`~/.hirn`) vs local (`<cwd>/.agents`) install paths.
pub fn install_target(
    name: &str,
    local: bool,
    home_dir: Option<PathBuf>,
    cwd: &Path,
) -> Result<InstallTarget, InstallError> {
    let root = if local {
        cwd.join(".agents")
    } else {
        let home = home_dir.ok_or(InstallError::NoHomeDir)?;
        home.join(".hirn")
    };
    Ok(InstallTarget {
        skill_dir: root.join("skills").join(name),
        index_path: root.join("tools.json"),
        root,
    })
}

/// A fetched file: repo-relative path plus raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedFile {
    /// Path relative to the skill directory (e.g. `SKILL.md`).
    pub rel_path: String,
    /// Raw file bytes (kept verbatim — markdown, yml, images).
    pub bytes: Vec<u8>,
}

/// A skill directory discovered inside the fetched tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSkill {
    /// Skill name from `SKILL.md` frontmatter (`name:`).
    pub name: String,
    /// Short description from frontmatter (`description:`).
    pub description: String,
    /// Repo-relative path to the skill dir (e.g. `skills/engineering/tdd`).
    pub repo_dir: String,
    /// Repo-relative path to `SKILL.md` (for the index entry).
    pub skill_path: String,
    /// All files under the skill dir, relative to it.
    pub files: Vec<FetchedFile>,
}

/// What went wrong while fetching, detecting, or installing a skill.
#[derive(Debug)]
pub enum InstallError {
    /// No home directory (global installs need `~/.hirn`).
    NoHomeDir,
    /// HTTP fetch failed.
    Fetch(FetchError),
    /// No skill directory found under the source path.
    NoSkillsFound { source: SkillSource },
    /// `--only` named a skill that was not fetched.
    OnlyNotFound { only: String, found: Vec<String> },
    /// `SKILL.md` frontmatter was missing or invalid.
    InvalidFrontmatter { skill_path: String, reason: String },
    /// Filesystem failure with the path attached.
    Io { path: PathBuf, source: std::io::Error },
    /// Index read/write failure.
    Index(super::index::IndexError),
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::NoHomeDir => write!(f, "could not find home directory for ~/.hirn"),
            InstallError::Fetch(e) => write!(f, "{e}"),
            InstallError::NoSkillsFound { source } => {
                write!(f, "no SKILL.md found under `{source}`")
            }
            InstallError::OnlyNotFound { only, found } => {
                write!(f, "skill `{only}` not found (fetched: {})", found.join(", "))
            }
            InstallError::InvalidFrontmatter { skill_path, reason } => {
                write!(f, "{skill_path}: invalid SKILL.md frontmatter: {reason}")
            }
            InstallError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            InstallError::Index(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for InstallError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InstallError::Index(e) => Some(e),
            InstallError::Fetch(e) => Some(e),
            _ => None,
        }
    }
}

impl From<super::index::IndexError> for InstallError {
    fn from(e: super::index::IndexError) -> Self {
        InstallError::Index(e)
    }
}

/// What went wrong while fetching from GitHub (raw + tarball, no API).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// Non-2xx response with the URL attached.
    Http { url: String, status: u16 },
    /// Rate limited (HTTP 403/429). Raw + tarball rarely hit this; when
    /// they do the fix is waiting, not a code change.
    RateLimited { url: String },
    /// Transport failure (offline, DNS, TLS...).
    Transport(String),
    /// Response body was not the expected shape (bad tarball, bad UTF-8...).
    BadResponse { url: String, reason: String },
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Http { url, status } => write!(f, "{url}: HTTP {status}"),
            FetchError::RateLimited { url } => {
                write!(f, "{url}: rate limited — wait a few minutes and retry")
            }
            FetchError::Transport(msg) => write!(f, "network error: {msg}"),
            FetchError::BadResponse { url, reason } => write!(f, "{url}: {reason}"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Minimal `ureq`-based GitHub client (tarball + raw downloads, no API).
///
/// Discovery and file downloads go through `codeload.github.com` and
/// `raw.githubusercontent.com` — neither counts against the 60 req/hour
/// unauthenticated API quota that caused HTTP 403s on `api.github.com`.
/// The contents API is gone entirely; discovery extracts one tarball.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    agent: ureq::Agent,
    default_branch: String,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self::with_branch("main")
    }

    pub fn with_branch(branch: &str) -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .user_agent("hirn-proxy/0.1.0")
                .timeout(std::time::Duration::from_secs(60))
                .build(),
            default_branch: branch.to_string(),
        }
    }

    /// Download one raw file, trying the default branch first.
    pub fn download_raw(&self, source: &SkillSource, repo_path: &str) -> Result<Vec<u8>, FetchError> {
        let mut last = None;
        for branch in self.candidate_branches() {
            let url = raw_file_url(source, &branch, repo_path);
            match self.get_bytes(&url) {
                Ok(bytes) => return Ok(bytes),
                Err(e @ FetchError::Http { status: 404, .. }) => last = Some(e),
                Err(e) => return Err(e),
            }
        }
        Err(last.unwrap_or(FetchError::Http {
            url: raw_file_url(source, &self.default_branch, repo_path),
            status: 404,
        }))
    }

    /// Download the repo tarball, trying the default branch first.
    ///
    /// Returns the bytes plus the branch that worked.
    pub fn download_tarball(&self, source: &SkillSource) -> Result<(Vec<u8>, String), FetchError> {
        let mut last = None;
        for branch in self.candidate_branches() {
            let url = tarball_url(source, &branch);
            match self.get_bytes(&url) {
                Ok(bytes) => return Ok((bytes, branch)),
                Err(e @ FetchError::Http { status: 404, .. }) => last = Some(e),
                Err(e) => return Err(e),
            }
        }
        Err(last.unwrap_or(FetchError::Http {
            url: tarball_url(source, &self.default_branch),
            status: 404,
        }))
    }

    fn candidate_branches(&self) -> Vec<String> {
        let mut branches = vec![self.default_branch.clone()];
        for fallback in ["master", "main"] {
            if !branches.iter().any(|b| b == fallback) {
                branches.push(fallback.to_string());
            }
        }
        branches
    }

    fn get_bytes(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let response = self.agent.get(url).call().map_err(|e| match e {
            ureq::Error::Status(403, _) | ureq::Error::Status(429, _) => FetchError::RateLimited {
                url: url.to_string(),
            },
            ureq::Error::Status(status, _) => FetchError::Http {
                url: url.to_string(),
                status,
            },
            ureq::Error::Transport(t) => FetchError::Transport(t.to_string()),
        })?;
        let mut buf = Vec::new();
        response.into_reader().read_to_end(&mut buf).map_err(|e| FetchError::BadResponse {
            url: url.to_string(),
            reason: e.to_string(),
        })?;
        Ok(buf)
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

/// `https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}`.
pub fn raw_file_url(source: &SkillSource, branch: &str, repo_path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{branch}/{repo_path}",
        source.owner, source.repo
    )
}

/// `https://codeload.github.com/{owner}/{repo}/tar.gz/refs/heads/{branch}`.
pub fn tarball_url(source: &SkillSource, branch: &str) -> String {
    format!(
        "https://codeload.github.com/{}/{}/tar.gz/refs/heads/{branch}",
        source.owner, source.repo
    )
}

/// Extract the repo tarball and collect every dir holding a SKILL file.
///
/// One tarball download replaces the old per-directory contents-API walk,
/// so discovery costs zero API quota. Scoped to `source.path` when given,
/// so `owner/repo/skills/engineering/implement` only installs that subtree.
pub fn discover_skills(
    client: &GitHubClient,
    source: &SkillSource,
) -> Result<Vec<DiscoveredSkill>, InstallError> {
    let (tarball, _branch) = client.download_tarball(source).map_err(InstallError::Fetch)?;
    let mut tree = extract_tarball(&tarball, source).map_err(InstallError::Fetch)?;
    if let Some(scope) = normalize_scope(source.path.as_deref()) {
        tree.retain(|(path, _)| path == &scope || path.starts_with(&format!("{scope}/")));
    }
    // Group files by directory; a skill is a dir holding a `SKILL.md`.
    let mut by_dir: BTreeMap<String, Vec<(String, Vec<u8>)>> = BTreeMap::new();
    for (path, bytes) in tree {
        let (dir, file) = match path.rsplit_once('/') {
            Some((dir, file)) => (dir.to_string(), file.to_string()),
            None => (String::new(), path),
        };
        by_dir.entry(dir).or_default().push((file, bytes));
    }
    let mut skills = Vec::new();
    for (dir, files) in &by_dir {
        let Some(skill_md) = files.iter().find(|(name, _)| name == "SKILL.md") else {
            continue;
        };
        let (name, description) =
            parse_frontmatter(&skill_md.1).map_err(|reason| InstallError::InvalidFrontmatter {
                skill_path: join_repo_path(dir, "SKILL.md"),
                reason,
            })?;
        // Skill = its own files plus nested subtrees (agents/, references/).
        let prefix = if dir.is_empty() { String::new() } else { format!("{dir}/") };
        let mut skill_files: Vec<FetchedFile> = files
            .iter()
            .map(|(name, bytes)| FetchedFile {
                rel_path: name.clone(),
                bytes: bytes.clone(),
            })
            .collect();
        for (other_dir, other_files) in &by_dir {
            if other_dir != dir && other_dir.starts_with(&prefix) {
                let rel_dir = &other_dir[prefix.len()..];
                for (name, bytes) in other_files {
                    skill_files.push(FetchedFile {
                        rel_path: format!("{rel_dir}/{name}"),
                        bytes: bytes.clone(),
                    });
                }
            }
        }
        skill_files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        skills.push(DiscoveredSkill {
            name,
            description,
            skill_path: join_repo_path(dir, "SKILL.md"),
            repo_dir: dir.clone(),
            files: skill_files,
        });
    }
    if skills.is_empty() {
        return Err(InstallError::NoSkillsFound { source: source.clone() });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Strip the `<repo>-<branch>/` tarball prefix to repo-relative paths.
///
/// Caps entries (10k) and bytes (200 MB) to bound hostile tarballs.
fn extract_tarball(tarball: &[u8], source: &SkillSource) -> Result<Vec<(String, Vec<u8>)>, FetchError> {
    let url = tarball_url(source, "tarball");
    let decoder = flate2::read::GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(decoder);
    let mut out = Vec::new();
    let mut total_bytes: u64 = 0;
    let entries = archive.entries().map_err(|e| FetchError::BadResponse {
        url: url.clone(),
        reason: format!("unreadable tarball: {e}"),
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("unreadable tarball entry: {e}"),
        })?;
        if out.len() >= 10_000 {
            break;
        }
        let path = entry.path().map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("bad tarball path: {e}"),
        })?;
        // First segment is `<repo>-<branch>/`; drop it.
        let mut parts = path.components();
        let _top = parts.next();
        let rel: PathBuf = parts.collect();
        if rel.as_os_str().is_empty() {
            continue;
        }
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| FetchError::BadResponse {
            url: url.clone(),
            reason: format!("unreadable tarball entry: {e}"),
        })?;
        total_bytes += bytes.len() as u64;
        if total_bytes > 200_000_000 {
            return Err(FetchError::BadResponse {
                url: url.clone(),
                reason: "tarball exceeds 200 MB".to_string(),
            });
        }
        out.push((rel_str, bytes));
    }
    Ok(out)
}

fn normalize_scope(path: Option<&str>) -> Option<String> {
    path.map(|p| p.trim_matches('/').to_string()).filter(|p| !p.is_empty())
}

fn join_repo_path(dir: &str, file: &str) -> String {
    if dir.is_empty() {
        file.to_string()
    } else {
        format!("{dir}/{file}")
    }
}

/// Minimal frontmatter parser: `---\nname: x\ndescription: y\n---`.
///
/// Only `name` and `description` are read; the body is never inspected.
/// Accepts YAML supersets (quotes, extra keys) via `serde_yaml`.
pub fn parse_frontmatter(skill_md: &[u8]) -> Result<(String, String), String> {
    let text = std::str::from_utf8(skill_md).map_err(|e| format!("not UTF-8: {e}"))?;
    let rest = text.strip_prefix("---").ok_or("missing opening `---`")?;
    let end = rest.find("\n---").ok_or("missing closing `---`")?;
    let yaml = &rest[..end];
    #[derive(Deserialize)]
    struct Frontmatter {
        name: Option<String>,
        description: Option<String>,
    }
    let fm: Frontmatter =
        serde_yaml::from_str(yaml).map_err(|e| format!("invalid YAML frontmatter: {e}"))?;
    let name = fm.name.filter(|n| !n.trim().is_empty()).ok_or("missing `name:`")?;
    let description = fm
        .description
        .filter(|d| !d.trim().is_empty())
        .ok_or("missing `description:`")?;
    Ok((name.trim().to_string(), description.trim().to_string()))
}

/// Write skill files to `<root>/skills/<name>/`, refusing to escape the dir.
pub fn write_skill_files(skill_dir: &Path, files: &[FetchedFile]) -> Result<usize, InstallError> {
    std::fs::create_dir_all(skill_dir).map_err(|source| InstallError::Io {
        path: skill_dir.to_path_buf(),
        source,
    })?;
    let canonical_base = skill_dir.canonicalize().unwrap_or_else(|_| skill_dir.to_path_buf());
    let mut count = 0;
    for file in files {
        let dest = skill_dir.join(&file.rel_path);
        let normalized = normalize_join(&canonical_base, &dest);
        if !normalized.starts_with(&canonical_base) {
            return Err(InstallError::Io {
                path: dest,
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "path escapes skill dir"),
            });
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|source| InstallError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        std::fs::write(&dest, &file.bytes).map_err(|source| InstallError::Io {
            path: dest,
            source,
        })?;
        count += 1;
    }
    Ok(count)
}

fn normalize_join(base: &Path, joined: &Path) -> PathBuf {
    let mut out = base.to_path_buf();
    for comp in joined.components() {
        use std::path::Component;
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {}
            Component::Normal(part) => out.push(part),
        }
    }
    out
}

/// Full install pipeline: discover → filter → write → index.
///
/// Returns `(installed_names, index_path)`. New entries arrive active;
/// re-installs keep the user's existing `active` flag.
pub fn install(
    client: &GitHubClient,
    source: &SkillSource,
    only: Option<&str>,
    local: bool,
    home_dir: Option<PathBuf>,
    cwd: &Path,
) -> Result<(Vec<String>, PathBuf), InstallError> {
    use super::index as index_mod;
    let mut discovered = discover_skills(client, source)?;
    if let Some(only) = only {
        let found: Vec<String> = discovered.iter().map(|s| s.name.clone()).collect();
        discovered.retain(|s| s.name == only);
        if discovered.is_empty() {
            return Err(InstallError::OnlyNotFound {
                only: only.to_string(),
                found,
            });
        }
    }
    let mut installed = Vec::new();
    let mut index_path = PathBuf::new();
    for skill in &discovered {
        let target = install_target(&skill.name, local, home_dir.clone(), cwd)?;
        index_path = target.index_path.clone();
        if target.skill_dir.exists() {
            std::fs::remove_dir_all(&target.skill_dir).map_err(|source| InstallError::Io {
                path: target.skill_dir.clone(),
                source,
            })?;
        }
        write_skill_files(&target.skill_dir, &skill.files)?;
        let mut index = index_mod::load(&target.index_path)?;
        let repo_root = format!("{}/{}", source.owner, source.repo);
        index_mod::upsert(
            &mut index,
            skill.name.clone(),
            super::index::SkillEntry {
                source: repo_root,
                source_type: "github".to_string(),
                skill_path: skill.skill_path.clone(),
                active: true,
            },
        );
        index_mod::save(&target.index_path, &index)?;
        installed.push(skill.name.clone());
    }
    if installed.is_empty() {
        return Err(InstallError::Io {
            path: cwd.to_path_buf(),
            source: std::io::Error::other("nothing installed"),
        });
    }
    Ok((installed, index_path))
}
