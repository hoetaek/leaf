use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

fn leaf_command() -> Command {
    static SHARED_CONFIG_DIR: std::sync::OnceLock<assert_fs::TempDir> = std::sync::OnceLock::new();
    let config_dir = SHARED_CONFIG_DIR
        .get_or_init(|| assert_fs::TempDir::new().expect("temp shared config dir"));
    let mut command = Command::cargo_bin("leaf").expect("leaf binary exists");
    command.env("LEAF_CONFIG_DIR", config_dir.path());
    command
}

fn leaf_command_with_config(config_dir: &Path) -> Command {
    let mut command = leaf_command();
    command.env("LEAF_CONFIG_DIR", config_dir);
    command
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

fn write_leaf_status(repo: &assert_fs::TempDir, slug: &str, pressed: bool) {
    write_status(
        repo,
        &format!(".leaf/02-leaves/{slug}/00-status.md"),
        "- stage: leaf\n\
         - current phase: Architect\n\
         - current gate: ⑦ Tasks\n\
         - first missing gate: ⑧ Artifact\n\
         - next action: render tree\n",
    );
    if pressed {
        repo.child(format!(".leaf/02-leaves/{slug}/pressed.md"))
            .write_str("# Pressed\n")
            .expect("pressed digest");
    }
}

fn write_sprout_status(repo: &assert_fs::TempDir, slug: &str) {
    write_status(
        repo,
        &format!(".leaf/01-sprouts/{slug}/00-status.md"),
        "- stage: sprout\n\
         - current phase: Learn\n\
         - current gate: ② Unknowns\n\
         - first missing gate: ③ Criteria\n\
         - next action: continue\n",
    );
}

fn write_fallen_status(repo: &assert_fs::TempDir, slug: &str) {
    write_status(
        repo,
        &format!(".leaf/03-fallen/{slug}/00-status.md"),
        "- stage: fallen\n- fallen reason: archived\n",
    );
}

#[test]
fn help_lists_init_and_new() {
    leaf_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("promote").not())
        .stdout(predicate::str::contains("fall"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("review"))
        .stdout(predicate::str::contains("profile"))
        .stdout(predicate::str::contains("checkpoint"))
        .stdout(predicate::str::contains("tree"))
        .stdout(predicate::str::contains("graph"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn tree_help_describes_demo_as_top_to_bottom_not_left_to_right() {
    leaf_command()
        .args(["tree", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--demo"))
        .stdout(predicate::str::contains("top-to-bottom"))
        .stdout(predicate::str::contains("left-to-right").not());
}

#[test]
fn graph_json_exports_pressed_leaf_nodes_and_link_edges() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();
    write_status(
        &repo,
        ".leaf/02-leaves/reference/00-status.md",
        "- stage: leaf\n\
         - current phase: Feedback\n\
         - current gate: ⑨ Review\n\
         - first missing gate: ⑩ Retrospect\n\
         - next action: review\n",
    );
    repo.child(".leaf/02-leaves/reference/pressed.md")
        .write_str(
            "---\n\
             type: Leaf Pressed Digest\n\
             title: Reference Leaf\n\
             description: One-sentence summary.\n\
             resource: .leaf/02-leaves/reference\n\
             tags: [leaf, reference]\n\
             timestamp: 2026-06-22T10:00:00+09:00\n\
             citation_handle: leaf:reference\n\
             stage: leaf\n\
             ---\n\
             \n\
             # Reference Leaf\n",
        )
        .expect("pressed digest");
    repo.child(".leaf/02-leaves/reference/linked.md")
        .write_str("# Links\n\n- `cites` -> `okf:spec` - OKF source\n")
        .expect("linked metadata");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["graph", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid json");

    assert_eq!(json["leaf_root"], ".leaf");
    assert_eq!(json["nodes"][0]["id"], "leaf:reference");
    assert_eq!(json["nodes"][0]["title"], "Reference Leaf");
    assert_eq!(
        json["nodes"][0]["tags"],
        serde_json::json!(["leaf", "reference"])
    );
    assert_eq!(json["edges"][0]["source"], "leaf:reference");
    assert_eq!(json["edges"][0]["predicate"], "cites");
    assert_eq!(json["edges"][0]["target"], "okf:spec");
    assert_eq!(json["edges"][0]["note"], "OKF source");
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
fn leaf_tree_command_renders_color_by_default_even_when_stdout_is_captured() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_leaf_status(&repo, "alpha", true);
    write_leaf_status(&repo, "beta", false);
    write_sprout_status(&repo, "draft");
    write_fallen_status(&repo, "archived");

    let output = leaf_command()
        .current_dir(repo.path())
        .arg("tree")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.contains("\x1b["),
        "default tree output must keep ANSI: {text:?}"
    );
    assert!(text.contains("leaf tree"));
    assert!(text.contains("leaves 2"));
    assert!(text.contains("pressed 1"));
    assert!(text.contains("sprouts 1"));
    assert!(text.contains("fallen 1"));
    assert!(text.contains("active sprouts:"));
    assert!(text.contains("gold fruit:"));
    assert!(text.contains("green leaves:"));
    assert!(text.contains("fallen:"));
}

#[test]
fn leaf_tree_plain_removes_ansi_but_keeps_tree_semantics() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_leaf_status(&repo, "alpha", true);
    write_leaf_status(&repo, "beta", false);

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        !text.contains("\x1b["),
        "plain tree output must not contain ANSI: {text:?}"
    );
    assert!(text.contains("leaf tree"));
    assert!(text.contains("gold fruit:"));
    assert!(text.contains("green leaves:"));
    assert!(text.contains("alpha"));
    assert!(text.contains("beta"));
}

#[test]
fn leaf_tree_demo_plain_renders_python_g_style_stacked_stages_without_leaf_root() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--demo", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        !text.contains("\x1b["),
        "plain demo output must not contain ANSI: {text:?}"
    );
    assert!(text.contains("leaf tree demo"));
    assert!(
        !text.contains("left -> right"),
        "demo must not be the compressed left-to-right strip:\n{text}"
    );

    let stages = [
        "===== 0 leaves / seedling demo =====",
        "===== 3 leaves / young demo =====",
        "===== 10 leaves / branching demo =====",
        "===== 20 leaves / grown demo =====",
        "===== 50 leaves / mature demo =====",
        "===== 100 leaves / saturated demo =====",
    ];
    let mut previous = 0;
    for stage in stages {
        let position = text
            .find(stage)
            .unwrap_or_else(|| panic!("missing {stage:?} in stacked demo:\n{text}"));
        assert!(
            position >= previous,
            "stage sections must be ordered top to bottom:\n{text}"
        );
        previous = position;
    }
    assert!(
        !text.contains("===== 5 leaves / small demo ====="),
        "5-leaf stage is visually too close to adjacent stages and should not be in the public demo:\n{text}"
    );
}

#[test]
fn leaf_tree_demo_uses_color_by_default() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--demo"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.contains("\x1b["),
        "default demo output must keep ANSI: {text:?}"
    );
    assert!(text.contains("leaf tree demo"));
}

#[test]
fn leaf_tree_demo_plain_uses_full_size_stage_trees_not_tiny_panels() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--demo", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(
        text.lines().count() > 80,
        "demo should stack full-size stage trees like the Python g-mode, not tiny panels:\n{text}"
    );
}

#[test]
fn leaf_tree_output_is_deterministic_for_same_workspace() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_leaf_status(&repo, "alpha", true);
    write_leaf_status(&repo, "beta", false);
    write_sprout_status(&repo, "draft");
    write_fallen_status(&repo, "archived");

    let first = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let second = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(first, second);
}

#[test]
fn leaf_tree_no_pressed_many_leaves_shows_green_crown_without_gold_fruit() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    for index in 1..=50 {
        write_leaf_status(&repo, &format!("leaf-{index:02}"), false);
    }

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(text.contains("leaves 50"));
    assert!(text.contains("pressed 0"));
    assert!(text.contains("green leaves:"));
    assert!(text.contains("no gold fruit: no pressed leaf yet"));
    assert!(text.contains("leaf-50"));
}

#[test]
fn leaf_tree_all_pressed_omits_empty_green_leaf_legend() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    for slug in ["alpha", "beta", "gamma"] {
        write_leaf_status(&repo, slug, true);
    }

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(text.contains("pressed 3"));
    assert!(text.contains("gold fruit:"));
    assert!(!text.contains("green leaves:"));
}

#[test]
fn leaf_tree_sprouts_heavy_uses_active_sprouts_seedlings() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_leaf_status(&repo, "citable-leaf", true);
    for index in 1..=8 {
        write_sprout_status(&repo, &format!("draft-{index}"));
    }

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    assert!(text.contains("sprouts 8"));
    assert!(text.contains("active sprouts:"));
    assert!(text.contains(r"\|/"));
    assert!(text.contains("draft-1"));
}

#[test]
fn leaf_tree_fallen_items_render_below_living_sections() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_leaf_status(&repo, "alpha", true);
    write_sprout_status(&repo, "draft");
    write_fallen_status(&repo, "archived");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["tree", "--plain"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");

    let sprouts_at = text.find("active sprouts:").expect("sprouts section");
    let fallen_at = text.find("fallen:").expect("fallen section");
    assert!(
        fallen_at > sprouts_at,
        "fallen must be below living sections:\n{text}"
    );
    assert!(text.contains("archived"));
}

#[test]
fn leaf_tree_missing_leaf_root_fails_without_bootstrapping() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("tree")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            ".leaf/ is not initialized in this git repository",
        ))
        .stderr(predicate::str::contains("hint: run `leaf init`"));

    repo.child(".leaf").assert(predicate::path::missing());
}

#[test]
fn init_creates_stage_dirs_and_exclude_line() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/01-sprouts")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::missing());
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
fn init_creates_profile_file() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/PROFILE.md")
        .assert(predicate::path::is_file());
    let body = fs::read_to_string(repo.path().join(".leaf/PROFILE.md")).expect("profile readable");
    assert!(body.starts_with("# Profile"));
    assert!(body.contains("## Settled"));
    assert!(body.contains("## Provisional"));
}

#[test]
fn init_preserves_existing_profile_file() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    let custom = "# Profile\n\ncustom content that must survive\n\n## Settled\n\n## Provisional\n";
    repo.child(".leaf/PROFILE.md")
        .write_str(custom)
        .expect("write custom profile");

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(repo.path().join(".leaf/PROFILE.md")).expect("profile readable"),
        custom
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

    repo.child(".leaf/01-sprouts")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::missing());
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

    repo.child("leaf-store/01-sprouts")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child("leaf-store/04-pressed")
        .assert(predicate::path::missing());
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
        .child(".leaf/01-sprouts")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    worktree
        .child(".leaf/04-pressed")
        .assert(predicate::path::missing());
}

#[test]
fn list_writes_deterministic_text_output_for_captured_stdout() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/02-leaves/done/00-status.md",
        "- stage: leaf\n\
         - current phase: Architect\n\
         - current gate: ⑦ Task Graph\n\
         - first missing gate: ⑧ Artifact / execution\n\
         - next action: implement\n",
    );
    write_status(
        &repo,
        ".leaf/01-sprouts/draft/00-status.md",
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
            "STAGE   PHASE      GATE           SLUG     STATUS",
        ))
        .stdout(predicate::str::contains(
            "sprout  Learn      -              draft    partial",
        ))
        .stdout(predicate::str::contains(
            "leaf    Architect  ⑦ Task Graph   done     ok",
        ))
        .stdout(predicate::str::contains("pressed").not())
        .stdout(predicate::str::contains("BUCKET").not())
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
        "- stage: leaf\n\
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
        .stdout(predicate::str::contains("empty: sprouts, fallen"));
}

#[test]
fn list_json_outputs_stages_and_degraded_parse_states() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/01-sprouts/draft/00-status.md",
        "- state: seed\n- current phase: Learn\n",
    );
    write_status(
        &repo,
        ".leaf/02-leaves/done/00-status.md",
        "- stage: leaf\n\
         - current phase: Architect\n\
         - current gate: ⑦ Task Graph\n\
         - first missing gate: ⑧ Artifact / execution\n\
         - next action: implement\n",
    );
    write_status(
        &repo,
        ".leaf/03-fallen/archived/00-status.md",
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
    assert!(json.get("buckets").is_none());
    assert_eq!(json["stages"]["sprouts"]["count"], 1);
    assert_eq!(json["stages"]["leaves"]["count"], 1);
    assert_eq!(json["stages"]["fallen"]["count"], 1);
    assert!(json["stages"].get("pressed").is_none());
    assert_eq!(json["stages"]["sprouts"]["items"][0]["stage"], "sprout");
    assert_eq!(json["stages"]["sprouts"]["items"][0]["slug"], "draft");
    assert_eq!(
        json["stages"]["sprouts"]["items"][0]["status"]["parse_state"],
        "partial"
    );
    assert_eq!(
        json["stages"]["sprouts"]["items"][0]["status"]["stage"],
        "sprout"
    );
    assert!(
        json["stages"]["sprouts"]["items"][0]["status"]
            .get("state")
            .is_none()
    );
    assert!(
        json["stages"]["sprouts"]["items"][0]["status"]
            .get("legacy_state")
            .is_none()
    );
    assert_eq!(
        json["stages"]["sprouts"]["items"][0]["status"]["missing_fields"],
        serde_json::json!(["current_gate", "first_missing_gate", "next_action"])
    );
    assert_eq!(
        json["stages"]["leaves"]["items"][0]["path"],
        ".leaf/02-leaves/done"
    );
    assert_eq!(
        json["stages"]["leaves"]["items"][0]["status"]["parse_state"],
        "ok"
    );
    assert_eq!(
        json["stages"]["fallen"]["items"][0]["status"]["fallen_reason"],
        "completed"
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
fn review_writes_selected_leaf_work_for_captured_stdout() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/02-leaves/demo/00-status.md",
        "- stage: leaf\n\
         - current phase: Learn\n\
         - current gate: ① Intent\n\
         - first missing gate: none\n\
         - next action: review\n",
    );
    repo.child(".leaf/02-leaves/demo/01-Learn/01-intent.md")
        .write_str("# Intent\n\nopen this reader directly\n")
        .expect("intent");

    leaf_command()
        .current_dir(repo.path())
        .args(["review", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            ".leaf/02-leaves/demo/00-status.md",
        ))
        .stdout(predicate::str::contains("open this reader directly"));
}

#[test]
fn review_rejects_missing_slug() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    leaf_command()
        .current_dir(repo.path())
        .args(["review", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "leaf work does not exist: missing",
        ));
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
        .stdout(predicate::str::contains("OK stage_dirs_readable"));
}

#[test]
fn doctor_warning_only_workspace_exits_zero_with_warning_result() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_status(
        &repo,
        ".leaf/01-sprouts/draft/00-status.md",
        "- stage: sprout\n- current phase: Learn\n",
    );
    repo.child(".leaf/02-leaves")
        .create_dir_all()
        .expect("leaves");
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");

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
            "path    .leaf/01-sprouts/draft/00-status.md",
        ))
        .stdout(predicate::str::contains(
            "reason  missing status fields: current_gate, first_missing_gate, next_action",
        ));
}

#[test]
fn doctor_error_workspace_prints_findings_then_exits_one() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-sprouts")
        .create_dir_all()
        .expect("sprouts");
    repo.child(".leaf/02-leaves/no-status/01-Learn")
        .create_dir_all()
        .expect("leaf");
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");

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
        ".leaf/01-sprouts/duplicate/00-status.md",
        "- stage: sprout\n\
         - current phase: Learn\n\
         - current gate: ② Unknowns & Context\n\
         - first missing gate: ③ Criteria\n\
         - next action: continue\n",
    );
    write_status(
        &repo,
        ".leaf/02-leaves/duplicate/00-status.md",
        "- stage: leaf\n\
         - current phase: Feedback\n\
         - current gate: ⑨ Review\n\
         - first missing gate: ⑩ Retrospect\n\
         - next action: review\n",
    );
    repo.child(".leaf/03-fallen")
        .create_dir_all()
        .expect("fallen");

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
        serde_json::json!([".leaf/01-sprouts/duplicate", ".leaf/02-leaves/duplicate"])
    );
    assert!(duplicate.get("path").is_none());
}

#[test]
fn new_creates_sprout_skeleton_and_bootstraps_repo() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();

    for path in [
        ".leaf/01-sprouts/research-memo/00-status.md",
        ".leaf/01-sprouts/research-memo/01-Learn/01-intent.md",
        ".leaf/01-sprouts/research-memo/01-Learn/02-unknowns.md",
        ".leaf/01-sprouts/research-memo/01-Learn/02-references/README.md",
        ".leaf/01-sprouts/research-memo/02-Example/03-criteria.md",
        ".leaf/01-sprouts/research-memo/02-Example/04-wireframe.md",
        ".leaf/01-sprouts/research-memo/03-Architect/05-design.md",
        ".leaf/01-sprouts/research-memo/03-Architect/07-tasks.md",
    ] {
        repo.child(path).assert(predicate::path::is_file());
    }
    repo.child(".leaf/01-sprouts/research-memo/04-Feedback")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::missing());
    repo.child(".leaf/01-sprouts/research-memo/00-status.md")
        .assert(predicate::str::contains("- stage: sprout"));
    repo.child(".leaf/01-sprouts/research-memo/00-status.md")
        .assert(predicate::str::contains("- state:").not());
    assert_eq!(
        exclude_contents(repo.path())
            .lines()
            .filter(|line| *line == "/.leaf")
            .count(),
        1
    );
}

#[test]
fn new_sprout_passes_doctor_without_warnings() {
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
fn new_rejects_existing_sprout_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-sprouts/research-memo")
        .create_dir_all()
        .expect("sprout dir");
    repo.child(".leaf/01-sprouts/research-memo/00-status.md")
        .write_str("keep me\n")
        .expect("existing file");

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf sprout already exists"));

    repo.child(".leaf/01-sprouts/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn new_rejects_existing_leaf_or_fallen_slug_without_creating_duplicate_sprout() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();
    write_leaf_status(&repo, "research-memo", false);
    write_fallen_status(&repo, "archived-memo");

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf slug already exists"));
    leaf_command()
        .current_dir(repo.path())
        .args(["new", "archived-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf slug already exists"));

    repo.child(".leaf/01-sprouts/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/01-sprouts/archived-memo")
        .assert(predicate::path::missing());
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
fn checkpoint_copies_selected_gate_with_timestamped_name() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    repo.child(".leaf/01-sprouts/research-memo/02-Example/03-criteria.md")
        .write_str("# Criteria\n\nCurrent report\n")
        .expect("criteria");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo", "--criteria"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "checkpointed .leaf/01-sprouts/research-memo/02-Example/03-criteria.md to .leaf/01-sprouts/research-memo/02-Example/",
        ))
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");
    let checkpoint_path = text.trim().split(" to ").nth(1).expect("checkpoint path");

    assert!(checkpoint_path.ends_with(" 03-criteria.md"));
    repo.child(checkpoint_path)
        .assert("# Criteria\n\nCurrent report\n");
}

#[test]
fn checkpoint_accepts_numeric_gate_flag() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    repo.child(".leaf/01-sprouts/research-memo/03-Architect/07-tasks.md")
        .write_str("task graph\n")
        .expect("tasks");

    leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo", "--7"])
        .assert()
        .success()
        .stdout(predicate::str::contains(" 07-tasks.md"));

    leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo", "--gate", "03"])
        .assert()
        .success()
        .stdout(predicate::str::contains(" 03-criteria.md"));
}

#[test]
fn checkpoint_copies_folder_based_wireframe() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    std::fs::remove_file(
        repo.path()
            .join(".leaf/01-sprouts/research-memo/02-Example/04-wireframe.md"),
    )
    .expect("replace scaffolded wireframe file with folder layout");
    repo.child(".leaf/01-sprouts/research-memo/02-Example/04-wireframe/index.html")
        .write_str("<html></html>\n")
        .expect("wireframe html");
    repo.child(".leaf/01-sprouts/research-memo/02-Example/04-wireframe/contracts.md")
        .write_str("# Contracts\n")
        .expect("wireframe contracts");

    let output = leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo", "--wireframe"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "checkpointed .leaf/01-sprouts/research-memo/02-Example/04-wireframe to .leaf/01-sprouts/research-memo/02-Example/",
        ))
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8 output");
    let checkpoint_path = text.trim().split(" to ").nth(1).expect("checkpoint path");

    assert!(checkpoint_path.ends_with(" 04-wireframe"));
    repo.child(format!("{checkpoint_path}/index.html"))
        .assert("<html></html>\n");
    repo.child(format!("{checkpoint_path}/contracts.md"))
        .assert("# Contracts\n");
}

#[test]
fn checkpoint_requires_exactly_one_gate() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();

    leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing gate flag"));

    leaf_command()
        .current_dir(repo.path())
        .args(["checkpoint", "research-memo", "--criteria", "--3"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("choose exactly one gate flag"));
}

#[test]
fn promote_is_not_a_command() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["promote", "research-memo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'promote'",
        ));
}

#[test]
fn fall_moves_sprout_to_fallen_and_updates_status() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["new", "research-memo"])
        .assert()
        .success();
    repo.child(".leaf/01-sprouts/research-memo/01-Learn/01-intent.md")
        .write_str("preserve this intent\n")
        .expect("intent");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "completed"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "moved .leaf/01-sprouts/research-memo/ to .leaf/03-fallen/research-memo/",
        ));

    repo.child(".leaf/01-sprouts/research-memo")
        .assert(predicate::path::missing());
    repo.child(".leaf/03-fallen/research-memo/01-Learn/01-intent.md")
        .assert("preserve this intent\n");

    let status = fs::read_to_string(
        repo.path()
            .join(".leaf/03-fallen/research-memo/00-status.md"),
    )
    .expect("fallen status");
    assert!(status.contains("# Fallen Status"));
    assert!(status.contains("- stage: fallen"));
    assert!(status.contains("- fallen from: .leaf/01-sprouts/research-memo"));
    assert!(status.contains("- fallen reason: completed"));
    assert!(!status.contains("- fall reason:"));
    assert!(!repo.path().join(".leaf/fallen").exists());
}

#[test]
fn fall_moves_leaf_to_fallen_and_updates_status() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/02-leaves/research-memo/01-Learn")
        .create_dir_all()
        .expect("leaf dirs");
    repo.child(".leaf/02-leaves/research-memo/00-status.md")
        .write_str("# Previous Status\n\n- stage: leaf\n- next action: review\n")
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
    assert!(status.contains("# Fallen Status"));
    assert!(status.contains("- stage: fallen"));
    assert!(status.contains("- fallen from: .leaf/02-leaves/research-memo"));
    assert!(status.contains("- fallen reason: completed"));
    assert!(status.contains("- closure summary: -"));
    assert!(status.contains("- reusable lessons: -"));
    assert!(status.contains("- unresolved limits: -"));
    assert!(status.contains("- successor: -"));
    assert!(status.contains("## Fall Log"));
    assert!(status.contains("## Previous Status"));
    assert!(status.contains("- next action: review"));
}

#[test]
fn fall_rejects_missing_slug_without_creating_old_dirs() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "abandoned"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("leaf does not exist"));

    repo.child(".leaf/01-seeds")
        .assert(predicate::path::missing());
    repo.child(".leaf/sprouts")
        .assert(predicate::path::missing());
    repo.child(".leaf/leaves")
        .assert(predicate::path::missing());
    repo.child(".leaf/fallen")
        .assert(predicate::path::missing());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::missing());
}

#[test]
fn fall_rejects_existing_fallen_without_overwrite() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-sprouts/research-memo")
        .create_dir_all()
        .expect("sprout dir");
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

    repo.child(".leaf/01-sprouts/research-memo")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen/research-memo/00-status.md")
        .assert("keep me\n");
}

#[test]
fn fall_rejects_ambiguous_sprout_and_leaf_sources_without_moving_either() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    write_sprout_status(&repo, "research-memo");
    write_leaf_status(&repo, "research-memo", false);

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "superseded"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ambiguous leaf slug"))
        .stderr(predicate::str::contains("run leaf doctor"));

    repo.child(".leaf/01-sprouts/research-memo/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/02-leaves/research-memo/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/03-fallen/research-memo")
        .assert(predicate::path::missing());
}

#[test]
fn fall_rejects_blank_reason() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    repo.child(".leaf/01-sprouts/research-memo")
        .create_dir_all()
        .expect("sprout dir");

    leaf_command()
        .current_dir(repo.path())
        .args(["fall", "research-memo", "--reason", "  "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fallen reason cannot be empty"));
}

#[test]
fn init_creates_stage_dirs() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());

    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();

    repo.child(".leaf/01-sprouts")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/02-leaves")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/03-fallen")
        .assert(predicate::path::is_dir());
    repo.child(".leaf/04-pressed")
        .assert(predicate::path::missing());
}

#[test]
fn new_leaves_old_numbered_dirs_in_place_and_creates_sprout() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());
    repo.child(".leaf/01-seeds/old-idea/00-status.md")
        .write_str("- state: seed\n")
        .expect("old legacy state status");
    repo.child(".leaf/02-leaves/active/01-Learn/01-intent.md")
        .write_str("# Intent\n")
        .expect("old leaf intent");

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
    repo.child(".leaf/01-sprouts/fresh/00-status.md")
        .assert(predicate::path::is_file());
}

#[test]
fn new_does_not_migrate_old_numbered_conflicts() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());
    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .write_str("- state: seed\n")
        .expect("old unnumbered legacy state");
    repo.child(".leaf/01-seeds/whatever/00-status.md")
        .write_str("- state: seed\n")
        .expect("old numbered legacy state");

    leaf_command()
        .current_dir(repo.path())
        .arg("new")
        .arg("whatever")
        .assert()
        .success();

    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/01-seeds/whatever/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/01-sprouts/whatever/00-status.md")
        .assert(predicate::path::is_file());
}

#[test]
fn doctor_warns_for_unnumbered_legacy_dirs_without_migrating() {
    let repo = assert_fs::TempDir::new().expect("tempdir");
    git_init(repo.path());
    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .write_str("- state: seed\n")
        .expect("old unnumbered seed");
    repo.child(".leaf/pressed/reference.md")
        .write_str("# Reference\n")
        .expect("old unnumbered pressed");

    leaf_command()
        .current_dir(repo.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("old_stage_dir_present"))
        .stdout(predicate::str::contains(".leaf/seeds"))
        .stdout(predicate::str::contains("pressed_stage_dir_present"))
        .stdout(predicate::str::contains(".leaf/pressed"));

    repo.child(".leaf/seeds/legacy-item/00-status.md")
        .assert(predicate::path::is_file());
    repo.child(".leaf/pressed/reference.md")
        .assert(predicate::path::is_file());
}

#[test]
fn init_creates_global_profile_template() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    let config = assert_fs::TempDir::new().expect("temp config dir");

    leaf_command_with_config(config.path())
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile.md"));

    config
        .child("profile.md")
        .assert(predicate::path::is_file());
    let body = fs::read_to_string(config.path().join("profile.md")).expect("profile readable");
    assert!(
        body.starts_with("# Global Profile"),
        "unexpected template: {body}"
    );
    assert!(body.contains("## User Language"));
}

#[test]
fn init_preserves_existing_global_profile() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    let config = assert_fs::TempDir::new().expect("temp config dir");
    let custom = "# Global Profile\n\n## User Language\n\n- 한국어\n";
    config
        .child("profile.md")
        .write_str(custom)
        .expect("write custom global profile");

    leaf_command_with_config(config.path())
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile.md").not());

    assert_eq!(
        fs::read_to_string(config.path().join("profile.md")).expect("profile readable"),
        custom
    );
}

#[test]
fn profile_prints_global_then_local_with_source_markers() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    let config = assert_fs::TempDir::new().expect("temp config dir");
    config
        .child("profile.md")
        .write_str("## User Language\n\n- 한국어\n")
        .expect("write global profile");
    leaf_command_with_config(config.path())
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();
    repo.child(".leaf/PROFILE.md")
        .write_str("## Settled\n\n- repo fact\n")
        .expect("write local profile");

    let output = leaf_command_with_config(config.path())
        .current_dir(repo.path())
        .arg("profile")
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");

    let global_marker = stdout.find("<!-- global:").expect("global marker present");
    let local_marker = stdout.find("<!-- local:").expect("local marker present");
    assert!(
        global_marker < local_marker,
        "global must precede local:\n{stdout}"
    );
    assert!(
        stdout.contains("- 한국어"),
        "global content missing:\n{stdout}"
    );
    assert!(
        stdout.contains("- repo fact"),
        "local content missing:\n{stdout}"
    );
}

#[test]
fn profile_outside_git_repo_prints_global_only() {
    let dir = assert_fs::TempDir::new().expect("temp non-repo dir");
    let config = assert_fs::TempDir::new().expect("temp config dir");
    config
        .child("profile.md")
        .write_str("## User Language\n\n- 한국어\n")
        .expect("write global profile");

    leaf_command_with_config(config.path())
        .current_dir(dir.path())
        .arg("profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("- 한국어"))
        .stdout(predicate::str::contains("(not in a git repository)"));
}

#[test]
fn profile_marks_missing_global_profile() {
    let repo = assert_fs::TempDir::new().expect("temp repo");
    git_init(repo.path());
    leaf_command()
        .current_dir(repo.path())
        .arg("init")
        .assert()
        .success();
    let empty_config = assert_fs::TempDir::new().expect("temp config dir");

    leaf_command_with_config(empty_config.path())
        .current_dir(repo.path())
        .arg("profile")
        .assert()
        .success()
        .stdout(predicate::str::contains("(missing; run `leaf init`)"));
}
