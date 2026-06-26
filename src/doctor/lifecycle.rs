use super::{DoctorFinding, Severity, pressed};
use crate::fs_ext::{DirectoryStatus, directory_status};
use crate::inventory::{
    OLD_NUMBERED_STAGE_DIRS, Stage, StageDir, parse_status_summary, status_triple_state,
};
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn check_stage_dirs(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    let mut all_stage_dirs_readable = true;

    for stage_dir in stage_dirs() {
        let stage_name = stage_dir.dir_name();
        let stage_dir = leaf_root.join(stage_name);
        let stage_status = directory_status(&stage_dir)?;

        match stage_status {
            DirectoryStatus::Directory => {
                if let Err(err) = fs::read_dir(&stage_dir) {
                    all_stage_dirs_readable = false;
                    findings.push(
                        DoctorFinding::error(
                            "stage_dir_unreadable",
                            format!("failed to read stage dir {stage_name}: {err}"),
                        )
                        .with_path(format!(".leaf/{stage_name}")),
                    );
                }
            }
            DirectoryStatus::NotDirectory => {
                all_stage_dirs_readable = false;
                findings.push(
                    DoctorFinding::error(
                        "stage_dir_not_directory",
                        format!("stage dir {stage_name} is not a directory"),
                    )
                    .with_path(format!(".leaf/{stage_name}")),
                );
            }
            DirectoryStatus::Missing => {
                all_stage_dirs_readable = false;
                findings.push(
                    DoctorFinding::warn(
                        "stage_dir_missing",
                        format!("stage dir {stage_name} is missing"),
                    )
                    .with_path(format!(".leaf/{stage_name}")),
                );
            }
        }
    }

    for stage_dir in OLD_NUMBERED_STAGE_DIRS {
        let Some(old_name) = stage_dir.old_numbered_dir_name() else {
            continue;
        };
        push_legacy_stage_dir_warning(leaf_root, findings, stage_dir, old_name);
    }

    for (stage_dir, old_name) in [
        (StageDir::Sprouts, "seeds"),
        (StageDir::Leaves, "leaves"),
        (StageDir::Fallen, "fallen"),
        (StageDir::Pressed, "pressed"),
    ] {
        push_legacy_stage_dir_warning(leaf_root, findings, stage_dir, old_name);
    }

    if all_stage_dirs_readable {
        findings.push(DoctorFinding::ok(
            "stage_dirs_readable",
            "stage dirs readable",
        ));
    }

    Ok(())
}

fn push_legacy_stage_dir_warning(
    leaf_root: &Path,
    findings: &mut Vec<DoctorFinding>,
    stage_dir: StageDir,
    old_name: &str,
) {
    let old_dir = leaf_root.join(old_name);
    if !old_dir.is_dir() {
        return;
    }

    let (code, message) = match stage_dir {
        StageDir::Pressed => (
            "pressed_stage_dir_present",
            "top-level pressed dir is obsolete; move digests into matching leaf pressed.md"
                .to_string(),
        ),
        _ => (
            "old_stage_dir_present",
            format!("old stage dir {old_name} is present; run the migration operator"),
        ),
    };
    findings.push(DoctorFinding::warn(code, message).with_path(format!(".leaf/{old_name}")));
}

/// Read-only pass over the raw entries of each stage directory.
///
/// Classifies every entry as visible leaf-work or ignored stray, validates the
/// status file of each visible item, and reports slugs that appear in more than
/// one lifecycle stage. Stage-dir problems (missing, unreadable,
/// not-a-directory) are already reported by [`check_stage_dirs`], so unreadable
/// stages are simply skipped here.
pub(super) fn check_entries(leaf_root: &Path, findings: &mut Vec<DoctorFinding>) -> Result<()> {
    // slug -> repo-relative directory paths, accumulated in stage
    // order so duplicate findings list their paths deterministically.
    let mut slug_paths: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

    for stage_dir in stage_dirs() {
        let dir_name = stage_dir.dir_name();
        let stage_dir_path = leaf_root.join(dir_name);

        let mut entries: Vec<(String, PathBuf, fs::FileType)> = Vec::new();
        match fs::read_dir(&stage_dir_path) {
            Ok(read_dir) => {
                for entry in read_dir {
                    let entry = entry?;
                    let file_type = entry.file_type()?;
                    let name = entry.file_name().to_string_lossy().into_owned();
                    entries.push((name, entry.path(), file_type));
                }
            }
            // A missing or unreadable stage dir is reported by check_stage_dirs; skip it.
            Err(_) => continue,
        }

        entries.sort_by(|left, right| left.0.cmp(&right.0));

        for (name, path, file_type) in entries {
            if !file_type.is_dir() {
                findings.push(
                    DoctorFinding::warn(
                        "ignored_stage_entry",
                        format!("ignored non-directory entry in {dir_name}: {name}"),
                    )
                    .with_path(format!(".leaf/{dir_name}/{name}")),
                );
                continue;
            }
            slug_paths
                .entry(name.clone())
                .or_default()
                .push(PathBuf::from(format!(".leaf/{dir_name}/{name}")));
            check_item_status(stage_dir, dir_name, &name, &path, findings);
            check_item_boundary_polish(stage_dir, dir_name, &name, &path, findings);
            pressed::check_item_pressed_digest(stage_dir, dir_name, &name, &path, findings);
            pressed::check_item_linked_metadata(stage_dir, dir_name, &name, &path, findings);
        }
    }

    for paths in slug_paths.into_values() {
        if paths.len() > 1 {
            findings.push(
                DoctorFinding::warn("duplicate_slug", "slug appears in more than one stage")
                    .with_paths(paths),
            );
        }
    }

    Ok(())
}

/// Read and validate `<item>/00-status.md` for one visible leaf-work directory.
fn check_item_status(
    stage_dir: StageDir,
    dir_name: &str,
    slug: &str,
    item_path: &Path,
    findings: &mut Vec<DoctorFinding>,
) {
    let status_path = item_path.join("00-status.md");
    let rel_status = format!(".leaf/{dir_name}/{slug}/00-status.md");

    let content = match fs::read_to_string(&status_path) {
        Ok(content) => content,
        Err(err) => {
            findings.push(
                DoctorFinding::error(
                    "status_unreadable",
                    format!("failed to read status file {rel_status}: {err}"),
                )
                .with_path(rel_status),
            );
            return;
        }
    };

    let summary = parse_status_summary(&content, stage_dir);

    if !summary.missing_fields.is_empty() {
        let labels = summary
            .missing_fields
            .iter()
            .map(|&field| field.label())
            .collect::<Vec<_>>()
            .join(", ");
        let severity = match stage_dir {
            StageDir::Fallen => Severity::Error,
            _ => Severity::Warn,
        };
        findings.push(
            DoctorFinding::new(
                severity,
                "status_missing_fields",
                format!("missing status fields: {labels}"),
            )
            .with_path(rel_status.clone()),
        );
    }

    if summary.legacy_state.is_some() {
        findings.push(
            DoctorFinding::warn(
                "legacy_state_field",
                "status uses old state field; write canonical stage instead",
            )
            .with_path(rel_status.clone()),
        );
    }

    if has_status_field(&content, "fall reason") {
        findings.push(
            DoctorFinding::warn(
                "legacy_fall_reason_field",
                "status uses old fall reason field; write fallen reason instead",
            )
            .with_path(rel_status.clone()),
        );
    }

    if let (Some(expected), Some(actual)) = (expected_stage(stage_dir), summary.stage.as_deref())
        && actual != expected.label()
    {
        findings.push(
            DoctorFinding::error(
                "stage_dir_mismatch",
                format!(
                    "stage {actual} conflicts with directory {dir_name}; expected {}",
                    expected.label()
                ),
            )
            .with_path(rel_status.clone()),
        );
    }

    if stage_dir == StageDir::Leaves
        && summary
            .current_gate
            .as_deref()
            .and_then(parse_gate_index)
            .is_some_and(|gate| gate < 9)
    {
        findings.push(
            DoctorFinding::warn(
                "leaf_before_feedback",
                "leaf stage is for work that has passed ⑧ and entered Feedback",
            )
            .with_path(rel_status.clone()),
        );
    }

    // The why/what/wireframe triple is what the detail header and the status
    // preview surface "at a glance". Sprouts and leaves should carry it — this
    // deliberately includes a pressed leaf (a `02-leaves` item with a
    // `pressed.md`), since a reference-worthy leaf is exactly where the triple
    // matters most. Only the legacy top-level `StageDir::Pressed` dir and
    // `StageDir::Fallen` are exempt. A `none — …` value is a valid
    // understanding-only answer and is not flagged.
    if matches!(stage_dir, StageDir::Sprouts | StageDir::Leaves) {
        let triple = status_triple_state(&content);
        if !triple.missing.is_empty() {
            findings.push(
                DoctorFinding::warn(
                    "status_triple_missing",
                    format!(
                        "status is missing the {} line(s); lock the why/what/wireframe triple with leaf:learn so the preview shows what this leaf is",
                        triple.missing.join(", ")
                    ),
                )
                .with_path(rel_status.clone()),
            );
        }
        // A `TODO` placeholder is acceptable transiently in a sprout (just
        // scaffolded, filled at the Learn-close triple lock); only a leaf — work
        // that has passed ⑧ — shipping a placeholder is a real defect.
        if stage_dir == StageDir::Leaves && !triple.unfilled.is_empty() {
            findings.push(
                DoctorFinding::warn(
                    "status_triple_unfilled",
                    format!(
                        "status triple {} still holds the scaffold placeholder; fill it with leaf:learn",
                        triple.unfilled.join(", ")
                    ),
                )
                .with_path(rel_status.clone()),
            );
        }
    }
}

/// Report any *already-passed* phase that still carries the polish-pending
/// marker (`leaf:polish` removes it). This is the floor under `leaf next`: it
/// makes a skipped boundary polish visible even when `leaf next` was bypassed.
/// Non-blocking (Warn), and only for in-flight or completed work.
fn check_item_boundary_polish(
    stage_dir: StageDir,
    dir_name: &str,
    slug: &str,
    item_path: &Path,
    findings: &mut Vec<DoctorFinding>,
) {
    if stage_dir == StageDir::Fallen {
        return;
    }

    let status_path = item_path.join("00-status.md");
    // status_unreadable is already reported by check_item_status.
    let Ok(content) = fs::read_to_string(&status_path) else {
        return;
    };

    let summary = parse_status_summary(&content, stage_dir);
    let Some(current) = summary
        .current_phase
        .as_deref()
        .and_then(crate::phase::Phase::from_status_value)
    else {
        return;
    };

    for phase in crate::phase::Phase::ORDER {
        if phase.index() >= current.index() {
            break;
        }
        if crate::phase::phase_unpolished(&item_path.join(phase.dir())) {
            findings.push(
                DoctorFinding::warn(
                    "boundary_unpolished",
                    format!(
                        "phase {} was left unpolished before the boundary; run leaf:polish on it, then remove its marker",
                        phase.name()
                    ),
                )
                .with_impact(
                    "경계 polish 누락 — 누적 문서가 하나의 보고서로 다듬어지지 않은 채 다음 phase로 넘어갔다"
                        .to_string(),
                )
                .with_path(format!(".leaf/{dir_name}/{slug}/{}", phase.dir())),
            );
        }
    }
}

fn stage_dirs() -> [StageDir; 3] {
    [StageDir::Sprouts, StageDir::Leaves, StageDir::Fallen]
}

/// The canonical `stage` value expected for items living in `stage_dir`.
fn expected_stage(stage_dir: StageDir) -> Option<Stage> {
    match stage_dir {
        StageDir::Sprouts => Some(Stage::Sprout),
        StageDir::Leaves => Some(Stage::Leaf),
        StageDir::Fallen => Some(Stage::Fallen),
        StageDir::Pressed => None,
    }
}

fn parse_gate_index(value: &str) -> Option<usize> {
    let first = value.trim_start().chars().next()?;
    match first {
        '①' => Some(1),
        '②' => Some(2),
        '③' => Some(3),
        '④' => Some(4),
        '⑤' => Some(5),
        '⑥' => Some(6),
        '⑦' => Some(7),
        '⑧' => Some(8),
        '⑨' => Some(9),
        '⑩' => Some(10),
        ch if ch.is_ascii_digit() => value
            .trim_start()
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok(),
        'g' | 'G' => value
            .trim_start()
            .strip_prefix(['g', 'G'])?
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok(),
        _ => None,
    }
}

fn has_status_field(content: &str, field: &str) -> bool {
    content.lines().any(|line| {
        let Some(rest) = line.trim_start().strip_prefix("- ") else {
            return false;
        };
        let Some((raw_key, _)) = rest.split_once(':') else {
            return false;
        };
        raw_key
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .eq_ignore_ascii_case(field)
    })
}
