use predicates::prelude::*;
use std::fs;
use test_support::cli::{ccs_cmd, input_json_with_cwd, write_basic_config};

#[test]
fn one_line_output_with_small_timeout() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    // Set minimal allowed timeout (50ms) and full format
    write_basic_config(home, Some(50));

    let cwd = home.join("work");
    fs::create_dir_all(&cwd).unwrap();

    let input = input_json_with_cwd(cwd.to_str().unwrap());

    let mut cmd = ccs_cmd();
    cmd.env("HOME", home);
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match("^[^\n]*$").unwrap());
}
