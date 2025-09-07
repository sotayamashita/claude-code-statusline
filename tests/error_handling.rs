use assert_cmd::Command;
use predicates::prelude::*;
use std::env;
use std::fs;

fn ccs_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

fn valid_input_json() -> String {
    // minimal valid JSON based on types
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
fn invalid_json_produces_concise_stdout_and_stderr_details() {
    // Use an isolated HOME so that a user's invalid ~/.config/claude-code-statusline.toml
    // does not interfere with this test.
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let mut cmd = ccs_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin("this is not json");
    cmd.assert()
        .stdout(predicate::str::contains(
            "Failed to build status line due to invalid json",
        ))
        .stderr(predicate::str::contains("Failed to parse JSON"));
}

#[test]
fn invalid_toml_config_produces_concise_stdout_and_stderr_details() {
    // Prepare a temp HOME with invalid config
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let cfg_dir = home.join(".config");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("claude-code-statusline.toml"),
        "this is not = toml",
    )
    .unwrap();

    let mut cmd = ccs_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(valid_input_json());
    cmd.assert()
        .stdout(predicate::str::contains(
            "Failed to build status line due to invalid config",
        ))
        .stderr(predicate::str::contains("Config error"));
}
