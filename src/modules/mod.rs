use crate::types::context::Context;
use std::any::Any;

/// Trait for module-specific configuration
pub trait ModuleConfig: Any + Send + Sync {
    /// Allow downcasting to concrete config types
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;

    /// Get the format string for this module
    #[allow(dead_code)]
    fn format(&self) -> &str {
        ""
    }

    /// Get the style string for this module
    #[allow(dead_code)]
    fn style(&self) -> &str {
        ""
    }
}

/// Default implementation for cases where no config is provided
pub struct EmptyConfig;

impl ModuleConfig for EmptyConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait that all status line modules must implement
pub trait Module {
    /// Returns the name of the module
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Determines if this module should be displayed
    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool;

    /// Renders the module's output as a string
    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String;
}

// Re-export module implementations
pub mod claude_model;
pub mod directory;
pub mod git_branch;
pub mod git_status;

pub use claude_model::ClaudeModelModule;
pub use directory::DirectoryModule;
use git_branch::GitBranchModule;
use git_status::GitStatusModule;

/// Central module dispatcher - creates module instances based on name
/// This implements the Factory pattern for dynamic module creation
pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    match name {
        "directory" => Some(Box::new(DirectoryModule::from_context(context))),
        "claude_model" => Some(Box::new(ClaudeModelModule::from_context(context))),
        "git_branch" => Some(Box::new(GitBranchModule::from_context(context))),
        "git_status" => Some(Box::new(GitStatusModule::from_context(context))),
        _ => None,
    }
}
