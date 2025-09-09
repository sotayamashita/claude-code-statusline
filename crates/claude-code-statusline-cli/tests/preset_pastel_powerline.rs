use predicates::prelude::*;
use std::fs;
use test_support::cli::{ccs_cmd_with_home, config_dir_for_home};

fn valid_input_json() -> String {
    r#"{
        "hook_event_name": "Status",
        "session_id": "s-pp-1",
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
fn pastel_powerline_minimal_directory_and_model() {
    // Use module-level formats with literal styles to ensure bracket specs render
    // and do not leak as plain text.
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let cfg_dir = config_dir_for_home(home);
    fs::create_dir_all(&cfg_dir).unwrap();
    let toml = r#"
        format = "$directory $claude_model"

        [directory]
        # bridge to the model segment color
        format = "[$path ](fg:black bg:#a8d8ef)[](fg:#a8d8ef bg:#e4bee6)"
        style = ""

        [claude_model]
        format = "[$symbol$model](fg:black bg:#e4bee6)"
        style = ""
        symbol = "<"
    "#;
    fs::write(cfg_dir.join("claude-code-statusline.toml"), toml).unwrap();

    let mut cmd = ccs_cmd_with_home(home);
    cmd.write_stdin(valid_input_json());
    cmd.assert()
        .success()
        // Must not leak bracket-style markup literals like `](fg:...`
        .stdout(predicate::str::contains("](").not())
        // Should contain model name (ANSI present)
        .stdout(predicate::str::contains("Opus"));
}
