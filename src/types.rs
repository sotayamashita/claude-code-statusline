use serde::Deserialize;

// Claude Code JSON input structures
// Reference: https://docs.anthropic.com/en/docs/claude-code/statusline#json-input-structure
// Complete structure from official documentation

#[derive(Debug, Deserialize)]
pub struct ClaudeInput {
    pub hook_event_name: String,
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub model: ModelInfo,
    pub workspace: WorkspaceInfo,
    pub version: String,
    pub output_style: OutputStyle,
}

// Model information from Claude Code
#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

// Workspace paths information
#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    pub current_dir: String,
    pub project_dir: String,
}

// Output style configuration
#[derive(Debug, Deserialize)]
pub struct OutputStyle {
    pub name: String,
}