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

impl Default for ClaudeModelModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ClaudeModelModule {
    fn name(&self) -> &str {
        "claude_model"
    }

    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool {
        // Check if the module is disabled in config
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::ClaudeModelConfig>()
        {
            if cfg.disabled {
                return false;
            }
        }
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
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};
    use crate::types::context::Context;
    use rstest::*;

    /// Helper to create context with specific model
    fn context_with_model(model_name: &str) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: format!("claude-{}", model_name.to_lowercase()),
                display_name: model_name.to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/test/dir".to_string(),
                project_dir: Some("/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };
        Context::new(input, Config::default())
    }

    #[rstest]
    #[case("Opus", "<Opus>")]
    #[case("Sonnet", "<Sonnet>")]
    #[case("Haiku", "<Haiku>")]
    #[case("Claude-3.5", "<Claude-3.5>")]
    fn test_model_rendering(#[case] model_name: &str, #[case] expected: &str) {
        let module = ClaudeModelModule::new();
        let context = context_with_model(model_name);

        assert_eq!(module.name(), "claude_model");
        assert!(module.should_display(&context, &context.config.claude_model));
        assert_eq!(
            module.render(&context, &context.config.claude_model),
            expected
        );
    }

    #[rstest]
    #[case("", false)]
    #[case("   ", false)]
    #[case("\t\n", false)]
    #[case("Opus", true)]
    fn test_should_display_with_different_model_names(
        #[case] model_name: &str,
        #[case] should_display: bool,
    ) {
        let module = ClaudeModelModule::new();
        let context = context_with_model(model_name);

        assert_eq!(
            module.should_display(&context, &context.config.claude_model),
            should_display
        );
    }

    #[rstest]
    fn test_module_metadata() {
        let module = ClaudeModelModule::new();
        assert_eq!(module.name(), "claude_model");
    }

    #[rstest]
    fn test_from_context_constructor() {
        let context = context_with_model("Opus");
        let module = ClaudeModelModule::from_context(&context);
        assert_eq!(module.name(), "claude_model");
    }
}
