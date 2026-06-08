use crate::inventory::BUCKETS;
use anyhow::Result;
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

    match fs::symlink_metadata(&leaf_root) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            findings.push(
                DoctorFinding::ok("leaf_root_present", ".leaf initialized").with_path(".leaf"),
            );
        }
        Ok(_) => {
            findings.push(
                DoctorFinding::error("leaf_root_not_directory", ".leaf is not a directory")
                    .with_path(".leaf"),
            );
            return Ok(DoctorReport::new(".leaf", findings));
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            findings.push(
                DoctorFinding::error("leaf_root_missing", ".leaf is not initialized")
                    .with_path(".leaf"),
            );
            return Ok(DoctorReport::new(".leaf", findings));
        }
        Err(err) => return Err(err.into()),
    }

    check_buckets(&leaf_root, &mut findings)?;

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
        let lifecycle_metadata = fs::symlink_metadata(&lifecycle);
        let lifecycle_is_dir =
            matches!(&lifecycle_metadata, Ok(metadata) if metadata.file_type().is_dir());

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

        match lifecycle_metadata {
            Ok(metadata) if metadata.file_type().is_dir() => {
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
            Ok(_) => {
                all_lifecycle_buckets_readable = false;
                findings.push(
                    DoctorFinding::error(
                        "lifecycle_bucket_not_directory",
                        format!("lifecycle bucket {lifecycle_name} is not a directory"),
                    )
                    .with_path(format!(".leaf/{lifecycle_name}")),
                );
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                all_lifecycle_buckets_readable = false;
                findings.push(
                    DoctorFinding::warn(
                        "lifecycle_bucket_missing",
                        format!("lifecycle bucket {lifecycle_name} is missing"),
                    )
                    .with_path(format!(".leaf/{lifecycle_name}")),
                );
            }
            Err(err) => return Err(err.into()),
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

    fn assert_finding(
        report: &DoctorReport,
        severity: Severity,
        code: &str,
        location: Option<Location>,
    ) {
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.severity == severity && finding.code == code)
            .unwrap_or_else(|| panic!("missing finding {severity:?} {code}"));
        if let Some(location) = location {
            assert_eq!(finding.location, location);
        }
    }
}
