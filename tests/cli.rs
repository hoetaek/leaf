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

fn write_status(repo: &assert_fs::TempDir, path: &str, body: &str) {
    repo.child(path).write_str(body).expect("write status");
}

#[test]
fn help_lists_init_and_new() {
    leaf_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("promote"))
        .stdout(predicate::str::contains("fall"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("doctor"));
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

    repo.child(".leaf/01-seeds")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
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

    repo.child(".leaf/01-seeds")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
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

#[cfg(unix)]
#[test]
fn init_accepts_leaf_root_symlink_to_directory() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child("leaf-store")
        .create_dir_all()
        .expect("leaf store");
    std::os::unix::fs::symlink(repo.path().join("leaf-store"), repo.path().join(".leaf"))
        .expect("leaf symlink");

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child("leaf-store/01-seeds")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/04-pressed")
        .assert(predicate::path::is_dir());
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
        .child(".leaf/01-seeds")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/04-pressed")
        .assert(predicate::path::is_dir());
}

#[test]
fn list_writes_deterministic_text_output_for_captured_stdout() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/02-leaves/active/00-status.md",
        "- state: active\n\
         - current phase: Architect\n\
         - current gate: ⑦ Task Graph\n\
         - first missing gate: ⑧ Artifact / execution\n\
         - next action: implement\n",
    );
    write_status(
        &repo,
        ".leaf/01-seeds/draft/00-status.md",
        "- state: seed\n- current phase: Learn\n",
    );
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");
    repo.child(".leaf/04-pressed/summary.md")
        .write_str("# Summary\n")
        .expect("pressed");

    leaf_command()
        .current_dir(repo.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "BUCKET  PHASE      GATE           SLUG     STATUS",
        ))
        .stdout(predicate::str::contains(
            "seed    Learn      -              draft    partial",
        ))
        .stdout(predicate::str::contains(
            "leaf    Architect  ⑦ Task Graph   active   ok",
        ))
        .stdout(predicate::str::contains(
            "pressed -          -              summary  ok",
        ))
        .stdout(predicate::str::contains("STATE").not())
        .stdout(predicate::str::contains("empty: fallen"));
}

#[cfg(unix)]
#[test]
fn list_accepts_leaf_root_symlink_to_directory() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        "leaf-store/02-leaves/active/00-status.md",
        "- state: active\n\
         - current phase: Architect\n\
         - current gate: ⑦ Task Graph\n\
         - first missing gate: ⑧ Artifact / execution\n\
         - next action: implement\n",
    );
    std::os::unix::fs::symlink(repo.path().join("leaf-store"), repo.path().join(".leaf"))
        .expect("leaf symlink");

    leaf_command()
        .current_dir(repo.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("empty: seeds, fallen, pressed"));
}

#[test]
fn list_json_outputs_all_buckets_and_degraded_parse_states() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/01-seeds/draft/00-status.md",
        "- state: seed\n- current phase: Learn\n",
    );
    write_status(
        &repo,
        ".leaf/02-leaves/active/00-status.md",
        "- state: active\n\
         - current phase: Architect\n\
         - current gate: ⑦ Task Graph\n\
         - first missing gate: ⑧ Artifact / execution\n\
         - next action: implement\n",
    );
    write_status(
        &repo,
        ".leaf/03-fallen/done/00-status.md",
        "- state: fallen\n- fall reason: completed\n",
    );
    repo.child(".leaf/04-pressed/summary.md")
        .write_str("# Summary\n")
        .expect("pressed");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["leaf_root"], ".leaf");
    assert_eq!(json["buckets"]["seeds"]["count"], 1);
    assert_eq!(json["buckets"]["leaves"]["count"], 1);
    assert_eq!(json["buckets"]["fallen"]["count"], 1);
    assert_eq!(json["buckets"]["pressed"]["count"], 1);
    assert_eq!(json["buckets"]["seeds"]["items"][0]["bucket"], "seeds");
    assert_eq!(json["buckets"]["seeds"]["items"][0]["slug"], "draft");
    assert_eq!(
        json["buckets"]["seeds"]["items"][0]["status"]["parse_state"],
        "partial"
    );
    assert_eq!(
        json["buckets"]["seeds"]["items"][0]["status"]["missing_fields"],
        serde_json::json!(["current_gate", "first_missing_gate", "next_action"])
    );
    assert_eq!(
        json["buckets"]["leaves"]["items"][0]["path"],
        ".leaf/02-leaves/active"
    );
    assert_eq!(
        json["buckets"]["leaves"]["items"][0]["status"]["parse_state"],
        "ok"
    );
    assert_eq!(
        json["buckets"]["pressed"]["items"][0]["kind"],
        "pressed_digest"
    );
}

#[test]
fn list_missing_leaf_root_fails_without_bootstrapping() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    let exclude_before = exclude_contents(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            ".leaf/ is not initialized in this git repository",
        ))
        .stderr(predicate::str::contains("hint: run `leaf init`"));

    repo.child(".leaf").assert(predicate::path::missing());
    assert_eq!(exclude_contents(repo.path()), exclude_before);
}

#[test]
fn doctor_healthy_workspace_exits_zero_with_ready_result() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("summary  errors 0  warnings 0"))
        .stdout(predicate::str::contains(
            "result   ready: leaf list should display cleanly",
        ))
        .stdout(predicate::str::contains("OK checks"))
        .stdout(predicate::str::contains("OK leaf_root_present"))
        .stdout(predicate::str::contains("OK lifecycle_buckets_readable"));
}

#[test]
fn doctor_warning_only_workspace_exits_zero_with_warning_result() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/01-seeds/draft/00-status.md",
        "- state: seed\n- current phase: Learn\n",
    );
    repo.child(".leaf/02-leaves")
        .create_dir_all()
        .expect("leaves");
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");
    repo.child(".leaf/04-pressed")
        .create_dir_all()
        .expect("pressed");

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "result   usable with warnings: leaf list may be degraded",
        ))
        .stdout(predicate::str::contains("WARN status_missing_fields"))
        .stdout(predicate::str::contains(
            "path    .leaf/01-seeds/draft/00-status.md",
        ))
        .stdout(predicate::str::contains(
            "reason  missing status fields: current_gate, first_missing_gate, next_action",
        ));
}

#[test]
fn doctor_error_workspace_prints_findings_then_exits_one() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-seeds")
        .create_dir_all()
        .expect("seeds");
    repo.child(".leaf/02-leaves/no-status/01-Learn")
        .create_dir_all()
        .expect("leaf");
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");
    repo.child(".leaf/04-pressed")
        .create_dir_all()
        .expect("pressed");

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "result   not ready: fix errors before trusting leaf list",
        ))
        .stdout(predicate::str::contains("ERROR status_unreadable"))
        .stdout(predicate::str::contains(
            "path    .leaf/02-leaves/no-status/00-status.md",
        ));
}

#[test]
fn doctor_json_outputs_flat_findings_with_paths() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/01-seeds/duplicate/00-status.md",
        "- state: seed\n\
         - current phase: Learn\n\
         - current gate: ② Unknowns & Context\n\
         - first missing gate: ③ Criteria\n\
         - next action: promote\n",
    );
    write_status(
        &repo,
        ".leaf/02-leaves/duplicate/00-status.md",
        "- state: active\n\
         - current phase: Architect\n\
         - current gate: ⑦ Tasks\n\
         - first missing gate: ⑧ Artifact\n\
         - next action: implement\n",
    );
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");
    repo.child(".leaf/04-pressed")
        .create_dir_all()
        .expect("pressed");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["doctor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["leaf_root"], ".leaf");
    assert_eq!(json["summary"]["errors"], 0);
    assert_eq!(json["summary"]["warnings"], 1);
    let duplicate = json["findings"]
        .as_array()
        .expect("findings array")
        .iter()
        .find(|finding| finding["code"] == "duplicate_slug")
        .expect("duplicate finding");
    assert_eq!(duplicate["severity"], "warn");
    assert_eq!(
        duplicate["paths"],
        serde_json::json!([".leaf/01-seeds/duplicate", ".leaf/02-leaves/duplicate"])
    );
    assert!(duplicate.get("path").is_none());
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
        ".leaf/01-seeds/research-memo/00-status.md",
        ".leaf/01-seeds/research-memo/01-Learn/01-intent.md",
        ".leaf/01-seeds/research-memo/01-Learn/02-unknowns.md",
        ".leaf/01-seeds/research-memo/01-Learn/02-references/README.md",
        ".leaf/01-seeds/research-memo/02-Example/03-criteria.md",
        ".leaf/01-seeds/research-memo/02-Example/04-wireframe.md",
        ".leaf/01-seeds/research-memo/03-Architect/05-design.md",
        ".leaf/01-seeds/research-memo/03-Architect/07-tasks.md",
    ] {
        repo.child(path).assert(predicate::path::is_file());
    }
    repo.child(".leaf/01-seeds/research-memo/04-Feedback")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
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
fn new_seed_passes_doctor_without_warnings() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("summary  errors 0  warnings 0"))
        .stdout(predicate::str::contains(
            "result   ready: leaf list should display cleanly",
        ))
        .stdout(predicate::str::contains("status_missing_fields").not());
}

#[test]
fn new_rejects_existing_seed_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");
    repo.child(".leaf/01-seeds/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("existing file");

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf seed already exists"));

    repo.child(".leaf/01-seeds/research-memo/00-status.md")
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
fn promote_moves_seed_to_active_leaf_and_updates_status() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    repo.child(".leaf/01-seeds/research-memo/01-Learn/01-intent.md")
        .write_str("preserve this intent\n")
        .expect("intent");

    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved .leaf/01-seeds/research-memo/ to .leaf/02-leaves/research-memo/",
        ));

    repo.child(".leaf/01-seeds/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/02-leaves/research-memo/01-Learn/01-intent.md")
        .assert("preserve this intent\n");

    let status = fs::read_to_string(
        repo.path()
            .join(".leaf/02-leaves/research-memo/00-status.md"),
    )
    .expect("active leaf status");
    assert!(status.contains("# Leaf Status"));
    assert!(status.contains("- state: active"));
    assert!(status.contains("- current phase: Example"));
    assert!(status.contains("- promoted from: .leaf/01-seeds/research-memo"));
    assert!(status.contains("## Promotion Log"));
    assert!(status.contains("## Previous Status"));
    assert!(status.contains("- state: seed"));
}

#[test]
fn promoted_leaf_passes_doctor_without_warnings() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .success();

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("summary  errors 0  warnings 0"))
        .stdout(predicate::str::contains(
            "result   ready: leaf list should display cleanly",
        ))
        .stdout(predicate::str::contains("status_missing_fields").not());
}

#[test]
fn promote_rejects_missing_seed() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("seed does not exist"));

    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::missing());
}

#[test]
fn promote_rejects_existing_active_leaf_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");
    repo.child(".leaf/01-seeds/research-memo/00-status.md")
        .write_str("seed status\n")
        .expect("seed status");
    repo.child(".leaf/02-leaves/research-memo")
        .create_dir_all()
        .expect("leaf dir");
    repo.child(".leaf/02-leaves/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("active status");

    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("active leaf already exists"));

    repo.child(".leaf/01-seeds/research-memo/00-status.md")
        .assert("seed status\n");
    repo.child(".leaf/02-leaves/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn promote_rejects_existing_fallen_leaf_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");
    repo.child(".leaf/03-fallen/research-memo")
        .create_dir_all()
        .expect("fallen dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fallen leaf already exists"));

    repo.child(".leaf/01-seeds/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::missing());
}

#[test]
fn fall_moves_active_leaf_to_fallen_and_updates_status() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/02-leaves/research-memo/01-Learn")
        .create_dir_all()
        .expect("leaf dirs");
    repo.child(".leaf/02-leaves/research-memo/00-status.md")
        .write_str("# Previous Status\n\n- state: leaf\n- next action: review\n")
        .expect("status");
    repo.child(".leaf/02-leaves/research-memo/01-Learn/01-intent.md")
        .write_str("preserve this intent\n")
        .expect("intent");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "completed"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved .leaf/02-leaves/research-memo/ to .leaf/03-fallen/research-memo/",
        ));

    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/03-fallen/research-memo/01-Learn/01-intent.md")
        .assert("preserve this intent\n");

    let status = fs::read_to_string(
        repo.path()
            .join(".leaf/03-fallen/research-memo/00-status.md"),
    )
    .expect("fallen status");
    assert!(status.contains("# Leaf Status"));
    assert!(status.contains("- state: fallen"));
    assert!(status.contains("- fallen from: .leaf/02-leaves/research-memo"));
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
    repo.child(".leaf/01-seeds/research-memo")
        .create_dir_all()
        .expect("seed dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "abandoned"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("active leaf does not exist"));

    repo.child(".leaf/01-seeds/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen/research-memo")
        .assert(predicate::path::missing());
}

#[test]
fn fall_rejects_existing_fallen_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/02-leaves/research-memo")
        .create_dir_all()
        .expect("leaf dir");
    repo.child(".leaf/03-fallen/research-memo")
        .create_dir_all()
        .expect("fallen dir");
    repo.child(".leaf/03-fallen/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("fallen status");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "superseded"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fallen leaf already exists"));

    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn fall_rejects_blank_reason() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/02-leaves/research-memo")
        .create_dir_all()
        .expect("leaf dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "  "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fall reason cannot be empty"));
}

#[test]
fn init_creates_lifecycle_ordered_buckets() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/01-seeds")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::is_dir());
}

#[test]
fn legacy_buckets_migrate_to_prefixed_names_preserving_content() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());
    repo.child(".leaf/seeds/old-idea/00-status.md")
        .write_str("- state: seed\n")
        .expect("legacy seed status");
    repo.child(".leaf/leaves/active/01-Learn/01-intent.md")
        .write_str("# Intent\n")
        .expect("legacy leaf intent");

    leaf_command()
        .current_dir(repo.path())
        .arg("new")
        .arg("fresh")
        .assert()
        .success();

    repo.child(".leaf/01-seeds/old-idea/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/02-leaves/active/01-Learn/01-intent.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/seeds").assert(predicate::path::missing());
    repo.child(".leaf/leaves")
        .assert(predicate::path::missing());
    repo.child(".leaf/01-seeds/fresh/00-status.md")
        .assert(predicate::path::is_file());
}

#[test]
fn migration_bails_on_legacy_new_conflict_without_moving() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());
    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .write_str("- state: seed\n")
        .expect("legacy seed");
    repo.child(".leaf/01-seeds/prefixed-item/00-status.md")
        .write_str("- state: seed\n")
        .expect("prefixed seed");

    leaf_command()
        .current_dir(repo.path())
        .arg("new")
        .arg("whatever")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no files were moved"));

    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/01-seeds/prefixed-item/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/01-seeds/whatever")
        .assert(predicate::path::missing());
}
