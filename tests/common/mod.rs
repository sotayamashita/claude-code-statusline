// Test fixtures and helpers using rstest
pub mod fixtures;
pub mod builders;

pub use builders::{ClaudeInputBuilder, ContextBuilder};
pub use fixtures::{
    default_context,
    default_claude_input,
    default_config,
    test_paths,
    TestRenderer,
};