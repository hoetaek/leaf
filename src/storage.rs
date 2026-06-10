use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::git::RepoPaths;
use crate::inventory::{BUCKETS, Bucket};
use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const EXCLUDE_LINE: &str = "/.leaf";

pub(crate) fn ensure_leaf_root(paths: &RepoPaths) -> Result<bool> {
    let leaf_root = paths.root.join(".leaf");
    let mut changed = false;

    changed |= ensure_directory(&leaf_root)?;
    migrate_layout(&leaf_root)?;
    for bucket in BUCKETS {
        changed |= ensure_directory(&leaf_root.join(bucket.dir_name()))?;
    }
    changed |= ensure_exclude_line(&paths.exclude)?;

    Ok(changed)
}

/// Migrate a legacy `.leaf/` workspace to the lifecycle-ordered bucket names.
///
/// Pre-0.3 workspaces named the buckets `seeds`/`leaves`/`fallen`/`pressed`.
/// This renames each surviving legacy directory to its prefixed name with a
/// single `fs::rename`, leaving content untouched. It first scans every bucket
/// and bails before moving anything if both the legacy and current names exist
/// for any bucket, so a partial migration never happens. Returns whether any
/// directory was migrated.
pub(crate) fn migrate_layout(leaf_root: &Path) -> Result<bool> {
    for bucket in BUCKETS {
        let old = leaf_root.join(bucket.legacy_dir_name());
        let new = leaf_root.join(bucket.dir_name());
        if old.is_dir() && new.is_dir() {
            bail!(
                "cannot migrate .leaf/ layout: both '{}' and '{}' exist; merge manually, then re-run. no files were moved.",
                bucket.legacy_dir_name(),
                bucket.dir_name()
            );
        }
    }

    let mut migrated: Vec<Bucket> = Vec::new();
    for bucket in BUCKETS {
        let old = leaf_root.join(bucket.legacy_dir_name());
        let new = leaf_root.join(bucket.dir_name());
        if old.is_dir() && !new.exists() {
            fs::rename(&old, &new).with_context(|| {
                format!("failed to migrate {} to {}", old.display(), new.display())
            })?;
            migrated.push(bucket);
        }
    }

    if !migrated.is_empty() {
        eprintln!("migrated .leaf/ layout to lifecycle order:");
        for bucket in &migrated {
            eprintln!("  {} -> {}", bucket.legacy_dir_name(), bucket.dir_name());
        }
    }

    Ok(!migrated.is_empty())
}

fn ensure_directory(path: &Path) -> Result<bool> {
    match directory_status(path)? {
        DirectoryStatus::Directory => Ok(false),
        DirectoryStatus::NotDirectory => {
            bail!("path exists but is not a directory: {}", path.display())
        }
        DirectoryStatus::Missing => {
            fs::create_dir_all(path)
                .with_context(|| format!("failed to create directory {}", path.display()))?;
            Ok(true)
        }
    }
}

fn ensure_exclude_line(path: &Path) -> Result<bool> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }

    let existing = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read git exclude {}", path.display()));
        }
    };

    if existing.lines().any(|line| line == EXCLUDE_LINE) {
        return Ok(false);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open git exclude {}", path.display()))?;

    if !existing.is_empty() && !existing.ends_with('\n') {
        file.write_all(b"\n")
            .with_context(|| format!("failed to update git exclude {}", path.display()))?;
    }
    file.write_all(EXCLUDE_LINE.as_bytes())
        .with_context(|| format!("failed to update git exclude {}", path.display()))?;
    file.write_all(b"\n")
        .with_context(|| format!("failed to update git exclude {}", path.display()))?;

    Ok(true)
}
