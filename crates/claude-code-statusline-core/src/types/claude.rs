//! Claude Code JSON input type definitions
//!
//! This module defines the structures for parsing JSON input from Claude Code.
//! These types match the official Claude Code API specification.
//!
//! Reference: <https://docs.anthropic.com/en/docs/claude-code/statusline#json-input-structure>

use serde::{Deserialize, Serialize};

/// Main input structure from Claude Code
///
/// Contains all the information passed from Claude Code to the status line,
/// including session details, current directory, model information, and
/// optional workspace context.
///
/// # Example JSON
///
/// ```json
/// {
///     "session_id": "abc123",
///     "cwd": "/home/user/project",
///     "model": {
///         "id": "claude-3.5-sonnet",
///         "display_name": "Sonnet"
///     },
///     "workspace": {
///         "current_dir": "/home/user/project/src",
///         "project_dir": "/home/user/project"
///     }
/// }
/// ```
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<ContextWindow>,
}

/// Information about the current Claude model
///
/// Contains the model identifier and a human-readable display name.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelInfo {
    /// Model identifier (e.g., "claude-3.5-sonnet")
    pub id: String,
    /// Human-readable model name for display (e.g., "Sonnet")
    pub display_name: String,
}

/// Workspace context information
///
/// Provides details about the current workspace, including the
/// current working directory and optional project root.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceInfo {
    /// Current directory within the workspace
    pub current_dir: String,
    /// Optional project root directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<String>,
}

/// Output style configuration
///
/// Defines the output style to be used by Claude Code.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputStyle {
    /// Name of the output style
    pub name: String,
}

/// Context window usage information
///
/// Contains information about token usage and context window size.
/// Matches the structure sent by Claude Code.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContextWindow {
    /// Total input tokens used
    pub total_input_tokens: u64,
    /// Total output tokens generated
    pub total_output_tokens: u64,
    /// Total context window size
    pub context_window_size: u64,
    /// Detailed current usage breakdown (may be null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_usage: Option<CurrentUsage>,
}

/// Breakdown of current token usage (when available)
///
/// Tracks different types of tokens being used in the current context.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrentUsage {
    /// Regular input tokens
    pub input_tokens: u64,
    /// Tokens used for cache creation
    pub cache_creation_input_tokens: u64,
    /// Tokens read from cache
    pub cache_read_input_tokens: u64,
}
