//! Module registry and factory system
//!
//! Provides `Registry` and `ModuleFactory` to create modules dynamically
//! without hard-coded dispatcher matches. This enables pluggable modules
//! and paves the way for external/extra modules via configuration.

use super::{Module, ModuleConfig, claude_model::ClaudeModelModule, directory::DirectoryModule};
#[cfg(feature = "git")]
use super::{git_branch::GitBranchModule, git_status::GitStatusModule};
use crate::types::context::Context;

/// Factory trait for constructing modules and exposing their config binding
pub trait ModuleFactory: Send + Sync {
    /// Canonical module name (e.g., "directory")
    fn name(&self) -> &'static str;

    /// Create a fresh module instance for the given context
    fn create(&self, context: &Context) -> Box<dyn Module>;

    /// Obtain the module-specific config view from Context
    fn config<'a>(&self, context: &'a Context) -> Option<&'a dyn ModuleConfig>;
}

/// Simple in-memory registry of module factories
pub struct Registry {
    factories: Vec<Box<dyn ModuleFactory>>,
}

impl Registry {
    /// Empty registry
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    /// Default registry with built-in modules
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        reg.register_factory(DirectoryFactory);
        reg.register_factory(ClaudeModelFactory);
        #[cfg(feature = "git")]
        {
            reg.register_factory(GitBranchFactory);
            reg.register_factory(GitStatusFactory);
        }
        reg
    }

    /// Register a factory
    pub fn register_factory<F: ModuleFactory + 'static>(&mut self, f: F) {
        self.factories.push(Box::new(f));
    }

    /// Create a module by name
    pub fn create(&self, name: &str, context: &Context) -> Option<Box<dyn Module>> {
        self.factories
            .iter()
            .find(|f| f.name() == name)
            .map(|f| f.create(context))
    }

    /// Get module config by name
    pub fn config<'a>(&self, name: &str, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
        self.factories
            .iter()
            .find(|f| f.name() == name)
            .and_then(|f| f.config(context))
    }

    /// List registered module names
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&'static str> {
        self.factories.iter().map(|f| f.name()).collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in factories

struct DirectoryFactory;
impl ModuleFactory for DirectoryFactory {
    fn name(&self) -> &'static str {
        "directory"
    }
    fn create(&self, context: &Context) -> Box<dyn Module> {
        Box::new(DirectoryModule::from_context(context))
    }
    fn config<'a>(&self, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
        Some(&context.config.directory)
    }
}

struct ClaudeModelFactory;
impl ModuleFactory for ClaudeModelFactory {
    fn name(&self) -> &'static str {
        "claude_model"
    }
    fn create(&self, context: &Context) -> Box<dyn Module> {
        Box::new(ClaudeModelModule::from_context(context))
    }
    fn config<'a>(&self, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
        Some(&context.config.claude_model)
    }
}

#[cfg(feature = "git")]
struct GitBranchFactory;
#[cfg(feature = "git")]
impl ModuleFactory for GitBranchFactory {
    fn name(&self) -> &'static str {
        "git_branch"
    }
    fn create(&self, context: &Context) -> Box<dyn Module> {
        Box::new(GitBranchModule::from_context(context))
    }
    fn config<'a>(&self, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
        Some(&context.config.git_branch)
    }
}

#[cfg(feature = "git")]
struct GitStatusFactory;
#[cfg(feature = "git")]
impl ModuleFactory for GitStatusFactory {
    fn name(&self) -> &'static str {
        "git_status"
    }
    fn create(&self, context: &Context) -> Box<dyn Module> {
        Box::new(GitStatusModule::from_context(context))
    }
    fn config<'a>(&self, context: &'a Context) -> Option<&'a dyn ModuleConfig> {
        Some(&context.config.git_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ClaudeInput, ModelInfo};
    use crate::types::context::Context;

    #[test]
    fn default_registry_lists_core_modules() {
        let reg = Registry::with_defaults();
        let names = reg.list();
        assert!(names.contains(&"directory"));
        assert!(names.contains(&"claude_model"));
        #[cfg(feature = "git")]
        {
            assert!(names.contains(&"git_branch"));
            assert!(names.contains(&"git_status"));
        }
    }

    #[test]
    fn create_and_config_work_for_known_modules() {
        let cfg = Config::default();
        let input = ClaudeInput {
            hook_event_name: None,
            session_id: "s".into(),
            transcript_path: None,
            cwd: "/tmp".into(),
            model: ModelInfo {
                id: "claude-opus".into(),
                display_name: "Opus".into(),
            },
            workspace: None,
            version: None,
            output_style: None,
        };
        let ctx = Context::new(input, cfg);
        let reg = Registry::with_defaults();
        let m = reg.create("directory", &ctx).expect("module");
        assert_eq!(m.name(), "directory");
        assert!(reg.config("directory", &ctx).is_some());
        assert!(reg.create("unknown", &ctx).is_none());
    }
}
