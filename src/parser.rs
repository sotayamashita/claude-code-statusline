use crate::types::claude::ClaudeInput;
use crate::types::context::Context;
use anyhow::Result;
use std::collections::HashMap;

/// Parse JSON string into ClaudeInput
pub fn parse_claude_input(json_str: &str) -> Result<ClaudeInput> {
    Ok(serde_json::from_str(json_str)?)
}

/// Parse format string and replace variables with module outputs
///
/// Example:
/// - Input: format = "$directory $claude_model", module_outputs = {"directory": "~/project", "claude_model": "Opus"}
/// - Output: "~/project Opus"
pub fn parse_format(
    format: &str,
    _context: &Context,
    module_outputs: &HashMap<String, String>,
) -> String {
    let mut result = format.to_string();

    // Replace each variable with its corresponding output
    for (module_name, output) in module_outputs {
        let variable = format!("${module_name}");
        result = result.replace(&variable, output);
    }

    // Remove any remaining variables that don't have outputs
    let remaining_vars: Vec<String> = result
        .split_whitespace()
        .filter(|s| s.starts_with('$'))
        .map(|s| s.to_string())
        .collect();

    for var in remaining_vars {
        result = result.replace(&var, "");
    }

    // Clean up multiple spaces
    result = result.split_whitespace().collect::<Vec<_>>().join(" ");

    result
}

/// Extract module names from format string
///
/// Example:
/// - Input: "$directory $claude_model $character"
/// - Output: ["directory", "claude_model", "character"]
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
