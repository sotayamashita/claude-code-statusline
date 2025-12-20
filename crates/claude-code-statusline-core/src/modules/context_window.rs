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

    /// Creates a new ContextWindowModule while preserving the older API that accepted a `Context`.
    ///
    /// The supplied `context` parameter is unused and kept only for compatibility with call sites that
    /// previously constructed the module from a `Context`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ctx = crate::types::context::Context::default();
    /// let m1 = crate::modules::context_window::ContextWindowModule::new();
    /// let m2 = crate::modules::context_window::ContextWindowModule::from_context(&ctx);
    /// // Both constructors produce equivalent module instances (no internal state).
    /// ```
    pub fn from_context(_context: &Context) -> Self {
        Self::new()
    }

    /// Computes the context window usage as a percentage of the configured window size.
    ///
    /// When available, the calculation uses the breakdown in `current_usage` (sum of `input_tokens`,
    /// `cache_creation_input_tokens`, and `cache_read_input_tokens`); otherwise it falls back to
    /// `total_input_tokens`. Returns `None` when `context_window_size` is zero.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Given a context whose context window contains 25 used tokens and a window size of 100:
    /// // `calculate_percentage(&context)` returns `Some(25)`.
    /// let pct = calculate_percentage(&context);
    /// assert_eq!(pct, Some(25));
    /// ```
    fn calculate_percentage(context: &Context) -> Option<u64> {
        let ctx_window = context.input.context_window.as_ref()?;

        // Prefer current_usage breakdown if available (includes cached tokens)
        let total_tokens = if let Some(usage) = &ctx_window.current_usage {
            usage.input_tokens + usage.cache_creation_input_tokens + usage.cache_read_input_tokens
        } else {
            // Fallback to total_input_tokens if current_usage not available
            ctx_window.total_input_tokens
        };

        if ctx_window.context_window_size == 0 {
            return None;
        }

        total_tokens
            .checked_mul(100)?
            .checked_div(ctx_window.context_window_size)
    }

    /// Selects a style string from the config based on a usage percentage.
    ///
    /// Uses configurable thresholds from the config:
    /// - Less than `threshold_medium` -> `style_low`
    /// - `threshold_medium` through `threshold_high - 1` -> `style_medium`
    /// - `threshold_high` or greater -> `style_high`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let cfg = crate::types::config::ContextWindowConfig::default();
    /// assert_eq!(get_style_for_percentage(10, &cfg), cfg.style_low);
    /// assert_eq!(get_style_for_percentage(50, &cfg), cfg.style_medium);
    /// assert_eq!(get_style_for_percentage(95, &cfg), cfg.style_high);
    /// ```
    fn get_style_for_percentage(percentage: u64, config: &crate::types::config::ContextWindowConfig) -> String {
        if percentage < config.threshold_medium {
            config.style_low.clone()
        } else if percentage < config.threshold_high {
            config.style_medium.clone()
        } else {
            config.style_high.clone()
        }
    }
}

impl Default for ContextWindowModule {
    /// Constructs a new ContextWindowModule with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// let module = ContextWindowModule::default();
    /// let _module2 = ContextWindowModule::new();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ContextWindowModule {
    /// Module identifier for the context window module.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let module = crate::modules::context_window::ContextWindowModule::new();
    /// assert_eq!(module.name(), "context_window");
    /// ```
    ///
    /// # Returns
    ///
    /// `"context_window"` — the module's identifier string.
    fn name(&self) -> &str {
        "context_window"
    }

    /// Determine whether the module should be shown for the given context and configuration.
    ///
    /// Returns `true` if the context contains context window data and the `ContextWindowConfig` is not disabled, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::modules::context_window::ContextWindowModule;
    /// use crate::types::config::ContextWindowConfig;
    /// use crate::types::context::Context;
    ///
    /// let module = ContextWindowModule::new();
    /// let ctx = Context::default();
    /// let cfg = ContextWindowConfig::default();
    /// // With a default context that has no context_window data this returns false.
    /// assert!(!module.should_display(&ctx, &cfg));
    /// ```
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

    /// Renders the context window usage as a styled percentage string.
    ///
    /// If the context contains a context window and a percentage can be calculated, this returns
    /// the configured formatted string with tokens `percentage` and `symbol`, using dynamic color
    /// tiers when enabled. If the context has no usable context window data, an empty string is
    /// returned. If the provided `config` cannot be downcast to `ContextWindowConfig`, a simple
    /// fallback string in the form `"[ctx {percentage}%]"` is returned.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Illustrative example; types like `Context` and `ContextWindowConfig` are assumed to be in scope.
    /// let module = ContextWindowModule::new();
    /// // When context lacks context_window data, the module returns an empty string.
    /// let empty = module.render(&Context::default(), &SomeOtherConfig {});
    /// assert_eq!(empty, "");
    ///
    /// // When a context and config are provided and downcasting succeeds, the output contains the percentage.
    /// // let output = module.render(&populated_context, &context_window_config);
    /// // assert!(output.contains("25")); // e.g., "25" appears in the rendered string
    /// ```
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

            // Determine style based on percentage
            let style = if cfg.use_dynamic_color {
                Self::get_style_for_percentage(percentage, cfg)
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
    use crate::types::claude::{ClaudeInput, ContextWindow, ModelInfo};
    use crate::types::context::Context;
    use rstest::*;

    /// Creates a test `Context` containing a `ContextWindow` with the specified
    /// total input tokens, total output tokens, and context window size.
    ///
    /// # Examples
    ///
    /// ```
    /// let ctx = context_with_usage(100, 50, 400);
    /// assert_eq!(ctx.input.context_window.unwrap().total_input_tokens, 100);
    /// assert_eq!(ctx.input.context_window.unwrap().total_output_tokens, 50);
    /// assert_eq!(ctx.input.context_window.unwrap().context_window_size, 400);
    /// ```
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
    fn test_style_for_percentage(#[case] percentage: u64, #[case] expected_style: &str) {
        use crate::types::config::ContextWindowConfig;
        let config = ContextWindowConfig::default();
        let style = ContextWindowModule::get_style_for_percentage(percentage, &config);
        assert_eq!(style, expected_style);
    }

    #[rstest]
    #[case(30, "bold green")]
    #[case(50, "bold yellow")]
    #[case(80, "bold red")]
    fn test_custom_threshold_styles(#[case] percentage: u64, #[case] expected_style: &str) {
        use crate::types::config::ContextWindowConfig;
        let mut config = ContextWindowConfig::default();
        config.style_low = "bold green".to_string();
        config.style_medium = "bold yellow".to_string();
        config.style_high = "bold red".to_string();

        let style = ContextWindowModule::get_style_for_percentage(percentage, &config);
        assert_eq!(style, expected_style);
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