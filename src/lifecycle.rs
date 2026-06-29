use crate::inventory::{Stage, StageDir, parse_status_summary};
use crate::phase::Phase;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct FallResult {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
}

/// Outcome of promoting a sprout into a leaf.
pub(crate) struct KeepResult {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
    /// `Some(phase)` when the sprout had not reached `Feedback` yet, so the
    /// caller can warn that this may be a premature keep. `None` means the work
    /// looked complete (current phase `Feedback`) or had no readable phase.
    pub(crate) premature: Option<String>,
}

/// Promote a completed sprout into a leaf: move `.leaf/01-sprouts/<slug>/` to
/// `.leaf/02-leaves/<slug>/` and rewrite its `00-status.md` stage line. The
/// mirror of [`fall_leaf`]: rename first, then write status, rolling back the
/// move if the write fails so no half-promoted sprout is left behind.
pub(crate) fn keep_leaf(repo_root: &Path, slug: &str) -> Result<KeepResult> {
    let leaf_root = repo_root.join(".leaf");
    let source = leaf_root.join(Stage::Sprout.dir_name()).join(slug);
    let destination = leaf_root.join(Stage::Leaf.dir_name()).join(slug);

    // `real_dir_exists` also refuses a symlink source, mirroring fall_leaf.
    let source_present = real_dir_exists(&source)?;
    // The leaf destination doubles as the "already a leaf" signal: keep's only
    // source candidate is a sprout, so a present destination means the slug is
    // already a leaf — refuse without overwriting. If a sprout *also* exists, the
    // two collide; point at doctor to disambiguate, as fall_leaf does.
    if path_exists(&destination)? {
        if source_present {
            bail!(
                "ambiguous slug: {slug} exists in both {} and {}; run leaf doctor and disambiguate",
                source.display(),
                destination.display()
            );
        }
        bail!(
            "{slug} is already a leaf: {}; nothing to keep",
            destination.display()
        );
    }
    if !source_present {
        bail!("sprout does not exist: checked {}", source.display());
    }

    let status_path = source.join("00-status.md");
    let original = read_optional_status(&status_path)?;
    let premature = original.as_deref().and_then(premature_phase);
    let rewritten = original.as_deref().map(rewrite_stage_to_leaf);

    fs::rename(&source, &destination).with_context(|| {
        format!(
            "failed to move {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    if let Some(content) = rewritten {
        let dest_status = destination.join("00-status.md");
        if let Err(err) = fs::write(&dest_status, content) {
            if let Err(rollback_err) = fs::rename(&destination, &source) {
                return Err(err).with_context(|| {
                    format!(
                        "failed to write {}; also failed to roll back {} to {}: {rollback_err}",
                        dest_status.display(),
                        destination.display(),
                        source.display()
                    )
                });
            }
            return Err(err).with_context(|| {
                format!(
                    "failed to write {}; rolled back move to {}",
                    dest_status.display(),
                    source.display()
                )
            });
        }
    }

    Ok(KeepResult {
        source,
        destination,
        premature,
    })
}

/// Rewrite a sprout status into a leaf status: the preamble `- stage:` value
/// becomes `leaf` and the `# Sprout Status` title becomes `# Leaf Status`. Only
/// the preamble (before the first `##` heading) is touched for the stage field,
/// matching the status parser; every other line is preserved verbatim.
fn rewrite_stage_to_leaf(content: &str) -> String {
    let mut out = Vec::new();
    let mut in_preamble = true;
    for line in content.lines() {
        if in_preamble && line.trim_start().starts_with("##") {
            in_preamble = false;
        }
        if line.trim() == "# Sprout Status" {
            out.push("# Leaf Status".to_string());
        } else if in_preamble && is_stage_field(line) {
            out.push("- stage: leaf".to_string());
        } else {
            out.push(line.to_string());
        }
    }
    let mut result = out.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Match a `- stage:` status field line, case-insensitively.
fn is_stage_field(line: &str) -> bool {
    let trimmed = line.trim_start();
    let trimmed = trimmed.strip_prefix('-').map_or(trimmed, str::trim_start);
    trimmed.to_ascii_lowercase().starts_with("stage:")
}

/// `Some(phase)` when the sprout's `current phase` is not `Feedback`, so keep
/// can warn it may be premature. `None` when it is `Feedback` (looks complete).
/// An unreadable phase reads as premature (`Some("unknown")`): keep can't
/// confirm completion, so it warns rather than staying silent.
fn premature_phase(content: &str) -> Option<String> {
    match parse_status_summary(content, StageDir::Sprouts).current_phase {
        Some(phase) => {
            if Phase::from_status_value(&phase) == Some(Phase::Feedback) {
                None
            } else {
                Some(phase)
            }
        }
        None => Some("unknown".to_string()),
    }
}

pub(crate) fn fall_leaf(repo_root: &Path, slug: &str, reason: &str) -> Result<FallResult> {
    let reason = validate_reason(reason)?;
    let leaf_root = repo_root.join(".leaf");
    let sprout = leaf_root.join(Stage::Sprout.dir_name()).join(slug);
    let leaf = leaf_root.join(Stage::Leaf.dir_name()).join(slug);
    let destination = leaf_root.join(Stage::Fallen.dir_name()).join(slug);

    let sprout_exists = real_dir_exists(&sprout)?;
    let leaf_exists = real_dir_exists(&leaf)?;
    if sprout_exists && leaf_exists {
        bail!(
            "ambiguous leaf slug: {slug} exists in both {} and {}; run leaf doctor and disambiguate",
            sprout.display(),
            leaf.display()
        );
    }

    let source = if sprout_exists {
        sprout
    } else if leaf_exists {
        leaf
    } else {
        bail!(
            "leaf does not exist: checked {} and {}",
            sprout.display(),
            leaf.display()
        );
    };
    if path_exists(&destination)? {
        bail!("fallen leaf already exists: {slug}");
    }

    let status_path = source.join("00-status.md");
    let previous_status = read_optional_status(&status_path)?;

    let timestamp = unix_timestamp()?;
    let status = fallen_status(
        &source,
        &reason,
        &timestamp,
        previous_status.as_deref(),
        repo_root,
    );

    fs::rename(&source, &destination).with_context(|| {
        format!(
            "failed to move {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    let status_path = destination.join("00-status.md");
    if let Err(err) = fs::write(&status_path, status) {
        if let Err(rollback_err) = fs::rename(&destination, &source) {
            return Err(err).with_context(|| {
                format!(
                    "failed to write {}; also failed to roll back {} to {}: {rollback_err}",
                    status_path.display(),
                    destination.display(),
                    source.display()
                )
            });
        }
        return Err(err).with_context(|| {
            format!(
                "failed to write {}; rolled back move to {}",
                status_path.display(),
                source.display()
            )
        });
    }

    Ok(FallResult {
        source,
        destination,
    })
}

fn real_dir_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                bail!(
                    "leaf path is a symlink and cannot be moved: {}",
                    path.display()
                );
            }
            Ok(file_type.is_dir())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn read_optional_status(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn validate_reason(reason: &str) -> Result<String> {
    let reason = reason.trim();
    if reason.is_empty() {
        bail!("fallen reason cannot be empty");
    }
    Ok(reason.to_string())
}

fn unix_timestamp() -> Result<String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the unix epoch")?;
    Ok(format!("unix:{}", duration.as_secs()))
}

fn fallen_status(
    source: &Path,
    reason: &str,
    timestamp: &str,
    previous_status: Option<&str>,
    repo_root: &Path,
) -> String {
    let previous_status = previous_status.unwrap_or("").trim();
    let source_display = source
        .strip_prefix(repo_root)
        .unwrap_or(source)
        .display()
        .to_string();
    let mut status = format!(
        "# Fallen Status\n\n\
         - stage: fallen\n\
         - fallen at: {timestamp}\n\
         - fallen from: {source_display}\n\
         - fallen reason: {reason}\n\
         - closure summary: -\n\
         - reusable lessons: -\n\
         - unresolved limits: -\n\
         - successor: -\n\n\
         ## Fall Log\n\n\
         - {timestamp}: moved to fallen; no execution artifacts were created.\n"
    );

    if !previous_status.is_empty() {
        status.push_str("\n## Previous Status\n\n");
        status.push_str(previous_status);
        status.push('\n');
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_stage_to_leaf_flips_stage_and_title_only() {
        let sprout = "# Sprout Status\n\n\
                      - why: x\n\
                      - stage: sprout\n\
                      - current phase: Feedback\n\
                      - next action: keep\n\n\
                      ## Overview\n\
                      - stage: must-not-touch\n";
        let leaf = rewrite_stage_to_leaf(sprout);
        assert!(leaf.contains("# Leaf Status"));
        assert!(!leaf.contains("# Sprout Status"));
        assert!(leaf.contains("- stage: leaf"));
        // Preamble fields other than stage are preserved verbatim.
        assert!(leaf.contains("- why: x"));
        assert!(leaf.contains("- current phase: Feedback"));
        // A `- stage:` line after the first `##` heading is left untouched.
        assert!(leaf.contains("- stage: must-not-touch"));
        assert!(leaf.ends_with('\n'));
    }

    #[test]
    fn rewrite_stage_to_leaf_handles_mixed_case_and_indent() {
        let sprout = "# Sprout Status\n\n-   Stage:  sprout\n";
        let leaf = rewrite_stage_to_leaf(sprout);
        assert!(leaf.contains("- stage: leaf"));
        assert!(!leaf.to_lowercase().contains("stage:  sprout"));
    }

    #[test]
    fn rewrite_stage_to_leaf_without_trailing_newline() {
        let leaf = rewrite_stage_to_leaf("# Sprout Status\n\n- stage: sprout");
        assert!(leaf.contains("- stage: leaf"));
        assert!(!leaf.ends_with('\n'));
    }

    #[test]
    fn premature_phase_is_none_at_feedback() {
        let status = "- stage: sprout\n- current phase: Feedback\n";
        assert_eq!(premature_phase(status), None);
    }

    #[test]
    fn premature_phase_is_none_for_annotated_feedback() {
        let status = "- stage: sprout\n- current phase: Feedback (⑨ Review)\n";
        assert_eq!(premature_phase(status), None);
    }

    #[test]
    fn premature_phase_reports_earlier_phase() {
        let status = "- stage: sprout\n- current phase: Example\n";
        assert_eq!(premature_phase(status), Some("Example".to_string()));
    }

    #[test]
    fn premature_phase_reports_unknown_when_phase_missing() {
        let status = "- stage: sprout\n- next action: keep\n";
        assert_eq!(premature_phase(status), Some("unknown".to_string()));
    }
}
