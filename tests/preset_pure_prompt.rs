use predicates::prelude::*;
use rstest::*;
use std::fs;

mod common;
use common::cli::{
    ccs_cmd_with_home, config_dir_for_home, input_json_with_cwd, write_basic_config,
};

/// Verify the Pure Prompt preset renders a single line and module order.
#[rstest]
fn pure_preset_renders_single_line_in_order() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    write_basic_config(home, None);

    use git2::{Repository, Signature};
    use std::io::Write as _;
    use std::path::Path;

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

    // Ensure git_status is non-empty: create an untracked file
    let mut uf = std::fs::File::create(repo_dir.join("untracked.txt")).unwrap();
    writeln!(uf, "u").unwrap();
    uf.sync_all().unwrap();

    let _cfg_dir = config_dir_for_home(home);
    let mut cmd = ccs_cmd_with_home(home);
    cmd.write_stdin(input_json_with_cwd(repo_dir.to_str().unwrap()));

    // Single line, and basic ordering: directory -> git_branch -> git_status -> claude_model
    // We assert ordering via regex using key substrings: dir basename, branch symbol, '?' (untracked), and model name.
    // ANSI is left intact; we search by substrings.
    let dir_basename = repo_dir.file_name().unwrap().to_string_lossy().to_string();
    let re = format!(r"{dir}.*🌿.*\?.*Opus", dir = regex::escape(&dir_basename));

    cmd.assert()
        .success()
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap())
        .stdout(predicate::str::is_match(re).unwrap());
}
