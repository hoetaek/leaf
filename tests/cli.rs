use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn leaf_command() -> Command {
    Command::cargo_bin("leaf").expect("leaf binary exists")
}

#[test]
fn help_lists_init_and_new() {
    leaf_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"));
}
