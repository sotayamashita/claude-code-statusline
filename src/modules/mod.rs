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
