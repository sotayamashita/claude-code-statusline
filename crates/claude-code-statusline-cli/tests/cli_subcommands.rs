use std::fs;
use test_support::common::cli::{
    ccs_cmd, ccs_cmd_with_home, config_dir_for_home, write_basic_config,
};

#[test]
fn config_path_uses_home_and_points_to_new_toml() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    let cfg_dir = config_dir_for_home(home);
    let mut cmd = ccs_cmd_with_home(home);
    cmd.arg("config").arg("--path");
    // Compute expected path using same resolution logic
    let expected = cfg_dir.join("claude-code-statusline.toml");
    let out = cmd.assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert_eq!(s.trim(), expected.display().to_string());
}

#[test]
fn config_default_prints_valid_toml() {
    let mut cmd = ccs_cmd();
    cmd.arg("config").arg("--default");
    let out = cmd.assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    // Should contain some known defaults
    assert!(s.contains("format = \"$directory $claude_model\""));
    assert!(s.contains("command_timeout"));
}

#[test]
fn config_validate_ok_and_invalid() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    // valid config
    write_basic_config(home, Some(100));
    let mut ok = ccs_cmd_with_home(home);
    ok.arg("config").arg("--validate");
    ok.assert()
        .success()
        .stdout(predicates::str::contains("OK"));

    // invalid config (too small timeout)
    let cfg_dir = config_dir_for_home(home);
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("claude-code-statusline.toml"),
        r#"command_timeout = 10
format = "$directory $claude_model"
"#,
    )
    .unwrap();
    let mut bad = ccs_cmd_with_home(home);
    bad.arg("config").arg("--validate");
    bad.assert()
        .success()
        .stdout(predicates::str::contains("INVALID"))
        .stderr(predicates::str::contains("Config validation error"));
}

#[test]
fn modules_list_and_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    // include git modules in config so that --enabled can pick up from format
    write_basic_config(home, None);

    // --list: should contain at least core modules
    let mut list = ccs_cmd_with_home(home);
    list.arg("modules").arg("--list");
    let out = list.assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("directory"));
    assert!(s.contains("claude_model"));
    // CLI enables git feature; expect git modules to be registered
    assert!(s.contains("git_branch"));
    assert!(s.contains("git_status"));

    // --enabled: subset based on format and disabled flags
    let mut enabled = ccs_cmd_with_home(home);
    enabled.arg("modules").arg("--enabled");
    let out2 = enabled.assert().success().get_output().stdout.clone();
    let s2 = String::from_utf8(out2).unwrap();
    assert!(s2.contains("directory"));
    assert!(s2.contains("claude_model"));
}
