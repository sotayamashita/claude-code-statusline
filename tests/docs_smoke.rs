use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn ccs_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

fn valid_input_json() -> String {
    r#"{
        "hook_event_name": "Status",
        "session_id": "s-123",
        "transcript_path": null,
        "cwd": "/tmp",
        "model": {"id": "claude-opus", "display_name": "Opus"},
        "workspace": {"current_dir": "/tmp", "project_dir": "/tmp"},
        "version": "1.0.0",
        "output_style": null
    }"#
    .to_string()
}

#[test]
fn docs_format_example_runs_without_error() {
    // Prepare a temp HOME with docs example format including $claude_session (not implemented)
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let cfg_dir = home.join(".config");
    fs::create_dir_all(&cfg_dir).unwrap();
    let toml = r#"
        format = "$directory $git_branch $claude_model $claude_session"

        [git_branch]
        format = "[🌿 $branch]($style)"
        style = "bold green"

        [claude_session]
        format = "[🔗 $short_id]($style)"
        style = "italic yellow"

        [claude_model]
        format = "[$symbol$model]($style)"
        style = "bold yellow"
    "#;
    fs::write(cfg_dir.join("claude-code-statusline.toml"), toml).unwrap();

    let mut cmd = ccs_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(valid_input_json());
    // Should succeed; stdout should contain model name and not contain error text
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Opus"))
        .stdout(predicate::str::is_empty().not())
        .stdout(predicate::str::contains("Failed").not());
}
