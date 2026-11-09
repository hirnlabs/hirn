//! The error type shared by every tier and the RPC surface.

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

/// Everything the data backend can fail with.
#[derive(Debug)]
pub enum Error {
    /// The project directory or one of its files could not be read or written.
    Io {
        path: String,
        source: std::io::Error,
    },
    /// Markdown parsing or frontmatter handling failed.
    Parse(String),
    /// `schema.toml` could not be loaded.
    Schema(String),
    /// The Loro engine rejected an operation.
    Loro(String),
    /// A SQLite statement failed.
    Sqlite(String),
    /// The graph engine rejected an operation.
    Graph(String),
    /// An embedding could not be produced.
    Embedding(String),
    /// The caller asked for something that does not exist.
    NotFound(String),
    /// The caller asked for something the backend refuses.
    Invalid(String),
    /// The RPC envelope was malformed or named an unknown method.
    Rpc(String),
}

impl Error {
    /// Wrap an I/O failure with the path it happened on.
    pub fn io(path: impl AsRef<std::path::Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().display().to_string(),
            source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "io error at {path}: {source}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::Schema(msg) => write!(f, "schema error: {msg}"),
            Self::Loro(msg) => write!(f, "loro error: {msg}"),
            Self::Sqlite(msg) => write!(f, "sqlite error: {msg}"),
            Self::Graph(msg) => write!(f, "graph error: {msg}"),
            Self::Embedding(msg) => write!(f, "embedding error: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Invalid(msg) => write!(f, "invalid request: {msg}"),
            Self::Rpc(msg) => write!(f, "rpc error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value.to_string())
    }
}

impl From<loro::LoroError> for Error {
    fn from(value: loro::LoroError) -> Self {
        Self::Loro(value.to_string())
    }
}

impl From<loro::LoroEncodeError> for Error {
    fn from(value: loro::LoroEncodeError) -> Self {
        Self::Loro(value.to_string())
    }
}

impl From<grafeo::Error> for Error {
    fn from(value: grafeo::Error) -> Self {
        Self::Graph(value.to_string())
    }
}
