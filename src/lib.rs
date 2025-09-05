//! Beacon library shim
//!
//! Temporary compatibility layer that re-exports the public API from
//! the `beacon-core` crate so existing imports using `beacon::...` keep
//! working. New code can import directly from `beacon_core`, but this
//! shim allows a gradual migration without breaking external users.
//!
//! Examples
//!
//! - Engine API: `beacon::engine::Engine`
//! - Types: `beacon::types::context::Context`, `beacon::Config`
//! - Parser: `beacon::parse_claude_input`

// Engine is provided by beacon-core; re-export as a module path
pub use beacon_core::engine;

// Re-export core modules from beacon-core
pub use beacon_core as core; // optional alias for consumers
pub use beacon_core::config;
pub use beacon_core::debug;
pub use beacon_core::messages;
pub use beacon_core::modules;
pub use beacon_core::parser;
pub use beacon_core::style;
pub use beacon_core::timeout;
pub use beacon_core::types;

// Re-export commonly used items for convenience
pub use beacon_core::Config;
pub use beacon_core::parse_claude_input;
pub use beacon_core::types::context::Context;
pub use debug::DebugLogger;
