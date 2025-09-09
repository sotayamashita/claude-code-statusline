use claude_code_statusline_core::config::Config;
use claude_code_statusline_core::types::claude::{
    ClaudeInput, ModelInfo, OutputStyle, WorkspaceInfo,
};
use claude_code_statusline_core::types::context::Context;

/// Builder for creating test ClaudeInput instances
pub struct ClaudeInputBuilder {
    hook_event_name: Option<String>,
    session_id: String,
    transcript_path: Option<String>,
    cwd: String,
    model: ModelInfo,
    workspace: Option<WorkspaceInfo>,
    version: Option<String>,
    output_style: Option<OutputStyle>,
}

#[allow(dead_code)]
impl ClaudeInputBuilder {
    pub fn new() -> Self {
        Self {
            hook_event_name: None,
            session_id: "test-session".to_string(),
            transcript_path: None,
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/test/dir".to_string(),
                project_dir: Some("/test".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        }
    }

    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.cwd = cwd.to_string();
        if let Some(ref mut workspace) = self.workspace {
            workspace.current_dir = cwd.to_string();
        }
        self
    }

    pub fn with_model(mut self, display_name: &str) -> Self {
        self.model.display_name = display_name.to_string();
        self.model.id = format!("claude-{}", display_name.to_lowercase());
        self
    }

    pub fn with_model_id(mut self, id: &str, display_name: &str) -> Self {
        self.model.id = id.to_string();
        self.model.display_name = display_name.to_string();
        self
    }

    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = session_id.to_string();
        self
    }

    pub fn with_workspace(mut self, current_dir: &str, project_dir: &str) -> Self {
        self.workspace = Some(WorkspaceInfo {
            current_dir: current_dir.to_string(),
            project_dir: Some(project_dir.to_string()),
        });
        self
    }

    pub fn without_workspace(mut self) -> Self {
        self.workspace = None;
        self
    }

    pub fn build(self) -> ClaudeInput {
        ClaudeInput {
            hook_event_name: self.hook_event_name,
            session_id: self.session_id,
            transcript_path: self.transcript_path,
            cwd: self.cwd,
            model: self.model,
            workspace: self.workspace,
            version: self.version,
            output_style: self.output_style,
        }
    }
}

impl Default for ClaudeInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating test Context instances
pub struct ContextBuilder {
    input: ClaudeInputBuilder,
    config: Config,
}

#[allow(dead_code)]
impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            input: ClaudeInputBuilder::new(),
            config: Config::default(),
        }
    }

    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.input = self.input.with_cwd(cwd);
        self
    }

    pub fn with_model(mut self, display_name: &str) -> Self {
        self.input = self.input.with_model(display_name);
        self
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn with_directory_config<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut claude_code_statusline_core::types::config::DirectoryConfig),
    {
        f(&mut self.config.directory);
        self
    }

    pub fn with_claude_model_config<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut claude_code_statusline_core::types::config::ClaudeModelConfig),
    {
        f(&mut self.config.claude_model);
        self
    }

    pub fn build(self) -> Context {
        Context::new(self.input.build(), self.config)
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
