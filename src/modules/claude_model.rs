use super::{Module, ModuleConfig};
use crate::types::context::Context;

/// Module that displays the current Claude model name
pub struct ClaudeModelModule;

impl ClaudeModelModule {
    /// Create a new ClaudeModelModule instance
    pub fn new() -> Self {
        Self
    }

    /// Create from Context (kept for compatibility)
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }
}

impl Module for ClaudeModelModule {
    fn name(&self) -> &str {
        "claude_model"
    }

    fn should_display(&self, context: &Context, _config: &dyn ModuleConfig) -> bool {
        !context.model_display_name().trim().is_empty()
    }

    fn render(&self, context: &Context, _config: &dyn ModuleConfig) -> String {
        format!("<{}>", context.model_display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo};

    #[test]
    fn test_claude_model_module() {
        let module = ClaudeModelModule::new();

        // Create a mock ClaudeInput with model info
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/test".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert_eq!(module.name(), "claude_model");
        assert!(module.should_display(&context, &context.config.claude_model));
        assert_eq!(
            module.render(&context, &context.config.claude_model),
            "<Opus>"
        );
    }

    #[test]
    fn test_empty_model_name() {
        let module = ClaudeModelModule::new();

        // Create a mock ClaudeInput with empty model display name
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/test".to_string(),
            model: ModelInfo {
                id: "test".to_string(),
                display_name: "".to_string(), // Empty display name
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert!(!module.should_display(&context, &context.config.claude_model));
    }

    #[test]
    fn test_whitespace_only_model_name() {
        let module = ClaudeModelModule::new();

        // Create a mock ClaudeInput with whitespace-only model display name
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test".to_string(),
            transcript_path: None,
            cwd: "/test".to_string(),
            model: ModelInfo {
                id: "test".to_string(),
                display_name: "   ".to_string(), // Whitespace-only display name
            },
            workspace: None,
            version: None,
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        assert!(!module.should_display(&context, &context.config.claude_model));
    }
}
