#![allow(dead_code)]
use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub fn ccs_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Resolve the config directory for a given HOME path using the same
/// logic as the application (dirs::config_dir), without relying on the
/// current process HOME. This is done by temporarily setting HOME while
/// computing the path and restoring it immediately after.
pub fn config_dir_for_home(home: &Path) -> PathBuf {
    let _guard = env_lock().lock().unwrap();
    let orig_home = std::env::var_os("HOME");
    let orig_xdg = std::env::var_os("XDG_CONFIG_HOME");
    // SAFETY: tests are serialized by the lock above
    unsafe {
        std::env::set_var("HOME", home);
        // Ensure dirs::config_dir() resolves under the provided HOME
        std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
    }
    let path = claude_code_statusline_core::config_path();
    // restore original HOME
    match orig_home {
        Some(h) => unsafe { std::env::set_var("HOME", h) },
        None => unsafe { std::env::remove_var("HOME") },
    }
    // restore original XDG_CONFIG_HOME
    match orig_xdg {
        Some(v) => unsafe { std::env::set_var("XDG_CONFIG_HOME", v) },
        None => unsafe { std::env::remove_var("XDG_CONFIG_HOME") },
    }
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| home.join(".config"))
}

/// Create a `cargo_bin` command with `HOME` and `XDG_CONFIG_HOME` configured
/// to point at the per-test config directory under the given `home`.
pub fn ccs_cmd_with_home(home: &Path) -> Command {
    let cfg_dir = config_dir_for_home(home);
    let mut cmd = ccs_cmd();
    cmd.env("HOME", home);
    cmd.env("XDG_CONFIG_HOME", &cfg_dir);
    cmd
}

pub fn write_basic_config(home: &Path, command_timeout: Option<u64>) {
    let cfg_dir = config_dir_for_home(home);
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
