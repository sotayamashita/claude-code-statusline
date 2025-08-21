use super::{Module, ModuleConfig};
use crate::types::context::Context;
use std::path::Path;

/// Module that displays the current working directory with HOME abbreviation
pub struct DirectoryModule;

impl DirectoryModule {
    /// Create a new DirectoryModule instance
    pub fn new() -> Self {
        Self
    }

    /// Create from Context (kept for compatibility)
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }

    /// Abbreviate home directory to ~
    fn abbreviate_home(&self, path: &Path) -> String {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(relative) = path.strip_prefix(&home) {
                if relative.as_os_str().is_empty() {
                    return "~".to_string();
                }
                return format!("~/{}", relative.display());
            }
        }
        path.display().to_string()
    }
}

impl Module for DirectoryModule {
    fn name(&self) -> &str {
        "directory"
    }

    fn should_display(&self, _context: &Context, _config: &dyn ModuleConfig) -> bool {
        true // Always display directory
    }

    fn render(&self, context: &Context, _config: &dyn ModuleConfig) -> String {
        self.abbreviate_home(&context.current_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo};

    #[test]
    fn test_directory_module() {
        let module = DirectoryModule::new();

        // Create a mock ClaudeInput
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/Users/test/projects".to_string(),
            model: ModelInfo {
                id: "test".to_string(),
                display_name: "Test".to_string(),
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert_eq!(module.name(), "directory");
        assert!(module.should_display(&context, &context.config.directory));
    }

    #[test]
    fn test_home_directory_abbreviation() {
        let module = DirectoryModule::new();

        // Set HOME environment variable for testing
        // Note: set_var is unsafe in Rust 1.77+
        unsafe {
            std::env::set_var("HOME", "/Users/test");
        }

        // Test exact HOME directory
        let input_home = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/Users/test".to_string(),
            model: ModelInfo {
                id: "test".to_string(),
                display_name: "Test".to_string(),
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let config = Config::default();
        let context_home = Context::new(input_home, config.clone());
        assert_eq!(
            module.render(&context_home, &context_home.config.directory),
            "~"
        );

        // Test subdirectory of HOME
        let input_subdir = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/Users/test/projects".to_string(),
            model: ModelInfo {
                id: "test".to_string(),
                display_name: "Test".to_string(),
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let context_subdir = Context::new(input_subdir, config);
        assert_eq!(
            module.render(&context_subdir, &context_subdir.config.directory),
            "~/projects"
        );
    }
}
