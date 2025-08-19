/// Trait that all status line modules must implement
pub trait Module {
    /// Returns the name of the module
    fn name(&self) -> &str;
    
    /// Determines if this module should be displayed
    fn should_display(&self) -> bool;
    
    /// Renders the module's output as a string
    fn render(&self) -> String;
}

// Re-export module implementations
pub mod directory;
pub mod character;
pub mod claude_model;

pub use directory::DirectoryModule;
pub use character::CharacterModule;
pub use claude_model::ClaudeModelModule;