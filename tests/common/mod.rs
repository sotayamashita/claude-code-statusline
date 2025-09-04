// Test fixtures and helpers using rstest
pub mod builders;
pub mod cli;
pub mod fixtures;

pub use builders::{ClaudeInputBuilder, ContextBuilder};
pub use cli::{beacon_cmd, input_json_with_cwd, write_basic_config};
pub use fixtures::{
    TestRenderer, default_claude_input, default_config, default_context, test_paths,
};
