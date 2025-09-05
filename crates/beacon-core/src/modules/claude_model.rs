//! Claude model module for displaying the active AI model
//!
//! This module shows the current Claude model being used in the session
//! with optional symbol and styling.

use super::{Module, ModuleConfig};
use crate::types::context::Context;

/// Module that displays the current Claude model name
///
/// Renders the Claude model information with automatic compaction
/// for version numbers (e.g., "Opus 4.1" → "Opus4.1") and
/// customizable symbol prefix.
///
/// # Configuration
///
/// ```toml
/// [claude_model]
/// format = "[$symbol$model]($style)"
/// style = "bold yellow"
/// symbol = "<"
/// disabled = false
/// ```
///
/// # Display Rules
///
/// - Compacts spaces before digits (e.g., "Sonnet 3.5" → "Sonnet3.5")
/// - Only displays when model name is non-empty
/// - Can be disabled via configuration
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
            && cfg.disabled
        {
            return false;
        }
        !context.model_display_name().trim().is_empty()
    }

    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String {
        let model = context.model_display_name();

        // Compact pattern like "Opus 4.1" or "Sonnet 4" -> "Opus4.1" / "Sonnet4"
        // Rule: remove a single space immediately before a digit.
        let compacted_model = {
            let s = model;
            let mut out = String::with_capacity(s.len());
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len();
            let mut i = 0;
            while i < len {
                let c = chars[i];
                if c == ' ' && i + 1 < len && chars[i + 1].is_ascii_digit() {
                    // skip this space
                    i += 1;
                    continue;
                }
                out.push(c);
                i += 1;
            }
            out
        };

        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::ClaudeModelConfig>()
        {
            use std::collections::HashMap;
            let mut tokens = HashMap::new();
            tokens.insert("model", compacted_model);
            tokens.insert("symbol", cfg.symbol.clone());
            return crate::style::render_with_style_template(cfg.format(), &tokens, cfg.style());
        }

        compacted_model
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
    #[case("Opus")]
    #[case("Sonnet")]
    #[case("Haiku")]
    #[case("Claude-3.5")]
    fn test_model_rendering(#[case] model_name: &str) {
        let module = ClaudeModelModule::new();
        let context = context_with_model(model_name);

        assert_eq!(module.name(), "claude_model");
        assert!(module.should_display(&context, &context.config.claude_model));
        let rendered = module.render(&context, &context.config.claude_model);
        assert!(rendered.contains(model_name));
        // Default config applies ANSI style
        assert!(rendered.starts_with("\u{1b}[") && rendered.ends_with("\u{1b}[0m"));
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

    #[rstest]
    fn compacts_space_before_digits() {
        let module = ClaudeModelModule::new();
        let context = context_with_model("Sonnet 4");
        let rendered = module.render(&context, &context.config.claude_model);
        // strip ANSI for assertion
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        assert!(plain.contains("Sonnet4"));
        assert!(!plain.contains("Sonnet 4"));
    }
}
