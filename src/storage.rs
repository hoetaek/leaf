use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::git::RepoPaths;
use crate::inventory::STAGES;
use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const EXCLUDE_LINE: &str = "/.leaf";

pub(crate) fn ensure_leaf_root(paths: &RepoPaths) -> Result<bool> {
    let leaf_root = paths.root.join(".leaf");
    let mut changed = false;

    changed |= ensure_directory(&leaf_root)?;
    for stage in STAGES {
        changed |= ensure_directory(&leaf_root.join(stage.dir_name()))?;
    }
    changed |= ensure_exclude_line(&paths.exclude)?;

    Ok(changed)
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
