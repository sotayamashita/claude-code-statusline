//! Status line module system
//!
//! This module provides the infrastructure for modular status line components.
//! Each module implements the `Module` trait and can be dynamically loaded
//! based on the format string configuration.
//!
//! # Architecture
//!
//! - `Module` trait: Core interface for all status components
//! - `ModuleConfig` trait: Configuration interface for modules
//! - Factory pattern: Dynamic module creation via `handle_module`
//! - Timeout protection: Each module execution is time-bounded
//!
//! # Available Modules
//!
//! - `directory`: Current directory display
//! - `claude_model`: Claude model information
//! - `git_branch`: Current git branch
//! - `git_status`: Git repository status

use crate::debug::DebugLogger;
use crate::timeout::run_with_timeout;
use crate::types::context::Context;
use crate::error::CoreError;
use std::any::Any;
use std::time::Duration;

/// Trait for module-specific configuration
///
/// Each module can have its own configuration section in the TOML file.
/// This trait provides a common interface for accessing module configurations.
pub trait ModuleConfig: Any + Send + Sync {
    /// Allow downcasting to concrete config types
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;

    /// Get the format string for this module
    #[allow(dead_code)]
    fn format(&self) -> &str {
        ""
    }

    /// Get the style string for this module
    #[allow(dead_code)]
    fn style(&self) -> &str {
        ""
    }
}

/// Default implementation for cases where no config is provided
///
/// Used as a fallback when a module doesn't have specific configuration.
pub struct EmptyConfig;

impl ModuleConfig for EmptyConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait that all status line modules must implement
///
/// This is the core interface for creating status line components.
/// Each module determines when to display itself and how to render
/// its output based on the current context.
///
/// # Implementation Notes
///
/// - Modules should be stateless and thread-safe
/// - Heavy operations should be cached in Context
/// - Rendering should complete quickly to avoid timeouts
pub trait Module: Send + Sync {
    /// Returns the name of the module
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Determines if this module should be displayed
    fn should_display(&self, context: &Context, config: &dyn ModuleConfig) -> bool;

    /// Renders the module's output as a string
    fn render(&self, context: &Context, config: &dyn ModuleConfig) -> String;
}

// Re-export module implementations
pub mod claude_model;
pub mod directory;
#[cfg(feature = "git")]
pub mod git_branch;
#[cfg(feature = "git")]
pub mod git_status;
pub mod registry;

pub use claude_model::ClaudeModelModule;
pub use directory::DirectoryModule;
pub use registry::{ModuleFactory, Registry};

/// Central module dispatcher - creates module instances based on name
///
/// Implements the Factory pattern for dynamic module creation.
/// Returns a boxed module instance if the name matches a known module.
///
/// # Arguments
///
/// * `name` - Module name from the format string (e.g., "directory")
/// * `context` - Current execution context
///
/// # Returns
///
/// * `Some(Box<dyn Module>)` - Module instance if name is recognized
/// * `None` - If the module name is unknown
pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    // Gradual migration: delegate to Registry with built-in factories
    let registry = Registry::with_defaults();
    registry.create(name, context)
}

fn module_config_for<'a>(name: &str, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
    let registry = Registry::with_defaults();
    registry.config(name, context)
}

/// Renders a module with timeout protection
///
/// Executes both `should_display` and `render` methods with a timeout
/// based on the configuration's `command_timeout` value. This ensures
/// that slow modules don't block the status line generation.
///
/// # Arguments
///
/// * `name` - Module name to render
/// * `context` - Current execution context
/// * `logger` - Debug logger for error reporting
///
/// # Returns
///
/// * `Some(String)` - Rendered module output on success
/// * `None` - On timeout, error, or when module shouldn't display
///
/// # Timeout Behavior
///
/// If a module exceeds the configured timeout (default 500ms),
/// it will be skipped and an error logged to stderr.
pub fn render_module_with_timeout(
    name: &str,
    context: &Context,
    logger: &DebugLogger,
) -> Option<String> {
    let timeout_ms = context.config.command_timeout;
    let timeout = Duration::from_millis(timeout_ms);

    // should_display with timeout (fresh module instance)
    match run_with_timeout(timeout, {
        let ctx1 = context.clone();
        let name1 = name.to_string();
        move || {
            let module = handle_module(&name1, &ctx1)
                .ok_or_else(|| CoreError::UnknownModule(name1.clone()))?;
            let cfg = module_config_for(&name1, &ctx1)
                .ok_or_else(|| CoreError::MissingConfig(name1.clone()))?;
            Ok(module.should_display(&ctx1, cfg))
        }
    }) {
        Ok(Some(true)) => {}
        Ok(Some(false)) => return None,
        Ok(None) => {
            logger.log_stderr(&format!(
                "Module '{name}' timed out in should_display after {timeout_ms}ms"
            ));
            return None;
        }
        Err(e) => {
            logger.log_stderr(&format!("Module '{name}' error in should_display: {e}"));
            return None;
        }
    }

    // render with timeout (fresh module instance)
    match run_with_timeout(timeout, {
        let ctx2 = context.clone();
        let name2 = name.to_string();
        move || {
            let module = handle_module(&name2, &ctx2)
                .ok_or_else(|| CoreError::UnknownModule(name2.clone()))?;
            let cfg = module_config_for(&name2, &ctx2)
                .ok_or_else(|| CoreError::MissingConfig(name2.clone()))?;
            Ok(module.render(&ctx2, cfg))
        }
    }) {
        Ok(Some(s)) => Some(s),
        Ok(None) => {
            logger.log_stderr(&format!(
                "Module '{name}' timed out in render after {timeout_ms}ms"
            ));
            None
        }
        Err(e) => {
            logger.log_stderr(&format!("Module '{name}' error in render: {e}"));
            None
        }
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo, WorkspaceInfo};

    struct SleepyModule;

    impl SleepyModule {
        fn from_context(_context: &Context) -> Self {
            Self
        }
    }

    impl Module for SleepyModule {
        fn name(&self) -> &str {
            "sleepy"
        }
        fn should_display(&self, _context: &Context, _cfg: &dyn ModuleConfig) -> bool {
            true
        }
        fn render(&self, _context: &Context, _cfg: &dyn ModuleConfig) -> String {
            std::thread::sleep(std::time::Duration::from_millis(200));
            "[SLEEP]".to_string()
        }
    }

    // Extend dispatcher only in tests
    pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
        match name {
            "sleepy" => Some(Box::new(SleepyModule::from_context(context))),
            _ => super::handle_module(name, context),
        }
    }

    fn make_context(cwd: &str, timeout_ms: u64) -> Context {
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: cwd.to_string(),
            model: ModelInfo {
                id: "claude-opus".into(),
                display_name: "Opus".into(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: cwd.to_string(),
                project_dir: Some(cwd.to_string()),
            }),
            version: Some("1.0.0".into()),
            output_style: None,
        };
        let mut cfg = Config::default();
        cfg.command_timeout = timeout_ms;
        Context::new(input, cfg)
    }

    #[test]
    fn sleepy_module_times_out_and_is_omitted() {
        let logger = DebugLogger::new(true);
        let ctx = make_context("/tmp", 50);
        let out = render_module_with_timeout("sleepy", &ctx, &logger);
        assert!(out.is_none());
    }
}
