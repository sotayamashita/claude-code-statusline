#![allow(dead_code)]
use assert_cmd::Command;
use std::fs;
use std::path::Path;

pub fn ccs_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

pub fn write_basic_config(home: &Path, command_timeout: Option<u64>) {
    let cfg_dir = home.join(".config");
    fs::create_dir_all(&cfg_dir).unwrap();
    let mut toml = String::from(
        r#"
format = "$directory $git_branch $git_status $claude_model"

[git_branch]
format = "[🌿 $branch]($style)"
style = "bold green"

[git_status]
format = "([[$all_status$ahead_behind]]($style) )"
style = "bold red"

[claude_model]
format = "[$symbol$model]($style)"
style = "bold yellow"
"#,
    );
    if let Some(ms) = command_timeout {
        toml = format!("command_timeout = {ms}\n{toml}");
    }
    fs::write(cfg_dir.join("claude-code-statusline.toml"), toml).unwrap();
}

pub fn input_json_with_cwd(cwd: &str) -> String {
    format!(
        r#"{{
  "hook_event_name": "Status",
  "session_id": "s-123",
  "transcript_path": null,
  "cwd": "{cwd}",
  "model": {{"id": "claude-opus", "display_name": "Opus"}},
  "workspace": {{"current_dir": "{cwd}", "project_dir": "{cwd}"}},
  "version": "1.0.0",
  "output_style": null
}}"#
    )
}
