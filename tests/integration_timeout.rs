use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn beacon_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

#[test]
fn one_line_output_with_small_timeout() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let cfg_dir = home.join(".config");
    fs::create_dir_all(&cfg_dir).unwrap();
    // Set minimal allowed timeout (50ms) and full format
    let toml = r#"
        format = "$directory $git_branch $git_status $claude_model"
        command_timeout = 50

        [git_branch]
        format = "[🌿 $branch]($style)"
        style = "bold green"

        [git_status]
        format = "([[$all_status$ahead_behind]]($style) )"
        style = "bold red"

        [claude_model]
        format = "[$symbol$model]($style)"
        style = "bold yellow"
    "#;
    fs::write(cfg_dir.join("beacon.toml"), toml).unwrap();

    let cwd = home.join("work");
    fs::create_dir_all(&cwd).unwrap();

    let input = format!(
        r#"{{
        "hook_event_name": "Status",
        "session_id": "s-123",
        "transcript_path": null,
        "cwd": "{}",
        "model": {{"id": "claude-opus", "display_name": "Opus"}},
        "workspace": {{"current_dir": "{}", "project_dir": "{}"}},
        "version": "1.0.0",
        "output_style": null
    }}"#,
        cwd.to_str().unwrap(),
        cwd.to_str().unwrap(),
        cwd.to_str().unwrap()
    );

    let mut cmd = beacon_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap());
}
