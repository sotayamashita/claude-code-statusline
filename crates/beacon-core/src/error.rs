//! Core error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("unknown module: {0}")]
    UnknownModule(String),
    #[error("module timed out: {0}")]
    ModuleTimeout(String),
    #[cfg(feature = "git")]
    #[error(transparent)]
    Git(#[from] git2::Error),
}
