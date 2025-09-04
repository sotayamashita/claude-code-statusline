use crate::debug::DebugLogger;
use crate::timeout::run_with_timeout;
use crate::types::context::Context;
use std::any::Any;
use std::time::Duration;

/// Trait for module-specific configuration
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
pub struct EmptyConfig;

impl ModuleConfig for EmptyConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait that all status line modules must implement
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
pub mod git_branch;
pub mod git_status;

pub use claude_model::ClaudeModelModule;
pub use directory::DirectoryModule;
use git_branch::GitBranchModule;
use git_status::GitStatusModule;

/// Central module dispatcher - creates module instances based on name
/// This implements the Factory pattern for dynamic module creation
pub fn handle_module(name: &str, context: &Context) -> Option<Box<dyn Module>> {
    match name {
        "directory" => Some(Box::new(DirectoryModule::from_context(context))),
        "claude_model" => Some(Box::new(ClaudeModelModule::from_context(context))),
        "git_branch" => Some(Box::new(GitBranchModule::from_context(context))),
        "git_status" => Some(Box::new(GitStatusModule::from_context(context))),
        _ => None,
    }
}

fn module_config_for<'a>(name: &str, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
    match name {
        "directory" => Some(&context.config.directory),
        "claude_model" => Some(&context.config.claude_model),
        "git_branch" => Some(&context.config.git_branch),
        "git_status" => Some(&context.config.git_status),
        _ => None,
    }
}

/// Render a module with a global timeout based on `Config.command_timeout`.
/// - Returns Some(output) on success
/// - Returns None on timeout, error, or when not displayed
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
            let module =
                handle_module(&name1, &ctx1).ok_or_else(|| anyhow::anyhow!("unknown module"))?;
            let cfg =
                module_config_for(&name1, &ctx1).ok_or_else(|| anyhow::anyhow!("no config"))?;
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
            let module =
                handle_module(&name2, &ctx2).ok_or_else(|| anyhow::anyhow!("unknown module"))?;
            let cfg =
                module_config_for(&name2, &ctx2).ok_or_else(|| anyhow::anyhow!("no config"))?;
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
