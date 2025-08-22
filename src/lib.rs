// Export public modules for testing
pub mod config;
pub mod modules;
pub mod parser;
pub mod types;

// Re-export commonly used items
pub use config::Config;
pub use parser::parse_claude_input;
pub use types::context::Context;
