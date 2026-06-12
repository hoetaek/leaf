use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::inventory::{OLD_NUMBERED_STAGE_DIRS, Stage, StageDir, parse_status_summary};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct DoctorReport {
    pub(crate) leaf_root: PathBuf,
    pub(crate) findings: Vec<DoctorFinding>,
}

#[derive(Debug)]
pub(crate) struct DoctorSummary {
    pub(crate) ok: usize,
    pub(crate) warnings: usize,
    pub(crate) errors: usize,
}

#[derive(Debug)]
pub(crate) struct DoctorFinding {
    pub(crate) severity: Severity,
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) location: Location,
    pub(crate) impact: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Location {
    None,
    Path(PathBuf),
    Paths(Vec<PathBuf>),
}

pub(crate) fn check(repo_root: &Path) -> Result<DoctorReport> {
    let leaf_root = repo_root.join(".leaf");
    let mut findings = Vec::new();

    match directory_status(&leaf_root)? {
        DirectoryStatus::Directory => {
            findings.push(
                DoctorFinding::ok("leaf_root_present", ".leaf initialized").with_path(".leaf"),
            );
        }
        DirectoryStatus::NotDirectory => {
            findings.push(
                DoctorFinding::error("leaf_root_not_directory", ".leaf is not a directory")
                    .with_path(".leaf"),
            );
            return Ok(DoctorReport::new(".leaf", findings));
        }
        DirectoryStatus::Missing => {
            findings.push(
                DoctorFinding::error("leaf_root_missing", ".leaf is not initialized")
                    .with_path(".leaf"),
            );
            return Ok(DoctorReport::new(".leaf", findings));
        }
    }

    check_stage_dirs(&leaf_root, &mut findings)?;
    check_entries(&leaf_root, &mut findings)?;

    Ok(DoctorReport::new(".leaf", findings))
}

fn check_stage_dirs(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    let mut all_stage_dirs_readable = true;

    for stage_dir in stage_dirs() {
        let stage_name = stage_dir.dir_name();
        let stage_dir = leaf_root.join(stage_name);
        let stage_status = directory_status(&stage_dir)?;

        match stage_status {
            DirectoryStatus::Directory => {
                if let Err(err) = fs::read_dir(&stage_dir) {
                    all_stage_dirs_readable = false;
                    findings.push(
                        DoctorFinding::error(
                            "stage_dir_unreadable",
                            format!("failed to read stage dir {stage_name}: {err}"),
                        )
                        .with_path(format!(".leaf/{stage_name}")),
                    );
                }
            }
            DirectoryStatus::NotDirectory => {
                all_stage_dirs_readable = false;
                findings.push(
                    DoctorFinding::error(
                        "stage_dir_not_directory",
                        format!("stage dir {stage_name} is not a directory"),
                    )
                    .with_path(format!(".leaf/{stage_name}")),
                );
            }
            DirectoryStatus::Missing => {
                all_stage_dirs_readable = false;
                findings.push(
                    DoctorFinding::warn(
                        "stage_dir_missing",
                        format!("stage dir {stage_name} is missing"),
                    )
                    .with_path(format!(".leaf/{stage_name}")),
                );
            }
        }
    }

    for stage_dir in OLD_NUMBERED_STAGE_DIRS {
        let Some(old_name) = stage_dir.old_numbered_dir_name() else {
            continue;
        };
        push_legacy_stage_dir_warning(leaf_root, findings, stage_dir, old_name);
    }

    for (stage_dir, old_name) in [
        (StageDir::Sprouts, "seeds"),
        (StageDir::Leaves, "leaves"),
        (StageDir::Fallen, "fallen"),
        (StageDir::Pressed, "pressed"),
    ] {
        push_legacy_stage_dir_warning(leaf_root, findings, stage_dir, old_name);
    }

    if all_stage_dirs_readable {
        findings.push(DoctorFinding::ok(
            "stage_dirs_readable",
            "stage dirs readable",
        ));
    }

    Ok(())
}

fn push_legacy_stage_dir_warning(
    leaf_root: &Path,
    findings: &mut Vec<DoctorFinding>,
    stage_dir: StageDir,
    old_name: &str,
) {
    let old_dir = leaf_root.join(old_name);
    if !old_dir.is_dir() {
        return;
    }

    let (code, message) = match stage_dir {
        StageDir::Pressed => (
            "pressed_stage_dir_present",
            "top-level pressed dir is obsolete; move digests into matching leaf pressed.md"
                .to_string(),
        ),
        _ => (
            "old_stage_dir_present",
            format!("old stage dir {old_name} is present; run the migration operator"),
        ),
    };
    findings.push(DoctorFinding::warn(code, message).with_path(format!(".leaf/{old_name}")));
}

/// Read-only pass over the raw entries of each stage directory.
///
/// Classifies every entry as visible leaf-work or ignored stray, validates the
/// status file of each visible item, and reports
/// slugs that appear in more than one lifecycle stage. Stage-dir problems
/// (missing, unreadable, not-a-directory) are already reported by
/// [`check_stage_dirs`], so unreadable stages are simply skipped here.
fn check_entries(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    // slug -> repo-relative directory paths, accumulated in stage
    // order so duplicate findings list their paths deterministically.
    let mut slug_paths: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for stage_dir in stage_dirs() {
        let dir_name = stage_dir.dir_name();
        let stage_dir_path = leaf_root.join(dir_name);

        let mut entries: Vec<(String, PathBuf, fs::FileType)> = Vec::new();
        match fs::read_dir(&stage_dir_path) {
            Ok(read_dir) => {
                for entry in read_dir {
                    let entry = entry?;
                    let file_type = entry.file_type()?;
                    let name = entry.file_name().to_string_lossy().into_owned();
                    entries.push((name, entry.path(), file_type));
                }
            }
            // A missing or unreadable stage dir is reported by check_stage_dirs; skip it.
            Err(_) => continue,
        }

        entries.sort_by(|left, right| left.0.cmp(&right.0));

        for (name, path, file_type) in entries {
            if !file_type.is_dir() {
                findings.push(
                    DoctorFinding::warn(
                        "ignored_stage_entry",
                        format!("ignored non-directory entry in {dir_name}: {name}"),
                    )
                    .with_path(format!(".leaf/{dir_name}/{name}")),
                );
                continue;
            }
            slug_paths
                .entry(name.clone())
                .or_default()
                .push(PathBuf::from(format!(".leaf/{dir_name}/{name}")));
            check_item_status(stage_dir, dir_name, &name, &path, findings);
        }
    }

    for paths in slug_paths.into_values() {
        if paths.len() > 1 {
            findings.push(
                DoctorFinding::warn("duplicate_slug", "slug appears in more than one stage")
                    .with_paths(paths),
            );
        }
    }

    Ok(())
}

/// Read and validate `<item>/00-status.md` for one visible leaf-work directory.
fn check_item_status(
    stage_dir: StageDir,
    dir_name: &str,
    slug: &str,
    item_path: &Path,
    findings: &mut Vec<DoctorFinding>,
) {
    let status_path = item_path.join("00-status.md");
    let rel_status = format!(".leaf/{dir_name}/{slug}/00-status.md");

    let content = match fs::read_to_string(&status_path) {
        Ok(content) => content,
        Err(err) => {
            findings.push(
                DoctorFinding::error(
                    "status_unreadable",
                    format!("failed to read status file {rel_status}: {err}"),
                )
                .with_path(rel_status),
            );
            return;
        }
    };

    let summary = parse_status_summary(&content, stage_dir);

    if !summary.missing_fields.is_empty() {
        let labels = summary
            .missing_fields
            .iter()
            .map(|&field| field.label())
            .collect::<Vec<_>>()
            .join(", ");
        let severity = match stage_dir {
            StageDir::Fallen => Severity::Error,
            _ => Severity::Warn,
        };
        findings.push(
            DoctorFinding::new(
                severity,
                "status_missing_fields",
                format!("missing status fields: {labels}"),
            )
            .with_path(rel_status.clone()),
        );
    }

    if summary.legacy_state.is_some() {
        findings.push(
            DoctorFinding::warn(
                "legacy_state_field",
                "status uses old state field; write canonical stage instead",
            )
            .with_path(rel_status.clone()),
        );
    }

    if has_status_field(&content, "fall reason") {
        findings.push(
            DoctorFinding::warn(
                "legacy_fall_reason_field",
                "status uses old fall reason field; write fallen reason instead",
            )
            .with_path(rel_status.clone()),
        );
    }

    if let (Some(expected), Some(actual)) = (expected_stage(stage_dir), summary.stage.as_deref())
        && actual != expected.label()
    {
        findings.push(
            DoctorFinding::error(
                "stage_dir_mismatch",
                format!(
                    "stage {actual} conflicts with directory {dir_name}; expected {}",
                    expected.label()
                ),
            )
            .with_path(rel_status),
        );
    }
}

fn stage_dirs() -> [StageDir; 3] {
    [StageDir::Sprouts, StageDir::Leaves, StageDir::Fallen]
}

/// The canonical `stage` value expected for items living in `stage_dir`.
fn expected_stage(stage_dir: StageDir) -> Option<Stage> {
    match stage_dir {
        StageDir::Sprouts => Some(Stage::Sprout),
        StageDir::Leaves => Some(Stage::Leaf),
        StageDir::Fallen => Some(Stage::Fallen),
        StageDir::Pressed => None,
    }
}

fn has_status_field(content: &str, field: &str) -> bool {
    content.lines().any(|line| {
        let Some(rest) = line.trim_start().strip_prefix("- ") else {
            return false;
        };
        let Some((raw_key, _)) = rest.split_once(':') else {
            return false;
        };
        raw_key
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .eq_ignore_ascii_case(field)
    })
}

impl DoctorReport {
    pub(crate) fn new(leaf_root: impl Into<PathBuf>, findings: Vec<DoctorFinding>) -> Self {
        DoctorReport {
            leaf_root: leaf_root.into(),
            findings,
        }
    }

    pub(crate) fn summary(&self) -> DoctorSummary {
        let mut summary = DoctorSummary {
            ok: 0,
            warnings: 0,
            errors: 0,
        };
        for finding in &self.findings {
            match finding.severity {
                Severity::Ok => summary.ok += 1,
                Severity::Warn => summary.warnings += 1,
                Severity::Error => summary.errors += 1,
            }
        }
        summary
    }

    pub(crate) fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| finding.severity == Severity::Error)
    }
}

impl DoctorFinding {
    pub(crate) fn ok(code: &'static str, message: impl Into<String>) -> Self {
        DoctorFinding::new(Severity::Ok, code, message)
    }

    pub(crate) fn warn(code: &'static str, message: impl Into<String>) -> Self {
        DoctorFinding::new(Severity::Warn, code, message)
    }

    pub(crate) fn error(code: &'static str, message: impl Into<String>) -> Self {
        DoctorFinding::new(Severity::Error, code, message)
    }

    fn new(severity: Severity, code: &'static str, message: impl Into<String>) -> Self {
        DoctorFinding {
            severity,
            code,
            message: message.into(),
            location: Location::None,
            impact: None,
        }
    }

    pub(crate) fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location = Location::Path(path.into());
        self
    }

    pub(crate) fn with_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.location = Location::Paths(paths.into_iter().map(Into::into).collect());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn report_counts_findings_by_severity_and_detects_errors() {
        let report = DoctorReport::new(
            ".leaf",
            vec![
                DoctorFinding::ok("leaf_root_present", ".leaf initialized").with_path(".leaf"),
                DoctorFinding::warn("stage_dir_missing", "missing stage dir")
                    .with_path(".leaf/03-fallen"),
                DoctorFinding::error("stage_dir_not_directory", "stage dir is not a directory")
                    .with_path(".leaf/02-leaves"),
            ],
        );

        let summary = report.summary();

        assert_eq!(summary.ok, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.errors, 1);
        assert!(report.has_errors());
    }

    #[test]
    fn check_reports_missing_leaf_root_as_error() {
        let root = assert_fs::TempDir::new().expect("temp repo");

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        assert_finding(
            &report,
            Severity::Error,
            "leaf_root_missing",
            Some(Location::Path(".leaf".into())),
        );
    }

    #[test]
    fn check_reports_leaf_root_file_as_error() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf")
            .write_str("not a directory\n")
            .expect("leaf file");

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        assert_finding(
            &report,
            Severity::Error,
            "leaf_root_not_directory",
            Some(Location::Path(".leaf".into())),
        );
    }

    #[cfg(unix)]
    #[test]
    fn check_accepts_leaf_root_symlink_to_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child("leaf-store/01-sprouts")
            .create_dir_all()
            .expect("sprout stage_dir");
        root.child("leaf-store/02-leaves")
            .create_dir_all()
            .expect("leaf stage_dir");
        root.child("leaf-store/03-fallen")
            .create_dir_all()
            .expect("fallen stage_dir");
        std::os::unix::fs::symlink(root.path().join("leaf-store"), root.path().join(".leaf"))
            .expect("leaf symlink");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Ok,
            "leaf_root_present",
            Some(Location::Path(".leaf".into())),
        );
    }

    #[test]
    fn check_warns_for_missing_stage_dir() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-sprouts")
            .create_dir_all()
            .expect("sprout");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "stage_dir_missing",
            Some(Location::Path(".leaf/02-leaves".into())),
        );
    }

    #[test]
    fn check_warns_for_old_numbered_dir_without_migrating() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-seeds/old")
            .create_dir_all()
            .expect("old numbered");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "old_stage_dir_present",
            Some(Location::Path(".leaf/01-seeds".into())),
        );
        assert!(root.path().join(".leaf/01-seeds").is_dir());
        assert!(!root.path().join(".leaf/01-sprouts").exists());
    }

    #[test]
    fn check_warns_when_old_numbered_and_stage_dirs_coexist() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-seeds").create_dir_all().expect("old");
        root.child(".leaf/01-sprouts")
            .create_dir_all()
            .expect("stage");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "old_stage_dir_present",
            Some(Location::Path(".leaf/01-seeds".into())),
        );
    }

    fn create_lifecycle_stage_dirs(root: &assert_fs::TempDir) {
        for path in [".leaf/01-sprouts", ".leaf/02-leaves", ".leaf/03-fallen"] {
            root.child(path).create_dir_all().expect("stage_dir");
        }
    }

    fn write_status(root: &assert_fs::TempDir, path: &str, body: &str) {
        root.child(path).write_str(body).expect("status");
    }

    #[test]
    fn check_warns_for_partial_sprout_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/01-sprouts/draft/00-status.md",
            "# Status\n\n- stage: sprout\n- current phase: Learn\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "status_missing_fields",
            Some(Location::Path(".leaf/01-sprouts/draft/00-status.md".into())),
        );
        assert!(finding.message.contains("current_gate"));
        assert!(finding.message.contains("first_missing_gate"));
        assert!(finding.message.contains("next_action"));
    }

    #[test]
    fn check_errors_for_missing_active_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child(".leaf/02-leaves/no-status/01-Learn")
            .create_dir_all()
            .expect("leaf");

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        assert_finding(
            &report,
            Severity::Error,
            "status_unreadable",
            Some(Location::Path(
                ".leaf/02-leaves/no-status/00-status.md".into(),
            )),
        );
    }

    #[test]
    fn check_errors_for_fallen_status_missing_reason() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/03-fallen/closed/00-status.md",
            "# Leaf Status\n\n- stage: fallen\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Error,
            "status_missing_fields",
            Some(Location::Path(".leaf/03-fallen/closed/00-status.md".into())),
        );
        assert!(finding.message.contains("fallen_reason"));
    }

    #[test]
    fn check_errors_for_stage_dir_mismatch() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/wrong-stage/00-status.md",
            "- stage: sprout\n\
             - current phase: Example\n\
             - current gate: ③ Criteria\n\
             - first missing gate: ④ Wireframe\n\
             - next action: continue\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Error,
            "stage_dir_mismatch",
            Some(Location::Path(
                ".leaf/02-leaves/wrong-stage/00-status.md".into(),
            )),
        );
        assert!(
            finding
                .message
                .contains("stage sprout conflicts with directory 02-leaves")
        );
        assert!(finding.message.contains("expected leaf"));
    }

    #[test]
    fn check_warns_for_ignored_stage_entry_and_pressed_leftover() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child(".leaf/01-sprouts/loose.md")
            .write_str("ignored\n")
            .expect("loose sprout file");
        root.child(".leaf/04-pressed/notes.txt")
            .write_str("ignored\n")
            .expect("pressed txt");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "ignored_stage_entry",
            Some(Location::Path(".leaf/01-sprouts/loose.md".into())),
        );
        assert_finding(
            &report,
            Severity::Warn,
            "pressed_stage_dir_present",
            Some(Location::Path(".leaf/04-pressed".into())),
        );
    }

    #[test]
    #[cfg(unix)]
    fn check_warns_for_top_level_pressed_digest_leftover() {
        use std::os::unix::fs::PermissionsExt;

        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        let digest = root.child(".leaf/04-pressed/locked.md");
        digest.write_str("# Locked\n").expect("digest");
        // An on-disk .md file that exists but cannot be read: inventory::load
        // surfaces it in `leaf list` with parse_state=error, so doctor must
        // report it rather than treating any .md as a valid digest.
        fs::set_permissions(digest.path(), fs::Permissions::from_mode(0o000))
            .expect("revoke read permission");

        let report = check(root.path()).expect("doctor report");

        // Restore permissions so the TempDir can be cleaned up.
        fs::set_permissions(digest.path(), fs::Permissions::from_mode(0o644))
            .expect("restore permission");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "pressed_stage_dir_present",
            Some(Location::Path(".leaf/04-pressed".into())),
        );
    }

    #[test]
    fn check_warns_for_duplicate_slug_across_lifecycle_stages() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/01-sprouts/duplicate/00-status.md",
            "- stage: sprout\n\
             - current phase: Learn\n\
             - current gate: ② Unknowns & Context\n\
             - first missing gate: ③ Criteria\n\
             - next action: continue\n",
        );
        write_status(
            &root,
            ".leaf/02-leaves/duplicate/00-status.md",
            "- stage: leaf\n\
             - current phase: Architect\n\
             - current gate: ⑦ Tasks\n\
             - first missing gate: ⑧ Artifact\n\
             - next action: implement\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "duplicate_slug",
            Some(Location::Paths(vec![
                ".leaf/01-sprouts/duplicate".into(),
                ".leaf/02-leaves/duplicate".into(),
            ])),
        );
    }

    fn assert_finding<'a>(
        report: &'a DoctorReport,
        severity: Severity,
        code: &str,
        location: Option<Location>,
    ) -> &'a DoctorFinding {
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.severity == severity && finding.code == code)
            .unwrap_or_else(|| panic!("missing finding {severity:?} {code}"));
        if let Some(location) = location {
            assert_eq!(finding.location, location);
        }
        finding
    }
}
