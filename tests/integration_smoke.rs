use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn beacon_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("binary exists")
}

fn input_json_with_cwd(cwd: &str) -> String {
    format!(
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
        cwd, cwd, cwd
    )
}

fn write_basic_config(home: &std::path::Path) {
    let cfg_dir = home.join(".config");
    fs::create_dir_all(&cfg_dir).unwrap();
    let toml = r#"
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
    "#;
    fs::write(cfg_dir.join("beacon.toml"), toml).unwrap();
}

#[test]
fn smoke_without_git_repo_produces_one_line_output() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    write_basic_config(home);

    // Create a directory that is not a git repo
    let cwd = home.join("nogit");
    fs::create_dir_all(&cwd).unwrap();

    let mut cmd = beacon_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(input_json_with_cwd(cwd.to_str().unwrap()));

    // Should succeed; stdout should contain model name and be single line
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Opus"))
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap());
}

#[test]
fn smoke_with_git_repo_shows_branch_symbol() {
    use git2::{Repository, Signature};
    use std::io::Write as _;
    use std::path::Path;

    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    write_basic_config(home);

    // Init a repo with an initial commit and set HEAD to main
    let repo_dir = home.join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = Repository::init(&repo_dir).unwrap();

    let sig = Signature::now("Tester", "tester@example.com").unwrap();
    let fp = repo_dir.join("README.md");
    let mut f = std::fs::File::create(&fp).unwrap();
    writeln!(f, "init").unwrap();
    f.sync_all().unwrap();
    let mut idx = repo.index().unwrap();
    idx.add_path(Path::new("README.md")).unwrap();
    let tree_id = idx.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let commit = repo
        .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();
    let commit0 = repo.find_commit(commit).unwrap();
    if repo.find_branch("main", git2::BranchType::Local).is_err() {
        let _ = repo.branch("main", &commit0, true).unwrap();
    }
    let _ = repo.set_head("refs/heads/main");

    let mut cmd = beacon_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(input_json_with_cwd(repo_dir.to_str().unwrap()));
    cmd.assert()
        .success()
        // Expect the branch module emitted something with the symbol
        .stdout(predicate::str::contains("🌿"))
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap());
}
