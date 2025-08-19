use serde::{Deserialize, Serialize};

// Claude Code JSON input structures
// Reference: https://docs.anthropic.com/en/docs/claude-code/statusline#json-input-structure
// Complete structure from official documentation

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClaudeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_event_name: Option<String>,
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub model: ModelInfo,
    pub workspace: WorkspaceInfo,
    pub version: String,
    pub output_style: OutputStyle,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceInfo {
    pub current_dir: String,
    pub project_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputStyle {
    pub name: String,
}
