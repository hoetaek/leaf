use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn fall_leaf(repo_root: &Path, slug: &str, reason: &str) -> Result<PathBuf> {
    let reason = validate_reason(reason)?;
    let leaf_root = repo_root.join(".leaf");
    let source = leaf_root.join("leaves").join(slug);
    let destination = leaf_root.join("fallen").join(slug);

    require_directory(&source, "active leaf does not exist")?;
    if destination
        .try_exists()
        .with_context(|| format!("failed to inspect {}", destination.display()))?
    {
        bail!("fallen leaf already exists: {slug}");
    }

    let status_path = source.join("00-status.md");
    let previous_status = match fs::read_to_string(&status_path) {
        Ok(content) => Some(content),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(err).with_context(|| format!("failed to read {}", status_path.display()));
        }
    };

    let timestamp = unix_timestamp()?;
    fs::write(
        &status_path,
        fallen_status(slug, &reason, &timestamp, previous_status.as_deref()),
    )
    .with_context(|| format!("failed to write {}", status_path.display()))?;

    fs::rename(&source, &destination).with_context(|| {
        format!(
            "failed to move {} to {}",
            source.display(),
            destination.display()
        )
    })?;

    Ok(destination)
}

fn validate_reason(reason: &str) -> Result<String> {
    let reason = reason.trim();
    if reason.is_empty() {
        bail!("fall reason cannot be empty");
    }
    Ok(reason.to_string())
}

fn require_directory(path: &Path, missing_message: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(()),
        Ok(_) => bail!("path exists but is not a directory: {}", path.display()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!("{missing_message}: {}", path.display());
        }
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn unix_timestamp() -> Result<String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the unix epoch")?;
    Ok(format!("unix:{}", duration.as_secs()))
}

fn fallen_status(
    slug: &str,
    reason: &str,
    timestamp: &str,
    previous_status: Option<&str>,
) -> String {
    let previous_status = previous_status.unwrap_or("").trim();
    let mut status = format!(
        "# Leaf Status\n\n\
         - state: fallen\n\
         - fallen at: {timestamp}\n\
         - fallen from: .leaf/leaves/{slug}\n\
         - fall reason: {reason}\n\
         - closure summary: -\n\
         - reusable lessons: -\n\
         - unresolved limits: -\n\
         - successor: -\n\n\
         ## Fall Log\n\n\
         - {timestamp}: moved from active leaf to fallen; no execution artifacts were created.\n"
    );

    if !previous_status.is_empty() {
        status.push_str("\n## Previous Status\n\n");
        status.push_str(previous_status);
        status.push('\n');
    }

    status
}
