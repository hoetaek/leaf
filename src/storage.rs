use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::git::RepoPaths;
use crate::inventory::STAGES;
use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const EXCLUDE_LINES: &[&str] = &["/.leaf", crate::doctor::srp_sidecar::EXCLUDE_LINE];
const PROFILE_TEMPLATE: &str = "# Profile\n\nLEAF 작업 전체에 적용해야 하는 사용자 언어, 반복 요구, 에이전트 실수,\n오답노트, 재발 방지 교훈, 반복 사실을 짧고 명확하게 모은다. 한 작업에만\n필요한 내용은 gate file, retrospect, pressed에 둔다. PROFILE은 leaf-soul을\n부정하지 않고 구체화한다.\n\n## User Language\n\n- 미정\n\n## Settled\n\n## Provisional\n";

pub(crate) fn ensure_leaf_root(paths: &RepoPaths) -> Result<bool> {
    let leaf_root = paths.root.join(".leaf");
    let mut changed = false;

    changed |= ensure_directory(&leaf_root)?;
    for stage in STAGES {
        changed |= ensure_directory(&leaf_root.join(stage.dir_name()))?;
    }
    changed |= ensure_profile_file(&leaf_root)?;
    changed |= ensure_exclude_lines(&paths.exclude)?;

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

fn ensure_profile_file(leaf_root: &Path) -> Result<bool> {
    let path = leaf_root.join("PROFILE.md");
    match fs::symlink_metadata(&path) {
        Ok(_) => {
            let metadata = fs::metadata(&path)
                .with_context(|| format!("failed to inspect {}", path.display()))?;
            if metadata.is_dir() {
                bail!("path exists but is a directory: {}", path.display());
            }
            Ok(false)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(&path, PROFILE_TEMPLATE)
                .with_context(|| format!("failed to create profile {}", path.display()))?;
            Ok(true)
        }
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

pub(crate) fn ensure_exclude_lines(path: &Path) -> Result<bool> {
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

    let missing_lines = EXCLUDE_LINES
        .iter()
        .copied()
        .filter(|target| !existing.lines().any(|line| line == *target))
        .collect::<Vec<_>>();

    if missing_lines.is_empty() {
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
    for line in missing_lines {
        file.write_all(line.as_bytes())
            .with_context(|| format!("failed to update git exclude {}", path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("failed to update git exclude {}", path.display()))?;
    }

    Ok(true)
}
