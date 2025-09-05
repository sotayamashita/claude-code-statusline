//! Beacon Core Library
//!
//! This crate hosts the core modules, types, parser, style helpers,
//! timeout utilities, and configuration used by Beacon.

pub mod config;
pub mod debug;
pub mod engine;
pub mod messages;
pub mod modules;
pub mod parser;
pub mod style;
pub mod timeout;
pub mod types;

// Convenience re-exports for common types/functions
pub use config::Config;
pub use engine::Engine;
pub use parser::parse_claude_input;
pub use types::context::Context;
