use crate::config::Config;
use crate::types::claude::ClaudeInput;
use std::path::PathBuf;

/// Central context structure that holds all runtime data and configuration
pub struct Context {
    /// Raw input from Claude Code
    pub input: ClaudeInput,

    /// Application configuration
    pub config: Config,

    /// Current working directory (processed)
    pub current_dir: PathBuf,

    /// Project root directory (e.g., git repository root)
    /// Will be populated in Phase 2 when git support is added
    pub project_root: Option<PathBuf>,
}

impl Context {
    /// Create a new Context from ClaudeInput and Config
    pub fn new(input: ClaudeInput, config: Config) -> Self {
        let current_dir = PathBuf::from(&input.cwd);

        // For now, project_root is the same as workspace.project_dir if available
        let project_root = input
            .workspace
            .as_ref()
            .and_then(|ws| ws.project_dir.as_ref())
            .map(PathBuf::from);

        Self {
            input,
            config,
            current_dir,
            project_root,
        }
    }

    /// Get the current directory as a string
    pub fn current_dir_str(&self) -> &str {
        &self.input.cwd
    }

    /// Get the model display name
    pub fn model_display_name(&self) -> &str {
        &self.input.model.display_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::claude::{ModelInfo, WorkspaceInfo};

    #[test]
    fn test_context_creation() {
        let input = ClaudeInput {
            hook_event_name: Some("Status".to_string()),
            session_id: "test-123".to_string(),
            transcript_path: Some("/test/transcript.json".to_string()),
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/test/dir".to_string(),
                project_dir: Some("/test/project".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert_eq!(context.current_dir_str(), "/test/dir");
        assert_eq!(context.model_display_name(), "Opus");
        assert_eq!(context.project_root, Some(PathBuf::from("/test/project")));
    }

    #[test]
    fn test_context_without_workspace() {
        let input = ClaudeInput {
            hook_event_name: Some("Status".to_string()),
            session_id: "test-456".to_string(),
            transcript_path: None,
            cwd: "/another/dir".to_string(),
            model: ModelInfo {
                id: "claude-sonnet".to_string(),
                display_name: "Sonnet".to_string(),
            },
            workspace: None,
            version: Some("1.0.0".to_string()),
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert_eq!(context.current_dir_str(), "/another/dir");
        assert_eq!(context.model_display_name(), "Sonnet");
        assert_eq!(context.project_root, None);
    }
}
