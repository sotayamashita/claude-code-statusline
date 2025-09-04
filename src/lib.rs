//! Beacon library - Core functionality for the status line generator
//!
//! This library provides the building blocks for generating formatted
//! status lines for AI development environments. It exports all major
//! modules and commonly used types.
//!
//! # Modules
//!
//! * [`config`] - Configuration loading and validation from TOML files
//! * [`debug`] - Debug logging utilities for development
//! * [`messages`] - Predefined error and status messages
//! * [`modules`] - Status line component implementations
//! * [`parser`] - JSON input parsing and format string processing
//! * [`style`] - ANSI color and text styling utilities
//! * [`timeout`] - Command execution with timeout handling
//! * [`types`] - Core type definitions and data structures
//!
//! # Quick Start
//!
//! ```no_run
//! use beacon::{Config, parse_claude_input, Context};
//!
//! // Load configuration
//! let config = Config::load().expect("Failed to load config");
//!
//! // Parse JSON input
//! let json = r#"{"cwd":"/tmp","model":{"id":"claude","display_name":"Claude"}}"#;
//! let input = parse_claude_input(json).expect("Failed to parse input");
//!
//! // Create context and generate status line
//! let context = Context::new(input, config);
//! ```

/// Configuration loading and validation module
pub mod config;

/// Debug logging utilities for tracing execution
pub mod debug;

/// Predefined messages for various states and errors
pub mod messages;

/// Status line component modules (directory, git, model, etc.)
pub mod modules;

/// JSON parsing and format string processing utilities
pub mod parser;

/// ANSI color and text styling functions
pub mod style;

/// Command execution with timeout handling
pub mod timeout;

/// Core type definitions and data structures
pub mod types;

// Re-export commonly used items for convenience
pub use config::Config;
pub use debug::DebugLogger;
pub use parser::parse_claude_input;
pub use types::context::Context;
