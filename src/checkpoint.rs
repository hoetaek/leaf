use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GateSpec {
    pub(crate) index: usize,
    pub(crate) name: &'static str,
    pub(crate) file: &'static str,
    // Folder variants must stay aligned with CANONICAL_SOURCES in review.rs.
    pub(crate) folders: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointResult {
    pub(crate) source: PathBuf,
    pub(crate) checkpoint: PathBuf,
}

const GATES: [GateSpec; 10] = [
    GateSpec {
        index: 1,
        name: "intent",
        file: "01-Learn/01-intent.md",
        folders: &[],
    },
    GateSpec {
        index: 2,
        name: "unknowns",
        file: "01-Learn/02-unknowns.md",
        folders: &[],
    },
    GateSpec {
        index: 3,
        name: "criteria",
        file: "02-Example/03-criteria.md",
        folders: &[],
    },
    GateSpec {
        index: 4,
        name: "wireframe",
        file: "02-Example/04-wireframe.md",
        folders: &["02-Example/04-wireframe"],
    },
    GateSpec {
        index: 5,
        name: "design",
        file: "03-Architect/05-design.md",
        folders: &[],
    },
    GateSpec {
        index: 6,
        name: "critic",
        file: "03-Architect/06-critic.md",
        folders: &[],
    },
    GateSpec {
        index: 7,
        name: "tasks",
        file: "03-Architect/07-tasks.md",
        folders: &[],
    },
    GateSpec {
        index: 8,
        name: "execution",
        file: "03-Architect/08-execution.md",
        folders: &[],
    },
    GateSpec {
        index: 9,
        name: "review",
        file: "04-Feedback/09-review.md",
        folders: &["04-Feedback/09-reviews"],
    },
    GateSpec {
        index: 10,
        name: "retrospect",
        file: "04-Feedback/10-retrospect.md",
        folders: &["04-Feedback/10-retrospective"],
    },
];

pub(crate) fn gate_spec(value: &str) -> Result<GateSpec> {
    let normalized = value.trim().trim_start_matches("--").to_lowercase();
    GATES
        .iter()
        .copied()
        .find(|gate| {
            normalized == gate.name
                || normalized == format!("g{}", gate.index)
                || normalized.parse::<usize>().ok() == Some(gate.index)
                || normalized == gate.file_name_stem()
                || (gate.index == 8 && normalized == "artifact")
        })
        .with_context(|| format!("unknown gate: {value}"))
}

pub(crate) fn create(root_path: &Path, gate: GateSpec) -> Result<Vec<CheckpointResult>> {
    let mut sources = Vec::new();
    let file_source = root_path.join(gate.file);
    if file_source.is_file() {
        sources.push(file_source);
    }
    for folder in gate.folders {
        let folder_source = root_path.join(folder);
        if folder_source.is_dir() {
            sources.push(folder_source);
        }
    }

    if sources.is_empty() {
        let candidates: Vec<String> = std::iter::once(gate.file)
            .chain(gate.folders.iter().copied())
            .map(|relative| root_path.join(relative).display().to_string())
            .collect();
        bail!("gate source does not exist: {}", candidates.join(", "));
    }

    let timestamp = utc_timestamp(SystemTime::now())?;
    sources
        .into_iter()
        .map(|source| checkpoint_source(source, &timestamp))
        .collect()
}

fn checkpoint_source(source: PathBuf, timestamp: &str) -> Result<CheckpointResult> {
    let source_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .context("gate source has no valid UTF-8 name")?;
    let checkpoint = available_checkpoint_path(
        source
            .parent()
            .context("gate source has no parent directory")?,
        timestamp,
        source_name,
    );
    let copy_result = if source.is_dir() {
        copy_dir_recursive(&source, &checkpoint)
    } else {
        fs::copy(&source, &checkpoint).map(|_| ())
    };
    copy_result.with_context(|| {
        format!(
            "failed to checkpoint {} to {}",
            source.display(),
            checkpoint.display()
        )
    })?;

    Ok(CheckpointResult { source, checkpoint })
}

fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_target = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &entry_target)?;
        } else {
            fs::copy(entry.path(), &entry_target)?;
        }
    }
    Ok(())
}

fn available_checkpoint_path(directory: &Path, timestamp: &str, source_file_name: &str) -> PathBuf {
    let first = directory.join(checkpoint_file_name(timestamp, source_file_name, None));
    if !first.exists() {
        return first;
    }

    for counter in 2.. {
        let path = directory.join(checkpoint_file_name(
            timestamp,
            source_file_name,
            Some(counter),
        ));
        if !path.exists() {
            return path;
        }
    }
    unreachable!("unbounded checkpoint suffix search should return");
}

fn checkpoint_file_name(timestamp: &str, source_file_name: &str, counter: Option<usize>) -> String {
    match counter {
        Some(counter) => format!("{timestamp}-{counter} {source_file_name}"),
        None => format!("{timestamp} {source_file_name}"),
    }
}

fn utc_timestamp(time: SystemTime) -> Result<String> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .context("system time is before Unix epoch")?;
    let total_minutes = duration.as_secs() / 60;
    let minute = total_minutes % 60;
    let total_hours = total_minutes / 60;
    let hour = total_hours % 24;
    let days = (total_hours / 24) as i64;
    let (year, month, day) = civil_from_days(days);
    Ok(format!(
        "{:02}{:02}{:02}-{:02}{:02}",
        year.rem_euclid(100),
        month,
        day,
        hour,
        minute
    ))
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

impl GateSpec {
    fn file_name_stem(self) -> &'static str {
        self.file
            .rsplit_once('/')
            .map(|(_, file)| file)
            .unwrap_or(self.file)
            .trim_end_matches(".md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn gate_spec_accepts_names_numbers_and_file_stems() {
        assert_eq!(gate_spec("criteria").expect("criteria").index, 3);
        assert_eq!(gate_spec("3").expect("3").index, 3);
        assert_eq!(gate_spec("03").expect("03").index, 3);
        assert_eq!(gate_spec("g3").expect("g3").index, 3);
        assert_eq!(gate_spec("03-criteria").expect("stem").index, 3);
        assert_eq!(gate_spec("artifact").expect("artifact").index, 8);
    }

    #[test]
    fn checkpoint_copies_gate_file_next_to_source() {
        let root = assert_fs::TempDir::new().expect("temp root");
        root.child("02-Example").create_dir_all().expect("gate dir");
        root.child("02-Example/03-criteria.md")
            .write_str("criteria body\n")
            .expect("gate source");

        let results =
            create(root.path(), gate_spec("criteria").expect("gate")).expect("checkpoint");

        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(
            fs::read_to_string(&result.source).expect("source"),
            "criteria body\n"
        );
        assert_eq!(
            fs::read_to_string(&result.checkpoint).expect("checkpoint"),
            "criteria body\n"
        );
        assert!(
            result
                .checkpoint
                .file_name()
                .and_then(|name| name.to_str())
                .expect("checkpoint name")
                .ends_with(" 03-criteria.md")
        );
    }

    #[test]
    fn checkpoint_copies_wireframe_folder_recursively_when_file_missing() {
        let root = assert_fs::TempDir::new().expect("temp root");
        root.child("02-Example/04-wireframe/index.html")
            .write_str("<html></html>\n")
            .expect("wireframe html");
        root.child("02-Example/04-wireframe/contracts.md")
            .write_str("# Contracts\n")
            .expect("wireframe contracts");

        let results =
            create(root.path(), gate_spec("wireframe").expect("gate")).expect("checkpoint");

        assert_eq!(results.len(), 1);
        let checkpoint = &results[0].checkpoint;
        assert!(checkpoint.is_dir());
        assert!(
            checkpoint
                .file_name()
                .and_then(|name| name.to_str())
                .expect("checkpoint name")
                .ends_with(" 04-wireframe")
        );
        assert_eq!(
            fs::read_to_string(checkpoint.join("index.html")).expect("html copy"),
            "<html></html>\n"
        );
        assert_eq!(
            fs::read_to_string(checkpoint.join("contracts.md")).expect("contracts copy"),
            "# Contracts\n"
        );
    }

    #[test]
    fn checkpoint_copies_file_and_folder_sources_for_review_gate() {
        let root = assert_fs::TempDir::new().expect("temp root");
        root.child("04-Feedback/09-review.md")
            .write_str("# Review\n")
            .expect("review file");
        root.child("04-Feedback/09-reviews/01-first.md")
            .write_str("first review\n")
            .expect("review folder file");

        let results = create(root.path(), gate_spec("review").expect("gate")).expect("checkpoint");

        assert_eq!(results.len(), 2);
        assert!(results[0].checkpoint.is_file());
        assert!(results[1].checkpoint.is_dir());
        assert_eq!(
            fs::read_to_string(results[1].checkpoint.join("01-first.md")).expect("folder copy"),
            "first review\n"
        );
    }

    #[test]
    fn checkpoint_error_lists_file_and_folder_candidates() {
        let root = assert_fs::TempDir::new().expect("temp root");

        let error = create(root.path(), gate_spec("wireframe").expect("gate"))
            .expect_err("missing sources");

        let message = error.to_string();
        assert!(message.contains("02-Example/04-wireframe.md"), "{message}");
        assert!(message.contains("02-Example/04-wireframe"), "{message}");
    }

    #[test]
    fn available_checkpoint_path_suffixes_timestamp_without_overwriting() {
        let root = assert_fs::TempDir::new().expect("temp root");
        root.child("260612-1430 03-criteria.md")
            .write_str("first\n")
            .expect("existing checkpoint");

        let path = available_checkpoint_path(root.path(), "260612-1430", "03-criteria.md");

        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("260612-1430-2 03-criteria.md")
        );
    }
}
