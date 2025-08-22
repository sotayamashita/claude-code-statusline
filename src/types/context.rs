use crate::config::Config;
use crate::types::claude::ClaudeInput;
use std::path::PathBuf;

/// Central context structure that holds all runtime data and configuration
pub struct Context {
    /// Raw input from Claude Code
    pub input: ClaudeInput,

    /// Application configuration
    #[allow(dead_code)]
    pub config: Config,

    /// Current working directory (processed)
    pub current_dir: PathBuf,

    /// Project root directory (e.g., git repository root)
    /// Will be populated in Phase 2 when git support is added
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use rstest::rstest;

    /// Helper to create test ClaudeInput
    fn create_claude_input(cwd: &str, model: &str, workspace: Option<(&str, &str)>) -> ClaudeInput {
        ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: format!("claude-{}", model.to_lowercase()),
                display_name: model.to_string(),
            },
            workspace: workspace.map(|(current, project)| WorkspaceInfo {
                current_dir: current.to_string(),
                project_dir: Some(project.to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        }
    }

    #[rstest]
    #[case("/test/dir", "Opus")]
    #[case("/another/dir", "Sonnet")]
    #[case("/home/user", "Haiku")]
    fn test_context_creation(#[case] cwd: &str, #[case] model: &str) {
        let input = create_claude_input(cwd, model, Some((cwd, "/test/project")));
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), cwd);
        assert_eq!(context.model_display_name(), model);
    }

    #[rstest]
    #[case("/test/dir", "/test/project", Some(PathBuf::from("/test/project")))]
    #[case(
        "/another/dir",
        "/another/project",
        Some(PathBuf::from("/another/project"))
    )]
    fn test_context_with_workspace(
        #[case] cwd: &str,
        #[case] project: &str,
        #[case] expected_root: Option<PathBuf>,
    ) {
        let input = create_claude_input(cwd, "Opus", Some((cwd, project)));
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), cwd);
        assert_eq!(context.project_root, expected_root);
    }

    #[rstest]
    fn test_context_without_workspace() {
        let input = create_claude_input("/another/dir", "Sonnet", None);
        let context = Context::new(input, Config::default());

        assert_eq!(context.current_dir_str(), "/another/dir");
        assert_eq!(context.model_display_name(), "Sonnet");
        assert_eq!(context.project_root, None);
    }
}
