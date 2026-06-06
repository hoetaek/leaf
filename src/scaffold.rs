use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const DIRECTORIES: &[&str] = &[
    "01-Learn",
    "01-Learn/02-references",
    "02-Example",
    "03-Architect",
    "04-Feedback",
];

const FILES: &[(&str, &str)] = &[
    (
        "00-status.md",
        "# Seed Status\n\n- state: seed\n- current phase: Learn\n",
    ),
    (
        "01-Learn/01-intent.md",
        "# Intent\n\nCapture the raw idea and the current one-sentence intent here.\n",
    ),
    (
        "01-Learn/02-unknowns.md",
        "# Unknowns And Context\n\n## Domain concepts\n\n## Standards and conventions\n\n## External facts\n\n## Internal facts\n",
    ),
    (
        "01-Learn/02-references/README.md",
        "# References\n\nStore bulky source material here and summarize useful facts in ../02-unknowns.md.\n",
    ),
    (
        "02-Example/03-criteria.md",
        "# Criteria\n\nState purpose, constraints, and observable acceptance checks here.\n",
    ),
    (
        "02-Example/04-wireframe.md",
        "# Wireframe\n\nUse a concrete text-first example before generalizing the work.\n",
    ),
    (
        "03-Architect/05-design.md",
        "# Design\n\nRecord the implementation-facing design after the concrete example holds.\n",
    ),
    (
        "03-Architect/07-tasks.md",
        "# Tasks\n\nBreak the work into reviewable implementation slices.\n",
    ),
];

pub(crate) fn create_seed(repo_root: &Path, slug: &str) -> Result<PathBuf> {
    let seed = repo_root.join(".leaf").join("seeds").join(slug);
    match fs::create_dir(&seed) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            bail!("leaf seed already exists: {slug}");
        }
        Err(err) => {
            return Err(err).with_context(|| format!("failed to create seed {}", seed.display()));
        }
    }

    for directory in DIRECTORIES {
        let path = seed.join(directory);
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create directory {}", path.display()))?;
    }

    for (relative_path, body) in FILES {
        write_new_file(&seed.join(relative_path), body)?;
    }

    Ok(seed)
}

fn write_new_file(path: &Path, body: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("failed to create file {}", path.display()))?;
    file.write_all(body.as_bytes())
        .with_context(|| format!("failed to write file {}", path.display()))?;
    Ok(())
}
