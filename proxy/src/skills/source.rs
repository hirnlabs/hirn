use std::fmt;

/// A parsed `hirn add` source: `owner/repo`, `owner/repo/path`, or full URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSource {
    /// GitHub owner (`mattpocock`).
    pub owner: String,
    /// Repository name (`skills`).
    pub repo: String,
    /// Optional subdirectory inside the repo.
    pub path: Option<String>,
}

impl fmt::Display for SkillSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)?;
        if let Some(path) = &self.path {
            write!(f, "/{path}")?;
        }
        Ok(())
    }
}

/// What went wrong while parsing an `add` source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    /// Input had no usable segments at all.
    Empty,
    /// Shorthand had no `/` separating owner from repo.
    MissingRepo(String),
    /// A URL pointed somewhere other than github.com.
    UnsupportedHost(String),
    /// A `/owner/repo/...` URL path was missing the repo segment.
    MalformedUrl(String),
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceError::Empty => write!(f, "empty source"),
            SourceError::MissingRepo(input) => {
                write!(f, "expected `owner/repo`, got `{input}`")
            }
            SourceError::UnsupportedHost(host) => {
                write!(f, "only github.com sources are supported, got host `{host}`")
            }
            SourceError::MalformedUrl(input) => {
                write!(f, "expected github.com/owner/repo[/path], got `{input}`")
            }
        }
    }
}

impl std::error::Error for SourceError {}

/// Parse `owner/repo[/path]`, a GitHub URL, or a `/owner/repo/...` path.
///
/// Accepts both `tree/<branch>` and `blob/<branch>` URL infixes (branch is
/// ignored — fetches always use the default branch via the contents API).
pub fn parse_source(input: &str) -> Result<SkillSource, SourceError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SourceError::Empty);
    }
    if trimmed.contains("://") || trimmed.starts_with("github.com/") {
        parse_url_source(trimmed)
    } else {
        parse_shorthand_source(trimmed)
    }
}

fn parse_shorthand_source(input: &str) -> Result<SkillSource, SourceError> {
    let input = input.strip_suffix('/').unwrap_or(input);
    let mut parts = input.split('/').filter(|s| !s.is_empty());
    let owner = parts.next().unwrap_or("").to_string();
    let Some(repo) = parts.next() else {
        return Err(SourceError::MissingRepo(input.to_string()));
    };
    let rest: Vec<&str> = parts.collect();
    Ok(SkillSource {
        owner,
        repo: repo.to_string(),
        path: if rest.is_empty() { None } else { Some(rest.join("/")) },
    })
}

fn parse_url_source(input: &str) -> Result<SkillSource, SourceError> {
    let normalized = input.strip_prefix("https://").or_else(|| input.strip_prefix("http://")).unwrap_or(input);
    let normalized = normalized.strip_prefix("www.").unwrap_or(normalized);
    let (host, path) = match normalized.split_once('/') {
        Some(split) => split,
        None => return Err(SourceError::MalformedUrl(input.to_string())),
    };
    if host != "github.com" {
        return Err(SourceError::UnsupportedHost(host.to_string()));
    }
    let mut segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 2 {
        return Err(SourceError::MalformedUrl(input.to_string()));
    }
    let owner = segments.remove(0).to_string();
    let mut repo = segments.remove(0).to_string();
    repo = repo.strip_suffix(".git").unwrap_or(&repo).to_string();
    // Drop `tree/<branch>` / `blob/<branch>` infixes; branch is unused.
    if matches!(segments.first(), Some(s) if *s == "tree" || *s == "blob") {
        segments.drain(..1.min(segments.len()));
        if !segments.is_empty() {
            segments.remove(0);
        }
    }
    Ok(SkillSource {
        owner,
        repo,
        path: if segments.is_empty() { None } else { Some(segments.join("/")) },
    })
}
