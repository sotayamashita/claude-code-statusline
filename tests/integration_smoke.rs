use predicates::prelude::*;
use rstest::*;
use std::fs;
mod common;
use common::{beacon_cmd, input_json_with_cwd, write_basic_config};

#[rstest]
#[case(false)]
#[case(true)]
fn smoke_one_line_output(#[case] with_git_repo: bool) {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    write_basic_config(home, None);

    use git2::{Repository, Signature};
    use std::io::Write as _;
    use std::path::Path;

    // Prepare cwd depending on case
    let cwd = if with_git_repo {
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
        repo_dir
    } else {
        let d = home.join("nogit");
        fs::create_dir_all(&d).unwrap();
        d
    };

    let mut cmd = beacon_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(input_json_with_cwd(cwd.to_str().unwrap()));

    let assert = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Opus"))
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap());

    if with_git_repo {
        assert.stdout(predicate::str::contains("🌿"));
    } else {
        assert.stdout(predicate::str::contains("🌿").not());
    }
}
