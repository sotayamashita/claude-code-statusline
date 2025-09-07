//! Core error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    // Generic/flow errors
    #[error("unknown module: {0}")]
    UnknownModule(String),
    #[error("missing module config: {0}")]
    MissingConfig(String),
    #[error("module timed out: {0}")]
    ModuleTimeout(String),

    // Config and parsing
    #[error("invalid JSON input: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("invalid TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),

    // IO
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to read config at {path}: {source}")]
    ConfigRead {
        path: String,
        source: std::io::Error,
    },
    #[error("invalid TOML at {path}: {source}")]
    ConfigParse {
        path: String,
        source: toml::de::Error,
    },

    // Timeout helpers
    #[error("task panicked")]
    TaskPanic,
    #[error("worker disconnected")]
    WorkerDisconnected,

    // Validation
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    // Optional git errors
    #[cfg(feature = "git")]
    #[error(transparent)]
    Git(#[from] git2::Error),
}
