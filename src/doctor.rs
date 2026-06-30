use crate::fs_ext::{DirectoryStatus, directory_status};
use anyhow::Result;
use std::path::{Path, PathBuf};

mod lifecycle;
mod pressed;
pub(crate) mod srp_sidecar;

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

#[cfg(test)]
pub(crate) fn check(repo_root: &Path) -> Result<DoctorReport> {
    check_with_git_exclude(repo_root, None)
}

pub(crate) fn check_with_git_exclude(
    repo_root: &Path,
    git_exclude: Option<&Path>,
) -> Result<DoctorReport> {
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

    lifecycle::check_stage_dirs(&leaf_root, &mut findings)?;
    lifecycle::check_entries(&leaf_root, &mut findings)?;
    srp_sidecar::check(repo_root, git_exclude, &mut findings);

    Ok(DoctorReport::new(".leaf", findings))
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

    pub(crate) fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = Some(impact.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use std::fs;

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
    fn check_warns_for_leaf_before_feedback() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/early/00-status.md",
            "- stage: leaf\n\
             - current phase: Architect\n\
             - current gate: ⑧ Execution\n\
             - first missing gate: ⑨ Review\n\
             - next action: finish execution\n",
        );

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "leaf_before_feedback",
            Some(Location::Path(".leaf/02-leaves/early/00-status.md".into())),
        );
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
    fn check_accepts_pressed_digest_with_okf_frontmatter() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
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

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.code.starts_with("pressed_frontmatter")),
            "valid pressed digest should not emit frontmatter findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn check_warns_for_pressed_digest_missing_frontmatter() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
            .write_str("# Reference Leaf\n")
            .expect("pressed digest");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "pressed_frontmatter_missing",
            Some(Location::Path(
                ".leaf/02-leaves/reference/pressed.md".into(),
            )),
        );
        let impact = finding.impact.as_deref().expect("frontmatter template");
        assert!(impact.contains("expected frontmatter:"));
        assert!(impact.contains("type: Leaf Pressed Digest"));
        assert!(impact.contains("citation_handle: leaf:{slug}"));
    }

    #[test]
    fn check_warns_for_pressed_digest_missing_fields_and_invalid_type() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
            .write_str(
                "---\n\
                 type: Note\n\
                 title: Reference Leaf\n\
                 ---\n\
                 \n\
                 # Reference Leaf\n",
            )
            .expect("pressed digest");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let missing = assert_finding(
            &report,
            Severity::Warn,
            "pressed_frontmatter_missing_fields",
            Some(Location::Path(
                ".leaf/02-leaves/reference/pressed.md".into(),
            )),
        );
        assert!(missing.message.contains("description"));
        assert!(missing.message.contains("citation_handle"));
        assert!(
            missing
                .impact
                .as_deref()
                .expect("frontmatter template")
                .contains("timestamp: <ISO 8601 local timestamp>")
        );
        let invalid_type = assert_finding(
            &report,
            Severity::Warn,
            "pressed_frontmatter_invalid_type",
            Some(Location::Path(
                ".leaf/02-leaves/reference/pressed.md".into(),
            )),
        );
        assert!(invalid_type.message.contains("Leaf Pressed Digest"));
        assert!(invalid_type.message.contains("Note"));
    }

    #[test]
    fn check_warns_when_pressed_digest_is_not_in_leaves() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/01-sprouts/draft/00-status.md",
            "- stage: sprout\n\
             - current phase: Learn\n\
             - current gate: ② Unknowns & Context\n\
             - first missing gate: ③ Criteria\n\
             - next action: continue\n",
        );
        root.child(".leaf/01-sprouts/draft/pressed.md")
            .write_str(
                "---\n\
                 type: Leaf Pressed Digest\n\
                 title: Draft\n\
                 description: Draft summary.\n\
                 resource: .leaf/01-sprouts/draft\n\
                 tags: [leaf, draft]\n\
                 timestamp: 2026-06-22T10:00:00+09:00\n\
                 citation_handle: leaf:draft\n\
                 stage: leaf\n\
                 ---\n\
                 \n\
                 # Draft\n",
            )
            .expect("pressed digest");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "pressed_digest_wrong_stage",
            Some(Location::Path(".leaf/01-sprouts/draft/pressed.md".into())),
        );
    }

    #[test]
    fn check_warns_for_pressed_digest_frontmatter_stage_not_leaf() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
            .write_str(
                "---\n\
                 type: Leaf Pressed Digest\n\
                 title: Reference Leaf\n\
                 description: One-sentence summary.\n\
                 resource: .leaf/02-leaves/reference\n\
                 tags: [leaf, reference]\n\
                 timestamp: 2026-06-22T10:00:00+09:00\n\
                 citation_handle: leaf:reference\n\
                 stage: sprout\n\
                 ---\n\
                 \n\
                 # Reference Leaf\n",
            )
            .expect("pressed digest");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "pressed_frontmatter_invalid_stage",
            Some(Location::Path(
                ".leaf/02-leaves/reference/pressed.md".into(),
            )),
        );
        assert!(finding.message.contains("leaf"));
        assert!(finding.message.contains("sprout"));
    }

    #[test]
    fn check_accepts_linked_metadata_edges_next_to_pressed_digest() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
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
        root.child(".leaf/02-leaves/reference/linked.md")
            .write_str("# Links\n\n- `cites` -> `okf:spec` - OKF source\n")
            .expect("linked metadata");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.code.starts_with("linked_metadata")),
            "valid linked metadata should not emit findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn check_warns_for_invalid_linked_metadata_edges() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/pressed.md")
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
                 ---\n",
            )
            .expect("pressed digest");
        root.child(".leaf/02-leaves/reference/linked.md")
            .write_str("# Links\n\n- `causes` -> `leaf:other`\n")
            .expect("linked metadata");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "linked_metadata_invalid_edge",
            Some(Location::Path(
                ".leaf/02-leaves/reference/linked.md:3".into(),
            )),
        );
        assert!(finding.message.contains("unknown link predicate"));
        assert!(
            finding
                .impact
                .as_deref()
                .expect("linked template")
                .contains("allowed predicates")
        );
    }

    #[test]
    fn check_warns_for_linked_metadata_without_pressed_digest() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        write_status(
            &root,
            ".leaf/02-leaves/reference/00-status.md",
            "- stage: leaf\n\
             - current phase: Feedback\n\
             - current gate: ⑨ Review\n\
             - first missing gate: ⑩ Retrospect\n\
             - next action: review\n",
        );
        root.child(".leaf/02-leaves/reference/linked.md")
            .write_str("# Links\n\n- `related_to` -> `leaf:other`\n")
            .expect("linked metadata");

        let report = check(root.path()).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "linked_metadata_without_pressed",
            Some(Location::Path(".leaf/02-leaves/reference/linked.md".into())),
        );
    }

    #[test]
    fn check_accepts_valid_srp_sidecar_contract() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child("src/phase.rs")
            .write_str("// phase\n")
            .expect("artifact");
        root.child("src/phase.rs.leaf.toml")
            .write_str(
                r#"
schema = "leaf.srp-sidecar.v1"
artifact = "src/phase.rs"
status = "advisory"
last_verified = "2026-06-26"
responsibility = "Owns LEAF phase ordering, labels, transitions, and polish-boundary checks."
does_not_own = ["File scaffolding; see src/scaffold.rs."]
contracts = ["Phase order is Learn -> Example -> Architect -> Feedback."]
split_signals = ["If this starts creating files, move that to src/scaffold.rs."]
"#,
            )
            .expect("sidecar");
        let exclude = write_srp_exclude(&root, srp_sidecar::EXCLUDE_LINE);

        let report = check_with_git_exclude(root.path(), Some(&exclude)).expect("doctor report");

        assert!(!report.has_errors());
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.code.starts_with("srp_sidecar")),
            "valid sidecar should not emit SRP findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn check_warns_for_srp_sidecar_missing_required_fields() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child("src/phase.rs")
            .write_str("// phase\n")
            .expect("artifact");
        root.child("src/phase.rs.leaf.toml")
            .write_str(
                r#"
schema = "leaf.srp-sidecar.v1"
artifact = "src/phase.rs"
status = "advisory"
"#,
            )
            .expect("sidecar");
        let exclude = write_srp_exclude(&root, srp_sidecar::EXCLUDE_LINE);

        let report = check_with_git_exclude(root.path(), Some(&exclude)).expect("doctor report");

        assert!(!report.has_errors());
        let finding = assert_finding(
            &report,
            Severity::Warn,
            "srp_sidecar_missing_field",
            Some(Location::Path("src/phase.rs.leaf.toml".into())),
        );
        assert!(
            finding.message.contains("last_verified") || finding.message.contains("responsibility")
        );
    }

    #[test]
    fn check_warns_when_srp_sidecar_exclude_pattern_is_missing() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child("src/phase.rs")
            .write_str("// phase\n")
            .expect("artifact");
        root.child("src/phase.rs.leaf.toml")
            .write_str(
                r#"
schema = "leaf.srp-sidecar.v1"
artifact = "src/phase.rs"
status = "advisory"
last_verified = "2026-06-26"
responsibility = "Owns LEAF phase ordering."
"#,
            )
            .expect("sidecar");
        let exclude = write_srp_exclude(&root, "/.leaf");

        let report = check_with_git_exclude(root.path(), Some(&exclude)).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "srp_sidecar_exclude_missing",
            Some(Location::Path(".git/info/exclude".into())),
        );
    }

    #[test]
    fn check_warns_when_srp_sidecar_is_stale() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child("src/phase.rs")
            .write_str("// phase v1\n")
            .expect("artifact");
        root.child("src/phase.rs.leaf.toml")
            .write_str(
                r#"
schema = "leaf.srp-sidecar.v1"
artifact = "src/phase.rs"
status = "advisory"
last_verified = "2026-06-26"
responsibility = "Owns LEAF phase ordering."
"#,
            )
            .expect("sidecar");
        std::thread::sleep(std::time::Duration::from_millis(20));
        root.child("src/phase.rs")
            .write_str("// phase v2\n")
            .expect("artifact update");
        let exclude = write_srp_exclude(&root, srp_sidecar::EXCLUDE_LINE);

        let report = check_with_git_exclude(root.path(), Some(&exclude)).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "srp_sidecar_stale",
            Some(Location::Path("src/phase.rs.leaf.toml".into())),
        );
    }

    #[test]
    fn check_warns_for_srp_sidecar_wrong_schema_and_unknown_fields() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        create_lifecycle_stage_dirs(&root);
        root.child("src/phase.rs")
            .write_str("// phase\n")
            .expect("artifact");
        root.child("src/phase.rs.leaf.toml")
            .write_str(
                r#"
schema = "leaf.srp-sidecar.v0"
artifact = "src/phase.rs"
status = "advisory"
last_verified = "2026-06-26"
responsibility = "Owns LEAF phase ordering."
notes = "This is where sidecars become junk drawers."
"#,
            )
            .expect("sidecar");
        let exclude = write_srp_exclude(&root, srp_sidecar::EXCLUDE_LINE);

        let report = check_with_git_exclude(root.path(), Some(&exclude)).expect("doctor report");

        assert!(!report.has_errors());
        assert_finding(
            &report,
            Severity::Warn,
            "srp_sidecar_invalid_schema",
            Some(Location::Path("src/phase.rs.leaf.toml".into())),
        );
        assert_finding(
            &report,
            Severity::Warn,
            "srp_sidecar_unknown_field",
            Some(Location::Path("src/phase.rs.leaf.toml".into())),
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

    fn write_srp_exclude(root: &assert_fs::TempDir, content: &str) -> PathBuf {
        root.child(".git/info").create_dir_all().expect("git info");
        root.child(".git/info/exclude")
            .write_str(content)
            .expect("exclude");
        root.path().join(".git/info/exclude")
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
