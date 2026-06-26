use super::DoctorFinding;
use crate::inventory::{Stage, StageDir};
use std::fs;
use std::path::Path;

const FRONTMATTER_TYPE: &str = "Leaf Pressed Digest";
const FRONTMATTER_FIELDS: &[&str] = &[
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "timestamp",
    "citation_handle",
    "stage",
];
const FRONTMATTER_TEMPLATE: &str = "\
---
type: Leaf Pressed Digest
title: <human-readable title>
description: <one-sentence summary for indexes and previews>
resource: <source path>
tags: [leaf, <short-topic-tag>]
timestamp: <ISO 8601 local timestamp>
citation_handle: leaf:{slug}
stage: leaf
---
";

/// Read and validate optional `<item>/linked.md` graph edges.
pub(super) fn check_item_linked_metadata(
    stage_dir: StageDir,
    dir_name: &str,
    slug: &str,
    item_path: &Path,
    findings: &mut Vec<DoctorFinding>,
) {
    let linked_path = item_path.join("linked.md");
    let rel_linked = format!(".leaf/{dir_name}/{slug}/linked.md");

    let metadata = match fs::metadata(&linked_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "linked_metadata_unreadable",
                    format!("failed to inspect linked metadata: {err}"),
                )
                .with_path(rel_linked),
            );
            return;
        }
    };

    if !metadata.is_file() {
        findings.push(
            DoctorFinding::warn(
                "linked_metadata_not_file",
                "linked.md exists but is not a regular file",
            )
            .with_path(rel_linked),
        );
        return;
    }

    if stage_dir != StageDir::Leaves {
        findings.push(
            DoctorFinding::warn(
                "linked_metadata_wrong_stage",
                "linked.md belongs next to pressed.md in .leaf/02-leaves",
            )
            .with_path(rel_linked.clone()),
        );
    }

    if !item_path.join("pressed.md").is_file() {
        findings.push(
            DoctorFinding::warn(
                "linked_metadata_without_pressed",
                "linked.md should only exist next to a pressed.md digest",
            )
            .with_path(rel_linked.clone()),
        );
    }

    let content = match fs::read_to_string(&linked_path) {
        Ok(content) => content,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "linked_metadata_unreadable",
                    format!("failed to read linked metadata: {err}"),
                )
                .with_path(rel_linked),
            );
            return;
        }
    };

    let mut edge_count = 0usize;
    for (line_index, line) in content.lines().enumerate() {
        match crate::graph::parse_link_line(line) {
            Ok(Some(_)) => edge_count += 1,
            Ok(None) => {}
            Err(message) => {
                findings.push(
                    DoctorFinding::warn("linked_metadata_invalid_edge", message)
                        .with_impact(expected_linked_metadata_message())
                        .with_path(format!("{rel_linked}:{}", line_index + 1)),
                );
            }
        }
    }

    if edge_count == 0 {
        findings.push(
            DoctorFinding::warn(
                "linked_metadata_no_edges",
                "linked.md has no graph edges; remove it or add `predicate -> target` rows",
            )
            .with_impact(expected_linked_metadata_message())
            .with_path(rel_linked),
        );
    }
}

/// Read and validate `<item>/pressed.md` when a visible leaf-work directory has one.
pub(super) fn check_item_pressed_digest(
    stage_dir: StageDir,
    dir_name: &str,
    slug: &str,
    item_path: &Path,
    findings: &mut Vec<DoctorFinding>,
) {
    let digest_path = item_path.join("pressed.md");
    let rel_digest = format!(".leaf/{dir_name}/{slug}/pressed.md");

    let metadata = match fs::metadata(&digest_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "pressed_digest_unreadable",
                    format!("failed to inspect pressed digest: {err}"),
                )
                .with_path(rel_digest),
            );
            return;
        }
    };

    if !metadata.is_file() {
        findings.push(
            DoctorFinding::warn(
                "pressed_digest_not_file",
                "pressed.md exists but is not a regular file",
            )
            .with_path(rel_digest),
        );
        return;
    }

    if stage_dir != StageDir::Leaves {
        findings.push(
            DoctorFinding::warn(
                "pressed_digest_wrong_stage",
                "pressed.md belongs only in .leaf/02-leaves after ⑧ passes",
            )
            .with_path(rel_digest.clone()),
        );
    }

    let content = match fs::read_to_string(&digest_path) {
        Ok(content) => content,
        Err(err) => {
            findings.push(
                DoctorFinding::warn(
                    "pressed_digest_unreadable",
                    format!("failed to read pressed digest: {err}"),
                )
                .with_path(rel_digest),
            );
            return;
        }
    };

    let Some(frontmatter) = parse_yaml_frontmatter(&content) else {
        findings.push(
            DoctorFinding::warn(
                "pressed_frontmatter_missing",
                "pressed.md must start with OKF-compatible YAML frontmatter",
            )
            .with_impact(expected_pressed_frontmatter_message())
            .with_path(rel_digest),
        );
        return;
    };

    let missing_fields = FRONTMATTER_FIELDS
        .iter()
        .copied()
        .filter(|field| !frontmatter_has_key(frontmatter, field))
        .collect::<Vec<_>>();
    if !missing_fields.is_empty() {
        findings.push(
            DoctorFinding::warn(
                "pressed_frontmatter_missing_fields",
                format!(
                    "pressed.md frontmatter missing fields: {}",
                    missing_fields.join(", ")
                ),
            )
            .with_impact(expected_pressed_frontmatter_message())
            .with_path(rel_digest.clone()),
        );
    }

    match frontmatter_value(frontmatter, "type") {
        Some(value) if value == FRONTMATTER_TYPE => {}
        Some(value) => {
            findings.push(
                DoctorFinding::warn(
                    "pressed_frontmatter_invalid_type",
                    format!(
                        "pressed.md frontmatter type must be {FRONTMATTER_TYPE:?}, got {value:?}"
                    ),
                )
                .with_impact(expected_pressed_frontmatter_message())
                .with_path(rel_digest.clone()),
            );
        }
        None => {}
    }

    match frontmatter_value(frontmatter, "stage") {
        Some(value) if value == Stage::Leaf.label() => {}
        Some(value) => {
            findings.push(
                DoctorFinding::warn(
                    "pressed_frontmatter_invalid_stage",
                    format!(
                        "pressed.md frontmatter stage must be {:?}, got {value:?}",
                        Stage::Leaf.label()
                    ),
                )
                .with_impact(expected_pressed_frontmatter_message())
                .with_path(rel_digest),
            );
        }
        None => {}
    }
}

fn parse_yaml_frontmatter(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }

    let start = content.find('\n').map_or(content.len(), |index| index + 1);
    let mut offset = start;
    for line in lines {
        if line.trim() == "---" {
            return content.get(start..offset);
        }
        offset += line.len();
        if content.as_bytes().get(offset) == Some(&b'\n') {
            offset += 1;
        }
    }

    None
}

fn frontmatter_has_key(frontmatter: &str, key: &str) -> bool {
    frontmatter_value(frontmatter, key).is_some()
}

fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        if raw_key.trim() != key {
            continue;
        }
        return Some(unquote_yaml_scalar(raw_value.trim()).to_string());
    }

    None
}

fn unquote_yaml_scalar(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

fn expected_pressed_frontmatter_message() -> String {
    format!("expected frontmatter:\n{FRONTMATTER_TEMPLATE}")
}

fn expected_linked_metadata_message() -> &'static str {
    "expected link rows like `- `cites` -> `leaf:other-slug` - optional note`; allowed predicates: cites, refines, supersedes, depends_on, derived_from, related_to"
}
