use rstest::*;
use beacon::config::Config;
use beacon::types::claude::ClaudeInput;
use beacon::types::context::Context;
use crate::common::builders::{ClaudeInputBuilder, ContextBuilder};

/// Default test configuration fixture
#[fixture]
pub fn default_config() -> Config {
    Config::default()
}

/// Default ClaudeInput fixture
#[fixture]
pub fn default_claude_input() -> ClaudeInput {
    ClaudeInputBuilder::new().build()
}

/// Default Context fixture
#[fixture]
pub fn default_context() -> Context {
    ContextBuilder::new().build()
}

/// Test paths fixture - provides common test paths
#[fixture]
pub fn test_paths() -> TestPaths {
    TestPaths {
        home: "/Users/test".to_string(),
        project: "/Users/test/projects/beacon".to_string(),
        deep_nested: "/Users/test/very/deep/nested/directory/structure/project".to_string(),
    }
}

pub struct TestPaths {
    pub home: String,
    pub project: String,
    pub deep_nested: String,
}

/// TestRenderer for module testing - Starship-inspired pattern
pub struct TestRenderer {
    context: Context,
}

impl TestRenderer {
    pub fn new() -> Self {
        Self {
            context: ContextBuilder::new().build(),
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = context;
        self
    }

    pub fn with_cwd(mut self, cwd: &str) -> Self {
        let new_input = ClaudeInputBuilder::new().with_cwd(cwd).build();
        self.context = Context::new(new_input, self.context.config.clone());
        self
    }

    pub fn with_model(mut self, display_name: &str) -> Self {
        let current_dir = self.context.current_dir.to_string_lossy().to_string();
        let new_input = ClaudeInputBuilder::new()
            .with_cwd(&current_dir)
            .with_model(display_name)
            .build();
        self.context = Context::new(new_input, self.context.config.clone());
        self
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Render a module and return its output
    pub fn render<M: beacon::modules::Module>(&self, module: &M) -> String {
        // For testing, we use EmptyConfig as default
        module.render(&self.context, &beacon::modules::EmptyConfig)
    }
}

/// Fixture for TestRenderer
#[fixture]
pub fn test_renderer() -> TestRenderer {
    TestRenderer::new()
}

/// Parameterized fixture for different model types
#[fixture]
#[once]
pub fn model_names() -> Vec<&'static str> {
    vec!["Opus", "Sonnet", "Haiku", "Claude-3.5"]
}

/// Parameterized fixture for different directory paths  
#[fixture]
#[once]
pub fn test_directories() -> Vec<&'static str> {
    vec![
        "/home/user/project",
        "/Users/test/Documents/code",
        "/var/www/html",
        "C:\\Users\\test\\project",
    ]
}