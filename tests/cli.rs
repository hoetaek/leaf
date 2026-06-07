use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

fn leaf_command() -> Command {
    Command::cargo_bin("leaf").expect("leaf binary exists")
}

fn git_command() -> Command {
    let mut command = Command::new("git");
    for (key, _) in std::env::vars() {
        if key.starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
}

fn git_init(path: &Path) {
    git_command().arg("init").arg(path).assert().success();
}

fn exclude_contents(repo: &Path) -> String {
    fs::read_to_string(repo.join(".git/info/exclude")).expect("exclude file is readable")
}

#[test]
fn help_lists_init_and_new() {
    leaf_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("fall"));
}

#[test]
fn version_flag_prints_package_version() {
    leaf_command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "leaf {}",
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn init_creates_leaf_buckets_and_exclude_line() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/seeds").assert(predicate::path::is_dir());
    repo.child(".leaf/leaves").assert(predicate::path::is_dir());
    repo.child(".leaf/fallen").assert(predicate::path::is_dir());
    repo.child(".leaf/pressed")
        .assert(predicate::path::is_dir());
    assert_eq!(
        exclude_contents(repo.path())
            .lines()
            .filter(|line| *line == "/.leaf")
            .count(),
        1
    );
}

#[test]
fn init_is_idempotent() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    assert_eq!(
        exclude_contents(repo.path())
            .lines()
            .filter(|line| *line == "/.leaf")
            .count(),
        1
    );
}

#[test]
fn init_from_nested_cwd_uses_repo_root() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child("nested/deep")
        .create_dir_all()
        .expect("nested directory");

    leaf_command()
        .current_dir(repo.child("nested/deep").path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/seeds").assert(predicate::path::is_dir());
    repo.child(".leaf/pressed")
        .assert(predicate::path::is_dir());
    repo.child("nested/deep/.leaf")
        .assert(predicate::path::missing());
}

#[test]
fn init_preserves_exclude_without_trailing_newline() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    fs::write(repo.path().join(".git/info/exclude"), "existing").expect("write exclude");

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    assert_eq!(exclude_contents(repo.path()), "existing\n/.leaf\n");
}

#[test]
fn init_fails_outside_git_repo() {
    let dir = assert_fs::TempDir::new().expect("temp dir");

    leaf_command()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn init_fails_when_leaf_path_is_a_file() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf")
        .write_str("not a directory")
        .expect("write conflict");

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn init_works_in_git_worktree() {
    let root = assert_fs::TempDir::new().expect("temp root");
    let repo = root.child("repo");
    repo.create_dir_all().expect("repo dir");
    git_init(repo.path());
    fs::write(repo.path().join("README.md"), "initial\n").expect("readme");
    git_command()
        .current_dir(repo.path())
        .args(["add", "README.md"])
        .assert()
        .success();
    git_command()
        .current_dir(repo.path())
        .args([
            "-c",
            "user.email=leaf@example.invalid",
            "-c",
            "user.name=Leaf Test",
            "commit",
            "-m",
            "initial",
        ])
        .assert()
        .success();

    let worktree = root.child("worktree");
    git_command()
        .current_dir(repo.path())
        .args([
            "worktree",
            "add",
            "-b",
            "leaf-test-worktree",
            worktree.path().to_str().expect("utf8 path"),
        ])
        .assert()
        .success();

    leaf_command()
        .current_dir(worktree.path())
        .arg("init")
        .assert()
        .success();

    worktree
        .child(".leaf/seeds")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/leaves")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/fallen")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/pressed")
        .assert(predicate::path::is_dir());
}

#[test]
fn new_creates_seed_skeleton_and_bootstraps_repo() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();

    for path in [
        ".leaf/seeds/research-memo/00-status.md",
        ".leaf/seeds/research-memo/01-Learn/01-intent.md",
        ".leaf/seeds/research-memo/01-Learn/02-unknowns.md",
        ".leaf/seeds/research-memo/01-Learn/02-references/README.md",
        ".leaf/seeds/research-memo/02-Example/03-criteria.md",
        ".leaf/seeds/research-memo/02-Example/04-wireframe.md",
        ".leaf/seeds/research-memo/03-Architect/05-design.md",
        ".leaf/seeds/research-memo/03-Architect/07-tasks.md",
    ] {
        repo.child(path).assert(predicate::path::is_file());
    }
    repo.child(".leaf/seeds/research-memo/04-Feedback")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/leaves/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/fallen").assert(predicate::path::is_dir());
    repo.child(".leaf/pressed")
        .assert(predicate::path::is_dir());
    assert_eq!(
        exclude_contents(repo.path())
            .lines()
            .filter(|line| *line == "/.leaf")
            .count(),
        1
    );
}

#[test]
fn new_rejects_existing_seed_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");
    repo.child(".leaf/seeds/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("existing file");

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf seed already exists"));

    repo.child(".leaf/seeds/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn new_rejects_invalid_slugs() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    for slug in ["bad/value", "bad value", "메모", "bad.value"] {
        leaf_command()
            .current_dir(repo.path())
            .args(["new", slug])
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid slug"));
    }
}

#[test]
fn fall_moves_active_leaf_to_fallen_and_updates_status() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/leaves/research-memo/01-Learn")
        .create_dir_all()
        .expect("leaf dirs");
    repo.child(".leaf/leaves/research-memo/00-status.md")
        .write_str("# Previous Status\n\n- state: leaf\n- next action: review\n")
        .expect("status");
    repo.child(".leaf/leaves/research-memo/01-Learn/01-intent.md")
        .write_str("preserve this intent\n")
        .expect("intent");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "completed"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved .leaf/leaves/research-memo/ to .leaf/fallen/research-memo/",
        ));

    repo.child(".leaf/leaves/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/fallen/research-memo/01-Learn/01-intent.md")
        .assert("preserve this intent\n");

    let status = fs::read_to_string(repo.path().join(".leaf/fallen/research-memo/00-status.md"))
        .expect("fallen status");
    assert!(status.contains("# Leaf Status"));
    assert!(status.contains("- state: fallen"));
    assert!(status.contains("- fallen from: .leaf/leaves/research-memo"));
    assert!(status.contains("- fall reason: completed"));
    assert!(status.contains("- closure summary: -"));
    assert!(status.contains("- reusable lessons: -"));
    assert!(status.contains("- unresolved limits: -"));
    assert!(status.contains("- successor: -"));
    assert!(status.contains("## Fall Log"));
    assert!(status.contains("## Previous Status"));
    assert!(status.contains("- next action: review"));
}

#[test]
fn fall_rejects_seed_without_active_leaf() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "abandoned"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("active leaf does not exist"));

    repo.child(".leaf/seeds/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/fallen/research-memo")
        .assert(predicate::path::missing());
}

#[test]
fn fall_rejects_existing_fallen_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/leaves/research-memo")
        .create_dir_all()
        .expect("leaf dir");
    repo.child(".leaf/fallen/research-memo")
        .create_dir_all()
        .expect("fallen dir");
    repo.child(".leaf/fallen/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("fallen status");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "superseded"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fallen leaf already exists"));

    repo.child(".leaf/leaves/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/fallen/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn fall_rejects_blank_reason() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/leaves/research-memo")
        .create_dir_all()
        .expect("leaf dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "  "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fall reason cannot be empty"));
}
