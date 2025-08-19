use crate::types::ClaudeInput;

/// Parse JSON string into ClaudeInput
pub fn parse_claude_input(json_str: &str) -> Result<ClaudeInput, serde_json::Error> {
    serde_json::from_str(json_str)
}

#[cfg(test)]
mod tests {
    use super::*;

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