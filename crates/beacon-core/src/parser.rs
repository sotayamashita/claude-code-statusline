//! JSON parsing and format string processing utilities
//!
//! This module provides functions for:
//! - Parsing JSON input from Claude Code
//! - Processing format strings with variable substitution
//! - Extracting module names from format strings
//!
//! # Format String Syntax
//!
//! Format strings use `$` prefix for variables that correspond to module names:
//! - `$directory` - Replaced with directory module output
//! - `$claude_model` - Replaced with model information
//! - `$git_branch` - Replaced with current git branch
//!
//! Example: `"$directory $git_branch $claude_model"`

use crate::error::CoreError;
use crate::types::claude::ClaudeInput;
use crate::types::context::Context;
use std::collections::HashMap;

/// Parses JSON string into ClaudeInput structure
///
/// Takes raw JSON input from stdin and deserializes it into
/// a strongly typed ClaudeInput struct.
///
/// # Arguments
///
/// * `json_str` - Raw JSON string to parse
///
/// # Returns
///
/// * `Ok(ClaudeInput)` - Successfully parsed input
/// * `Err` - JSON parsing or validation failed
///
/// # Examples
///
/// ```
/// use beacon::parse_claude_input;
///
/// let json = r#"{"session_id":"test","cwd":"/tmp","model":{"id":"claude","display_name":"Claude"}}"#;
/// let input = parse_claude_input(json).unwrap();
/// assert_eq!(input.cwd, "/tmp");
/// ```
pub fn parse_claude_input(json_str: &str) -> Result<ClaudeInput, CoreError> {
    Ok(serde_json::from_str(json_str)?)
}

/// Parses format string and substitutes variables with module outputs
///
/// Processes a format string by replacing variable placeholders (e.g., `$directory`)
/// with their corresponding values from the module outputs map.
///
/// # Arguments
///
/// * `format` - Format string containing variable placeholders
/// * `_context` - Context (currently unused, reserved for future use)
/// * `module_outputs` - Map of module names to their rendered outputs
///
/// # Returns
///
/// A string with all variables replaced by their values
///
/// # Examples
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use beacon::parser::parse_format;
/// # use beacon::{Context, Config, parse_claude_input};
/// # let json = r#"{"session_id":"test","cwd":"/tmp","model":{"id":"claude","display_name":"Claude"}}"#;
/// # let input = parse_claude_input(json).unwrap();
/// # let context = Context::new(input, Config::default());
/// let mut outputs = HashMap::new();
/// outputs.insert("directory".to_string(), "~/project".to_string());
/// outputs.insert("claude_model".to_string(), "Opus".to_string());
///
/// let result = parse_format("$directory $claude_model", &context, &outputs);
/// assert_eq!(result, "~/project Opus");
/// ```
pub fn parse_format(
    format: &str,
    _context: &Context,
    module_outputs: &HashMap<String, String>,
) -> String {
    // Process the format string token by token to handle variables correctly
    let tokens: Vec<String> = format
        .split_whitespace()
        .map(|token| {
            if token.starts_with('$') && token.len() > 1 {
                let module_name = &token[1..];
                module_outputs.get(module_name).cloned().unwrap_or_default()
            } else {
                token.to_string()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    tokens.join(" ")
}

/// Extracts module names from a format string
///
/// Scans the format string for variable placeholders (tokens starting with `$`)
/// and returns a list of module names that need to be rendered.
///
/// # Arguments
///
/// * `format` - Format string containing variable placeholders
///
/// # Returns
///
/// A vector of module names found in the format string
///
/// # Examples
///
/// ```
/// use beacon::parser::extract_modules_from_format;
///
/// let modules = extract_modules_from_format("$directory $claude_model");
/// assert_eq!(modules, vec!["directory", "claude_model"]);
/// ```
pub fn extract_modules_from_format(format: &str) -> Vec<String> {
    format
        .split_whitespace()
        .filter_map(|part| {
            if part.starts_with('$') && part.len() > 1 {
                Some(part[1..].to_string())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::types::claude::{ModelInfo, WorkspaceInfo};

    #[test]
    fn test_extract_modules_from_format() {
        let format = "$directory $claude_model $character";
        let modules = extract_modules_from_format(format);
        assert_eq!(modules, vec!["directory", "claude_model", "character"]);
    }

    #[test]
    fn test_extract_modules_from_format_with_extra_text() {
        let format = "prefix $directory middle $claude_model suffix";
        let modules = extract_modules_from_format(format);
        assert_eq!(modules, vec!["directory", "claude_model"]);
    }

    #[test]
    fn test_extract_modules_from_format_empty() {
        let format = "no variables here";
        let modules = extract_modules_from_format(format);
        assert_eq!(modules, Vec::<String>::new());
    }

    #[test]
    fn test_parse_format() {
        let input = ClaudeInput {
            hook_event_name: Some("Status".to_string()),
            session_id: "test-123".to_string(),
            transcript_path: Some("/test/transcript.json".to_string()),
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: Some(WorkspaceInfo {
                current_dir: "/test/dir".to_string(),
                project_dir: Some("/test/project".to_string()),
            }),
            version: Some("1.0.0".to_string()),
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        let mut module_outputs = HashMap::new();
        module_outputs.insert("directory".to_string(), "~/project".to_string());
        module_outputs.insert("claude_model".to_string(), "Opus".to_string());

        let format = "$directory $claude_model $character";
        let result = parse_format(format, &context, &module_outputs);

        // $character doesn't have output, so it should be removed
        assert_eq!(result, "~/project Opus");
    }

    #[test]
    fn test_parse_format_with_text() {
        let input = ClaudeInput {
            hook_event_name: Some("Status".to_string()),
            session_id: "test-123".to_string(),
            transcript_path: Some("/test/transcript.json".to_string()),
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: None,
            version: Some("1.0.0".to_string()),
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        let mut module_outputs = HashMap::new();
        module_outputs.insert("directory".to_string(), "~/project".to_string());
        module_outputs.insert("character".to_string(), ">".to_string());

        let format = "prefix $directory middle $character suffix";
        let result = parse_format(format, &context, &module_outputs);

        assert_eq!(result, "prefix ~/project middle > suffix");
    }

    #[test]
    fn test_parse_format_edge_cases() {
        let input = ClaudeInput {
            hook_event_name: Some("Status".to_string()),
            session_id: "test-123".to_string(),
            transcript_path: Some("/test/transcript.json".to_string()),
            cwd: "/test/dir".to_string(),
            model: ModelInfo {
                id: "claude-opus".to_string(),
                display_name: "Opus".to_string(),
            },
            workspace: None,
            version: Some("1.0.0".to_string()),
            output_style: None,
        };

        let config = Config::default();
        let context = Context::new(input, config);

        // Test with substring variable names
        let mut module_outputs = HashMap::new();
        module_outputs.insert("dir".to_string(), "short".to_string());
        module_outputs.insert("directory".to_string(), "long".to_string());

        // Should handle variable names correctly even when one is a substring of another
        let format = "$directory $dir";
        let result = parse_format(format, &context, &module_outputs);
        assert_eq!(result, "long short");

        // Test with variables without whitespace boundaries
        let format = "prefix$directory suffix";
        let result = parse_format(format, &context, &module_outputs);
        assert_eq!(result, "prefix$directory suffix");
    }

    #[test]
    fn test_parse_valid_claude_input() {
        let json_str = r#"{
            "hook_event_name": "Status",
            "session_id": "test-session-123",
            "transcript_path": "/path/to/transcript.json",
            "cwd": "/test/directory",
            "model": {
                "id": "claude-opus-4-1",
                "display_name": "Opus"
            },
            "workspace": {
                "current_dir": "/test/directory",
                "project_dir": "/test/project"
            },
            "version": "1.0.0",
            "output_style": {
                "name": "default"
            }
        }"#;

        let result = parse_claude_input(json_str);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.session_id, "test-session-123");
        assert_eq!(input.model.display_name, "Opus");
        assert_eq!(input.cwd, "/test/directory");
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_json = "not a valid json";
        let result = parse_claude_input(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_field() {
        // Missing "model" field
        let json_str = r#"{
            "hook_event_name": "Status",
            "session_id": "test-session-123",
            "transcript_path": "/path/to/transcript.json",
            "cwd": "/test/directory",
            "workspace": {
                "current_dir": "/test/directory",
                "project_dir": "/test/project"
            },
            "version": "1.0.0",
            "output_style": {
                "name": "default"
            }
        }"#;

        let result = parse_claude_input(json_str);
        assert!(result.is_err());
    }
}
