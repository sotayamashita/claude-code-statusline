//! claude-code-statusline Core Library
//!
//! This crate hosts the core modules, types, parser, style helpers,
//! timeout utilities, and configuration used by claude-code-statusline.

pub mod config;
pub mod debug;
pub mod engine;
pub mod error;
pub mod messages;
pub mod modules;
pub mod parser;
pub mod style;
pub mod timeout;
pub mod types;

// Convenience re-exports for common types/functions
pub use config::Config;
pub use config::ConfigProvider;
pub use config::config_path;
pub use engine::Engine;
pub use error::CoreError;
pub use parser::parse_claude_input;
pub use types::context::Context;
