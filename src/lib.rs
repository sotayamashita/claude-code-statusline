//! Beacon library shim
//!
//! フェーズ1の段階移行: core の実装を `beacon-core` クレートへ移動し、
//! ここでは公開面を再エクスポートします。既存の `beacon::...` パスは維持されます。

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
