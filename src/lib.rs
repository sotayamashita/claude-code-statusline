//! claude-code-statusline library shim
//!
//! Temporary compatibility layer that re-exports the public API from
//! the `claude-code-statusline-core` crate so existing imports using
//! `claude_code_statusline::...` keep working. New code can import
//! directly from `claude_code_statusline_core`, but this
//! shim allows a gradual migration without breaking external users.
//!
//! Examples
//!
//! - Engine API: `claude_code_statusline::engine::Engine`
//! - Types: `claude_code_statusline::types::context::Context`, `claude_code_statusline::Config`
//! - Parser: `claude_code_statusline::parse_claude_input`

// Engine is provided by claude-code-statusline-core; re-export as a module path
pub use claude_code_statusline_core::engine;

// Re-export core modules from claude-code-statusline-core
pub use claude_code_statusline_core as core; // optional alias for consumers
pub use claude_code_statusline_core::config;
pub use claude_code_statusline_core::debug;
pub use claude_code_statusline_core::messages;
pub use claude_code_statusline_core::modules;
pub use claude_code_statusline_core::parser;
pub use claude_code_statusline_core::style;
pub use claude_code_statusline_core::timeout;
pub use claude_code_statusline_core::types;

// Re-export commonly used items for convenience
pub use claude_code_statusline_core::Config;
pub use claude_code_statusline_core::parse_claude_input;
pub use claude_code_statusline_core::types::context::Context;
pub use debug::DebugLogger;
