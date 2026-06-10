use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectoryStatus {
    Directory,
    Missing,
    NotDirectory,
}

pub(crate) fn directory_status(path: &Path) -> Result<DirectoryStatus> {
    match fs::symlink_metadata(path) {
        Ok(_) => match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => Ok(DirectoryStatus::Directory),
            Ok(_) => Ok(DirectoryStatus::NotDirectory),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(DirectoryStatus::NotDirectory)
            }
            Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(DirectoryStatus::Missing),
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}
