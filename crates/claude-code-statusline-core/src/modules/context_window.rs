//! Context window usage module for displaying token consumption percentage
//!
//! This module shows the current context window usage as a percentage,
//! with color coding based on usage levels.

use super::{Module, ModuleConfig};
use crate::types::context::Context;

/// Module that displays context window usage percentage
///
/// Renders the context window usage with automatic color coding:
/// - Green: < 50% usage
/// - Yellow: 50-79% usage
/// - Red: ≥ 80% usage
///
/// # Configuration
///
/// ```toml
/// [context_window]
/// format = "[$symbol$percentage%]($style)"
/// symbol = "ctx "
/// disabled = false
/// ```
///
/// # Calculation
///
/// Percentage = (input_tokens + cache_creation_input_tokens + cache_read_input_tokens) * 100 / context_window_size
pub struct ContextWindowModule;

impl ContextWindowModule {
    /// Create a new ContextWindowModule instance
    pub fn new() -> Self {
        Self
    }

    /// Create from Context (kept for compatibility)
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }

    /// Calculate the usage percentage from context window data
    fn calculate_percentage(context: &Context) -> Option<u64> {
        let ctx_window = context.input.context_window.as_ref()?;

        // Use total_input_tokens directly from Claude Code
        let total_tokens = ctx_window.total_input_tokens;

        if ctx_window.context_window_size == 0 {
            return None;
        }

        Some((total_tokens * 100) / ctx_window.context_window_size)
    }

    /// Get the color style based on percentage
    fn get_color_for_percentage(percentage: u64) -> &'static str {
        if percentage < 50 {
            "green"
        } else if percentage < 80 {
            "yellow"
        } else {
            "red"
        }
    }
}

impl Default for ContextWindowModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ContextWindowModule {
    fn name(&self) -> &str {
        "context_window"
    }

    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool {
        // Check if the module is disabled in config
        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::ContextWindowConfig>()
        {
            if cfg.disabled {
                return false;
            }
        }

        // Only display if context_window data is present
        Self::calculate_percentage(context).is_some()
    }

    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String {
        let percentage = match Self::calculate_percentage(context) {
            Some(p) => p,
            None => return String::new(),
        };

        if let Some(cfg) = config
            .as_any()
            .downcast_ref::<crate::types::config::ContextWindowConfig>()
        {
            use std::collections::HashMap;
            let mut tokens = HashMap::new();
            tokens.insert("percentage", percentage.to_string());
            tokens.insert("symbol", cfg.symbol.clone());

            // Determine color based on percentage
            let color = Self::get_color_for_percentage(percentage);
            let style = if cfg.use_dynamic_color {
                color.to_string()
            } else {
                cfg.style.clone()
            };

            return crate::style::render_with_style_template(cfg.format(), &tokens, &style);
        }

        // Fallback rendering
        format!("[ctx {}%]", percentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ContextWindow, CurrentUsage, ModelInfo};
    use crate::types::context::Context;
    use rstest::*;

    /// Helper to create context with specific context window data
    fn context_with_usage(total_input: u64, total_output: u64, window_size: u64) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: None,
            version: Some("1.0.0".to_string()),
            output_style: None,
            context_window: Some(ContextWindow {
                total_input_tokens: total_input,
                total_output_tokens: total_output,
                context_window_size: window_size,
                current_usage: None,
            }),
        };
        Context::new(input, Config::default())
    }

    #[rstest]
    #[case(25000, 0, 200000, 12)] // 12.5% rounds down to 12%
    #[case(50000, 0, 200000, 25)] // 25%
    #[case(100000, 0, 200000, 50)] // 50%
    #[case(150000, 0, 200000, 75)] // 75%
    #[case(160000, 0, 200000, 80)] // 80%
    fn test_percentage_calculation(
        #[case] input: u64,
        #[case] output: u64,
        #[case] size: u64,
        #[case] expected: u64,
    ) {
        let context = context_with_usage(input, output, size);
        let percentage = ContextWindowModule::calculate_percentage(&context);
        assert_eq!(percentage, Some(expected));
    }

    #[rstest]
    #[case(30, "green")]
    #[case(49, "green")]
    #[case(50, "yellow")]
    #[case(79, "yellow")]
    #[case(80, "red")]
    #[case(95, "red")]
    fn test_color_for_percentage(#[case] percentage: u64, #[case] expected_color: &str) {
        let color = ContextWindowModule::get_color_for_percentage(percentage);
        assert_eq!(color, expected_color);
    }

    #[rstest]
    fn test_should_display_with_context_window() {
        let module = ContextWindowModule::new();
        let context = context_with_usage(50000, 0, 200000);
        assert!(module.should_display(&context, &context.config.context_window));
    }

    #[rstest]
    fn test_should_not_display_without_context_window() {
        let module = ContextWindowModule::new();
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: None,
            version: Some("1.0.0".to_string()),
            output_style: None,
            context_window: None,
        };
        let context = Context::new(input, Config::default());
        assert!(!module.should_display(&context, &context.config.context_window));
    }

    #[rstest]
    fn test_module_metadata() {
        let module = ContextWindowModule::new();
        assert_eq!(module.name(), "context_window");
    }

    #[rstest]
    fn test_render_contains_percentage() {
        let module = ContextWindowModule::new();
        let context = context_with_usage(50000, 0, 200000);
        let rendered = module.render(&context, &context.config.context_window);
        let plain = String::from_utf8(strip_ansi_escapes::strip(rendered)).unwrap();
        assert!(plain.contains("25"));
    }
}
