use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::inventory::{BUCKETS, Bucket, parse_status_summary};
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

    check_buckets(&leaf_root, &mut findings)?;
    check_entries(&leaf_root, &mut findings)?;

    Ok(DoctorReport::new(".leaf", findings))
}

fn check_buckets(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    let mut all_lifecycle_buckets_readable = true;

    for bucket in BUCKETS {
        let legacy_name = bucket.legacy_dir_name();
        let lifecycle_name = bucket.dir_name();
        let legacy = leaf_root.join(legacy_name);
        let lifecycle = leaf_root.join(lifecycle_name);

        let legacy_is_dir = legacy.is_dir();
        let lifecycle_status = directory_status(&lifecycle)?;
        let lifecycle_is_dir = lifecycle_status == DirectoryStatus::Directory;

        if legacy_is_dir && lifecycle_is_dir {
            all_lifecycle_buckets_readable = false;
            findings.push(
                DoctorFinding::error(
                    "legacy_bucket_conflict",
                    format!("legacy bucket {legacy_name} conflicts with lifecycle bucket {lifecycle_name}"),
                )
                .with_paths([
                    format!(".leaf/{legacy_name}"),
                    format!(".leaf/{lifecycle_name}"),
                ]),
            );
            continue;
        }

        if legacy_is_dir {
            findings.push(
                DoctorFinding::warn(
                    "legacy_bucket_present",
                    format!(
                        "legacy bucket {legacy_name} is present; leaf list will migrate layout"
                    ),
                )
                .with_path(format!(".leaf/{legacy_name}")),
            );
        }

        match lifecycle_status {
            DirectoryStatus::Directory => {
                if let Err(err) = fs::read_dir(&lifecycle) {
                    all_lifecycle_buckets_readable = false;
                    findings.push(
                        DoctorFinding::error(
                            "lifecycle_bucket_unreadable",
                            format!("failed to read lifecycle bucket {lifecycle_name}: {err}"),
                        )
                        .with_path(format!(".leaf/{lifecycle_name}")),
                    );
                }
            }
            DirectoryStatus::NotDirectory => {
                all_lifecycle_buckets_readable = false;
                findings.push(
                    DoctorFinding::error(
                        "lifecycle_bucket_not_directory",
                        format!("lifecycle bucket {lifecycle_name} is not a directory"),
                    )
                    .with_path(format!(".leaf/{lifecycle_name}")),
                );
            }
            DirectoryStatus::Missing => {
                all_lifecycle_buckets_readable = false;
                findings.push(
                    DoctorFinding::warn(
                        "lifecycle_bucket_missing",
                        format!("lifecycle bucket {lifecycle_name} is missing"),
                    )
                    .with_path(format!(".leaf/{lifecycle_name}")),
                );
            }
        }
    }

    if all_lifecycle_buckets_readable {
        findings.push(DoctorFinding::ok(
            "lifecycle_buckets_readable",
            "lifecycle buckets readable",
        ));
    }

    Ok(())
}

/// Read-only pass over the raw entries of each lifecycle bucket.
///
/// Classifies every entry (visible leaf-work directory, pressed digest, or
/// ignored stray), validates the status file of each visible item, and reports
/// slugs that appear in more than one lifecycle bucket. Bucket-level problems
/// (missing, unreadable, not-a-directory) are already reported by
/// [`check_buckets`], so unreadable buckets are simply skipped here.
fn check_entries(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    // slug -> repo-relative directory paths, accumulated in lifecycle-bucket
    // order so duplicate findings list their paths deterministically.
    let mut slug_paths: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for bucket in BUCKETS {
        let dir_name = bucket.dir_name();
        let bucket_dir = leaf_root.join(dir_name);

        let mut entries: Vec<(String, PathBuf, fs::FileType)> = Vec::new();
        match fs::read_dir(&bucket_dir) {
            Ok(read_dir) => {
                for entry in read_dir {
                    let entry = entry?;
                    let file_type = entry.file_type()?;
                    let name = entry.file_name().to_string_lossy().into_owned();
                    entries.push((name, entry.path(), file_type));
                }
            }
            // A missing or unreadable bucket is reported by check_buckets; skip it.
            Err(_) => continue,
        }

        entries.sort_by(|left, right| left.0.cmp(&right.0));

        for (name, path, file_type) in entries {
            match bucket {
                Bucket::Pressed => {
                    let is_md_file = file_type.is_file()
                        && path.extension().and_then(|ext| ext.to_str()) == Some("md");
                    if !is_md_file {
                        findings.push(
                            DoctorFinding::warn(
                                "ignored_pressed_entry",
                                format!("ignored non-digest entry in {dir_name}: {name}"),
                            )
                            .with_path(format!(".leaf/{dir_name}/{name}")),
                        );
                        continue;
                    }
                    // inventory::load reads each digest to project it into
                    // `leaf list`; an unreadable .md surfaces there with
                    // parse_state=error, so doctor reads it too and reports the
                    // failure instead of trusting type/extension alone.
                    if let Err(err) = fs::read_to_string(&path) {
                        findings.push(
                            DoctorFinding::error(
                                "pressed_digest_unreadable",
                                format!("failed to read pressed digest {dir_name}/{name}: {err}"),
                            )
                            .with_path(format!(".leaf/{dir_name}/{name}")),
                        );
                    }
                }
                Bucket::Seeds | Bucket::Leaves | Bucket::Fallen => {
                    if !file_type.is_dir() {
                        findings.push(
                            DoctorFinding::warn(
                                "ignored_lifecycle_entry",
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
                    check_item_status(bucket, dir_name, &name, &path, findings);
                }
            }
        }
    }

    for paths in slug_paths.into_values() {
        if paths.len() > 1 {
            findings.push(
                DoctorFinding::warn(
                    "duplicate_slug",
                    "slug appears in more than one lifecycle bucket",
                )
                .with_paths(paths),
            );
        }
    }

    Ok(())
}

/// Read and validate `<item>/00-status.md` for one visible leaf-work directory.
fn check_item_status(
    bucket: Bucket,
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

    let summary = parse_status_summary(&content, bucket);

    if !summary.missing_fields.is_empty() {
        let labels = summary
            .missing_fields
            .iter()
            .map(|&field| field.label())
            .collect::<Vec<_>>()
            .join(", ");
        let severity = match bucket {
            Bucket::Fallen => Severity::Error,
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

    if let (Some(expected), Some(actual)) = (expected_state(bucket), summary.state.as_deref()) {
        if actual != expected {
            findings.push(
                DoctorFinding::error(
                    "state_bucket_mismatch",
                    format!("state {actual} conflicts with bucket {dir_name}; expected {expected}"),
                )
                .with_path(rel_status),
            );
        }
    }
}

/// The lifecycle `state` value expected for items living in `bucket`.
fn expected_state(bucket: Bucket) -> Option<&'static str> {
    match bucket {
        Bucket::Seeds => Some("seed"),
        Bucket::Leaves => Some("active"),
        Bucket::Fallen => Some("fallen"),
        Bucket::Pressed => None,
    }
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
                DoctorFinding::warn("lifecycle_bucket_missing", "missing bucket")
                    .with_path(".leaf/04-pressed"),
                DoctorFinding::error("leaf_root_not_directory", ".leaf is not a directory")
                    .with_path(".leaf"),
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
        root.child("leaf-store/01-seeds")
            .create_dir_all()
            .expect("seed bucket");
        root.child("leaf-store/02-leaves")
            .create_dir_all()
            .expect("leaf bucket");
        root.child("leaf-store/03-fallen")
            .create_dir_all()
            .expect("fallen bucket");
        root.child("leaf-store/04-pressed")
            .create_dir_all()
            .expect("pressed bucket");
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
    fn check_warns_for_missing_lifecycle_bucket() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-seeds").create_dir_all().expect("seed");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "lifecycle_bucket_missing",
            Some(Location::Path(".leaf/02-leaves".into())),
        );
    }

    #[test]
    fn check_warns_for_legacy_only_bucket_without_migrating() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/seeds/old")
            .create_dir_all()
            .expect("legacy");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "legacy_bucket_present",
            Some(Location::Path(".leaf/seeds".into())),
        );
        assert!(root.path().join(".leaf/seeds").is_dir());
        assert!(!root.path().join(".leaf/01-seeds").exists());
    }

    #[test]
    fn check_errors_when_legacy_and_lifecycle_buckets_conflict() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/seeds").create_dir_all().expect("legacy");
        root.child(".leaf/01-seeds")
            .create_dir_all()
            .expect("lifecycle");

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        assert_finding(
            &report,
            Severity::Error,
            "legacy_bucket_conflict",
            Some(Location::Paths(vec![
                ".leaf/seeds".into(),
                ".leaf/01-seeds".into(),
            ])),
        );
    }

    fn create_lifecycle_buckets(root: &assert_fs::TempDir) {
        for path in [
            ".leaf/01-seeds",
            ".leaf/02-leaves",
            ".leaf/03-fallen",
            ".leaf/04-pressed",
        ] {
            root.child(path).create_dir_all().expect("bucket");
        }
    }

    fn write_status(root: &assert_fs::TempDir, path: &str, body: &str) {
        root.child(path).write_str(body).expect("status");
    }

    #[test]
    fn check_warns_for_partial_seed_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
        write_status(
            &root,
            ".leaf/01-seeds/draft/00-status.md",
            "# Status\n\n- state: seed\n- current phase: Learn\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "status_missing_fields",
            Some(Location::Path(".leaf/01-seeds/draft/00-status.md".into())),
        );
        assert!(finding.message.contains("current_gate"));
        assert!(finding.message.contains("first_missing_gate"));
        assert!(finding.message.contains("next_action"));
    }

    #[test]
    fn check_errors_for_missing_active_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
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
    fn check_errors_for_fallen_status_missing_state() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
        write_status(
            &root,
            ".leaf/03-fallen/closed/00-status.md",
            "# Leaf Status\n\n- fall reason: completed\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Error,
            "status_missing_fields",
            Some(Location::Path(".leaf/03-fallen/closed/00-status.md".into())),
        );
        assert!(finding.message.contains("state"));
    }

    #[test]
    fn check_errors_for_active_state_bucket_mismatch() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
        write_status(
            &root,
            ".leaf/02-leaves/wrong-state/00-status.md",
            "- state: seed\n\
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
            "state_bucket_mismatch",
            Some(Location::Path(
                ".leaf/02-leaves/wrong-state/00-status.md".into(),
            )),
        );
        assert!(
            finding
                .message
                .contains("state seed conflicts with bucket 02-leaves")
        );
        assert!(finding.message.contains("expected active"));
    }

    #[test]
    fn check_warns_for_ignored_lifecycle_and_pressed_entries() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
        root.child(".leaf/01-seeds/loose.md")
            .write_str("ignored\n")
            .expect("loose seed file");
        root.child(".leaf/04-pressed/notes.txt")
            .write_str("ignored\n")
            .expect("pressed txt");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "ignored_lifecycle_entry",
            Some(Location::Path(".leaf/01-seeds/loose.md".into())),
        );
        assert_finding(
            &report,
            Severity::Warn,
            "ignored_pressed_entry",
            Some(Location::Path(".leaf/04-pressed/notes.txt".into())),
        );
    }

    #[test]
    #[cfg(unix)]
    fn check_errors_for_unreadable_pressed_digest() {
        use std::os::unix::fs::PermissionsExt;

        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
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

        assert!(report.has_errors());
        assert_finding(
            &report,
            Severity::Error,
            "pressed_digest_unreadable",
            Some(Location::Path(".leaf/04-pressed/locked.md".into())),
        );
    }

    #[test]
    fn check_warns_for_duplicate_slug_across_lifecycle_buckets() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_buckets(&root);
        write_status(
            &root,
            ".leaf/01-seeds/duplicate/00-status.md",
            "- state: seed\n\
             - current phase: Learn\n\
             - current gate: ② Unknowns & Context\n\
             - first missing gate: ③ Criteria\n\
             - next action: promote\n",
        );
        write_status(
            &root,
            ".leaf/02-leaves/duplicate/00-status.md",
            "- state: active\n\
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
                ".leaf/01-seeds/duplicate".into(),
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
