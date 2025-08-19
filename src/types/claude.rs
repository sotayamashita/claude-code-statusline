use serde::{Deserialize, Serialize};

// Claude Code JSON input structures
// Reference: https://docs.anthropic.com/en/docs/claude-code/statusline#json-input-structure
// Complete structure from official documentation

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClaudeInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_event_name: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    pub cwd: String,
    pub model: ModelInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<OutputStyle>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceInfo {
    pub current_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputStyle {
    pub name: String,
}
