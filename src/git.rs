use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub(crate) struct RepoPaths {
    pub(crate) root: PathBuf,
    pub(crate) exclude: PathBuf,
}

pub(crate) fn repo_paths(cwd: impl AsRef<Path>) -> Result<RepoPaths> {
    let cwd = cwd.as_ref();
    let root = git_output(cwd, &["rev-parse", "--show-toplevel"])
        .context("not inside a git repository")?;
    let exclude = git_output(
        cwd,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            "info/exclude",
        ],
    )
    .context("failed to locate git info/exclude")?;

    Ok(RepoPaths {
        root: PathBuf::from(root),
        exclude: PathBuf::from(exclude),
    })
}

fn git_output(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        if stderr.is_empty() {
            bail!("git {} failed", args.join(" "));
        }
        bail!("git {} failed: {stderr}", args.join(" "));
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("git {} returned non-utf8 output", args.join(" ")))?;
    let value = stdout.trim().to_string();
    if value.is_empty() {
        bail!("git {} returned empty output", args.join(" "));
    }
    Ok(value)
}
