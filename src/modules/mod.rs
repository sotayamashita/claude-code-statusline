use crate::types::context::Context;

/// Trait that all status line modules must implement
pub trait Module {
    /// Returns the name of the module
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Determines if this module should be displayed
    fn should_display(&self) -> bool;

    /// Renders the module's output as a string
    fn render(&self) -> String;
}

// Re-export module implementations
pub mod claude_model;
pub mod directory;

pub use claude_model::ClaudeModelModule;
pub use directory::DirectoryModule;

/// Central module dispatcher - creates module instances based on name
/// This implements the Factory pattern for dynamic module creation
pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    match name {
        "directory" => Some(Box::new(DirectoryModule::from_context(context))),
        "claude_model" => Some(Box::new(ClaudeModelModule::from_context(context))),
        _ => None,
    }
}
