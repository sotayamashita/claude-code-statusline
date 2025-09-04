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

    /// Abbreviate home directory to ~ (cross-platform)
    fn abbreviate_home(&self, path: &Path) -> String {
        if let Some(home) = dirs::home_dir() {
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

impl Default for DirectoryModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DirectoryModule {
    fn name(&self) -> &str {
        "directory"
    }

    fn should_display(&self, _context: &Context, config: &dyn ModuleConfig) -> bool {
        // Check if the module is disabled in config
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::DirectoryConfig>()
        {
            return !cfg.disabled;
        }
        true // Default to displaying if no config found
    }

    fn render(&self, context: &Context, _config: &dyn ModuleConfig) -> String {
        self.abbreviate_home(&context.current_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::context::Context;
    use rstest::*;

    /// Fixture for creating test contexts
    #[fixture]
    fn test_context() -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/Users/test/projects".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/Users/test/projects".to_string(),
                project_dir: Some("/Users/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    /// Helper to create context with specific cwd
    fn context_with_cwd(cwd: &str) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: cwd.to_string(),
                project_dir: Some("/Users/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    #[rstest]
    fn test_directory_module(test_context: Context) {
        let module = DirectoryModule::new();
        assert_eq!(module.name(), "directory");
        assert!(module.should_display(&test_context, &test_context.config.directory));
    }

    #[rstest]
    #[case("/Users/test", "~")]
    #[case("/Users/test/projects", "~/projects")]
    #[case("/Users/test/Documents/code", "~/Documents/code")]
    fn test_home_directory_abbreviation(#[case] cwd: &str, #[case] expected: &str) {
        let module = DirectoryModule::new();
        // Save and set HOME environment variable
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", "/Users/test");
        }

        let context = context_with_cwd(cwd);
        assert_eq!(module.render(&context, &context.config.directory), expected);

        // Restore original HOME
        unsafe {
            if let Some(home) = original_home {
                std::env::set_var("HOME", home);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[rstest]
    #[case("/var/www/html", "/var/www/html")]
    #[case("/tmp/test", "/tmp/test")]
    #[case("/usr/local/bin", "/usr/local/bin")]
    fn test_non_home_paths(#[case] cwd: &str, #[case] expected: &str) {
        let module = DirectoryModule::new();
        let context = context_with_cwd(cwd);
        assert_eq!(module.render(&context, &context.config.directory), expected);
    }
}
