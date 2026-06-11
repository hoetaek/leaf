use crate::inventory::Stage;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct FallResult {
    pub(crate) source: PathBuf,
    pub(crate) destination: PathBuf,
}

pub(crate) fn fall_leaf(repo_root: &Path, slug: &str, reason: &str) -> Result<FallResult> {
    let reason = validate_reason(reason)?;
    let leaf_root = repo_root.join(".leaf");
    let sprout = leaf_root.join(Stage::Sprout.dir_name()).join(slug);
    let leaf = leaf_root.join(Stage::Leaf.dir_name()).join(slug);
    let destination = leaf_root.join(Stage::Fallen.dir_name()).join(slug);

    let source = if sprout.is_dir() {
        sprout
    } else if leaf.is_dir() {
        leaf
    } else {
        bail!(
            "leaf does not exist: checked {} and {}",
            sprout.display(),
            leaf.display()
        );
    };
    if destination
        .try_exists()
        .with_context(|| format!("failed to inspect {}", destination.display()))?
    {
        bail!("fallen leaf already exists: {slug}");
    }

    let status_path = source.join("00-status.md");
    let previous_status = read_optional_status(&status_path)?;

    let timestamp = unix_timestamp()?;
    fs::write(
        &status_path,
        fallen_status(
            &source,
            &reason,
            &timestamp,
            previous_status.as_deref(),
            repo_root,
        ),
    )
    .with_context(|| format!("failed to write {}", status_path.display()))?;

    fs::rename(&source, &destination).with_context(|| {
        format!(
            "failed to move {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    Ok(FallResult {
        source,
        destination,
    })
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
