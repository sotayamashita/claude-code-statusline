use super::{Module, ModuleConfig};
use crate::types::context::Context;
use std::path::Path;

pub struct DirectoryModule;

impl DirectoryModule {
    pub fn new() -> Self {
        Self
    }

    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }

    /// Abbreviate home directory to ~
    fn abbreviate_home(&self, path: &Path) -> String {
        if let Ok(home) = std::env::var("HOME") {
            if let Ok(relative) = path.strip_prefix(&home) {
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
    use crate::modules::EmptyConfig;
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
        let module_config = EmptyConfig;

        assert_eq!(module.name(), "directory");
        assert!(module.should_display(&context, &module_config));
    }
}
