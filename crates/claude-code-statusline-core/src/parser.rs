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
/// use claude_code_statusline_core::parse_claude_input;
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
/// Replaces `$<name>` tokens anywhere in the string (not only when
/// separated by whitespace) with their corresponding rendered outputs.
/// Unknown tokens are removed (replaced by an empty string).
///
/// # Arguments
///
/// * `format` - Format string containing `$<name>` variable tokens
/// * `_context` - Context (reserved for future use)
/// * `module_outputs` - Map of module names to their rendered outputs
///
/// # Returns
///
/// A string with all `$<name>` tokens replaced by their values while
/// preserving all other characters (including spaces) verbatim.
///
/// # Examples
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use claude_code_statusline_core::parser::parse_format;
/// # use claude_code_statusline_core::{Context, Config, parse_claude_input};
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
    // Scan the string and replace $<name> inline without altering
    // any other characters. A valid name starts with [A-Za-z_]
    // and continues with [A-Za-z0-9_]*.
    let bytes = format.as_bytes();
    let mut i = 0;
    let mut out = String::with_capacity(format.len());
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            let j = i + 1;
            // validate first identifier char
            let mut k = j;
            if k < bytes.len() {
                let c = bytes[k] as char;
                if c.is_ascii_alphabetic() || c == '_' {
                    k += 1;
                    // consume rest of identifier
                    while k < bytes.len() {
                        let c2 = bytes[k] as char;
                        if c2.is_ascii_alphanumeric() || c2 == '_' {
                            k += 1;
                        } else {
                            break;
                        }
                    }
                    let name = &format[j..k];
                    // Replace with module output (or empty string if missing)
                    if let Some(val) = module_outputs.get(name) {
                        out.push_str(val);
                    }
                    i = k;
                    continue;
                }
            }
            // Not a valid token — treat '$' literally
            out.push('$');
            i = start + 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    // Avoid a dangling trailing space when unknown tokens are removed
    // at the end (e.g., "$directory $character"). Do not alter
    // interior whitespace to preserve precise layout for Powerline-style
    // compositions.
    out.trim_end().to_string()
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
/// use claude_code_statusline_core::parser::extract_modules_from_format;
///
/// let modules = extract_modules_from_format("$directory $claude_model");
/// assert_eq!(modules, vec!["directory", "claude_model"]);
/// ```
pub fn extract_modules_from_format(format: &str) -> Vec<String> {
    // Scan for `$<name>` anywhere in the string and return unique names
    // in encounter order.
    use std::collections::HashSet;
    let bytes = format.as_bytes();
    let mut i = 0;
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let j = i + 1;
            let mut k = j;
            if k < bytes.len() {
                let c = bytes[k] as char;
                if c.is_ascii_alphabetic() || c == '_' {
                    k += 1;
                    while k < bytes.len() {
                        let c2 = bytes[k] as char;
                        if c2.is_ascii_alphanumeric() || c2 == '_' {
                            k += 1;
                        } else {
                            break;
                        }
                    }
                    let name = &format[j..k];
                    if seen.insert(name.to_string()) {
                        out.push(name.to_string());
                    }
                    i = k;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
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

        // Variables without whitespace boundaries must also be replaced now
        let format = "prefix$directory suffix";
        let result = parse_format(format, &context, &module_outputs);
        assert_eq!(result, "prefixlong suffix");
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
