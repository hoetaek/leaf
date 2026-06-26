use super::DoctorFinding;
use std::fs;
use std::path::{Path, PathBuf};

const SUFFIX: &str = ".leaf.local.toml";
const SCHEMA: &str = "leaf.srp-sidecar.v1";
pub(crate) const EXCLUDE_LINE: &str = "*.leaf.local.toml";
const REQUIRED_FIELDS: &[&str] = &[
    "schema",
    "artifact",
    "status",
    "last_verified",
    "responsibility",
];
const STRING_ARRAY_FIELDS: &[&str] = &["does_not_own", "contracts", "split_signals"];
const ALLOWED_FIELDS: &[&str] = &[
    "schema",
    "artifact",
    "status",
    "last_verified",
    "responsibility",
    "does_not_own",
    "contracts",
    "split_signals",
];

/// Validate strict SRP sidecar contracts (`*.leaf.local.toml`) kept next to
/// artifacts. These are local advisory files, but choosing TOML means `doctor`
/// can at least keep the contract shaped and paired with the current artifact.
pub(super) fn check(
    repo_root: &Path,
    git_exclude: Option<&Path>,
    findings: &mut Vec<DoctorFinding>,
) {
    let mut sidecars = Vec::new();
    collect(repo_root, repo_root, &mut sidecars, findings);

    if sidecars.is_empty() {
        return;
    }

    check_exclude(repo_root, git_exclude, findings);
    for sidecar_path in sidecars {
        check_one(repo_root, &sidecar_path, findings);
    }
}

fn collect(
    repo_root: &Path,
    directory: &Path,
    sidecars: &mut Vec<PathBuf>,
    findings: &mut Vec<DoctorFinding>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_scan_unreadable",
                    format!("failed to scan for SRP sidecars: {err}"),
                )
                .with_path(repo_relative_path(repo_root, directory)),
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }
            collect(repo_root, &path, sidecars, findings);
        } else if file_type.is_file() && name.ends_with(SUFFIX) {
            sidecars.push(path);
        }
    }
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | ".next" | "dist" | "coverage"
    )
}

fn check_exclude(repo_root: &Path, git_exclude: Option<&Path>, findings: &mut Vec<DoctorFinding>) {
    let Some(git_exclude) = git_exclude else {
        return;
    };
    let rel_exclude = repo_relative_path(repo_root, git_exclude);
    let content = match fs::read_to_string(git_exclude) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_exclude_missing",
                    format!(
                        "{EXCLUDE_LINE} is missing from .git/info/exclude; strict SRP sidecars may appear in git status"
                    ),
                )
                .with_path(rel_exclude),
            );
            return;
        }
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_exclude_unreadable",
                    format!("failed to read git exclude for SRP sidecars: {err}"),
                )
                .with_path(rel_exclude),
            );
            return;
        }
    };

    if !content.lines().any(|line| line.trim() == EXCLUDE_LINE) {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_exclude_missing",
                format!(
                    "{EXCLUDE_LINE} is missing from .git/info/exclude; strict SRP sidecars may appear in git status"
                ),
            )
            .with_path(rel_exclude),
        );
    }
}

fn check_one(repo_root: &Path, sidecar_path: &Path, findings: &mut Vec<DoctorFinding>) {
    let rel_sidecar = repo_relative_path(repo_root, sidecar_path);
    let expected_artifact = rel_sidecar.strip_suffix(SUFFIX).unwrap_or(&rel_sidecar);

    let content = match fs::read_to_string(sidecar_path) {
        Ok(content) => content,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_unreadable",
                    format!("failed to read SRP sidecar: {err}"),
                )
                .with_path(rel_sidecar),
            );
            return;
        }
    };

    let document = match toml::from_str::<toml::Value>(&content) {
        Ok(document) => document,
        Err(err) => {
            findings.push(
                DoctorFinding::warn("srp_sidecar_invalid_toml", format!("invalid TOML: {err}"))
                    .with_path(rel_sidecar),
            );
            return;
        }
    };

    let Some(table) = document.as_table() else {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_invalid_shape",
                "SRP sidecar must be a TOML table",
            )
            .with_path(rel_sidecar.clone()),
        );
        return;
    };

    for field in table.keys() {
        if !ALLOWED_FIELDS.contains(&field.as_str()) {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_unknown_field",
                    format!("SRP sidecar field `{field}` is not part of the v1 contract"),
                )
                .with_path(rel_sidecar.clone()),
            );
        }
    }

    for field in REQUIRED_FIELDS {
        if string_field(&document, field).is_none() {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_missing_field",
                    format!("SRP sidecar missing non-empty string field `{field}`"),
                )
                .with_path(rel_sidecar.clone()),
            );
        }
    }

    if let Some(schema) = string_field(&document, "schema")
        && schema != SCHEMA
    {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_invalid_schema",
                format!("SRP sidecar schema must be {SCHEMA:?}, got {schema:?}"),
            )
            .with_path(rel_sidecar.clone()),
        );
    }

    if let Some(status) = string_field(&document, "status")
        && status != "advisory"
    {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_invalid_status",
                format!("SRP sidecar status must be \"advisory\", got {status:?}"),
            )
            .with_path(rel_sidecar.clone()),
        );
    }

    for field in STRING_ARRAY_FIELDS {
        if let Some(value) = document.get(field)
            && !is_string_array(value)
        {
            findings.push(
                DoctorFinding::warn(
                    "srp_sidecar_invalid_array",
                    format!("SRP sidecar field `{field}` must be an array of strings"),
                )
                .with_path(rel_sidecar.clone()),
            );
        }
    }

    let Some(artifact) = string_field(&document, "artifact") else {
        return;
    };
    if !is_repo_relative_artifact_path(artifact) {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_invalid_artifact",
                "SRP sidecar artifact must be a repo-relative path that does not escape the repo",
            )
            .with_path(rel_sidecar.clone()),
        );
        return;
    }

    if artifact != expected_artifact {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_artifact_mismatch",
                format!("SRP sidecar artifact should be {expected_artifact:?}, got {artifact:?}"),
            )
            .with_path(rel_sidecar.clone()),
        );
    }

    let artifact_path = repo_root.join(artifact);
    if !artifact_path.exists() {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_artifact_missing",
                "SRP sidecar points at an artifact that does not exist",
            )
            .with_path(rel_sidecar.clone()),
        );
        return;
    }

    if let (Ok(sidecar_metadata), Ok(artifact_metadata)) =
        (fs::metadata(sidecar_path), fs::metadata(&artifact_path))
        && let (Ok(sidecar_modified), Ok(artifact_modified)) =
            (sidecar_metadata.modified(), artifact_metadata.modified())
        && artifact_modified > sidecar_modified
    {
        findings.push(
            DoctorFinding::warn(
                "srp_sidecar_stale",
                "paired artifact is newer than the SRP sidecar; verify or update the responsibility contract",
            )
            .with_path(rel_sidecar),
        );
    }
}

fn string_field<'a>(document: &'a toml::Value, field: &str) -> Option<&'a str> {
    document
        .get(field)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn is_string_array(value: &toml::Value) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().all(|item| item.as_str().is_some()))
}

fn is_repo_relative_artifact_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path
            .components()
            .all(|component| !matches!(component, std::path::Component::ParentDir))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> String {
    let path = path.strip_prefix(repo_root).unwrap_or(path);
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
