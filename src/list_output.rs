use crate::inventory::{
    Inventory, InventoryItem, ItemKind, ParseState, StageDir, StatusField, StatusSummary,
};
use crate::list_columns::{
    LIST_COLUMNS, ListColumnRow, parse_state_label, stage_label_plural, stage_label_singular,
    text_table,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ListRow {
    stage_label: String,
    slug: String,
    phase: String,
    gate: String,
    parse_state: ParseState,
}

pub(crate) fn write_text(writer: &mut impl Write, inventory: &Inventory) -> Result<()> {
    let rows = list_rows(inventory);

    writeln!(writer, "{}", text_table(&LIST_COLUMNS, &rows))?;

    let empty_stages: Vec<_> = inventory
        .stages
        .iter()
        .filter(|stage_dir| stage_dir.stage_dir != StageDir::Pressed)
        .filter(|stage_dir| stage_dir.items.is_empty())
        .map(|stage_dir| stage_label_plural(stage_dir.stage_dir))
        .collect();
    if !empty_stages.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "empty: {}", empty_stages.join(", "))?;
    }

    Ok(())
}

pub(crate) fn write_json(writer: &mut impl Write, inventory: &Inventory) -> Result<()> {
    let output = JsonInventory::from_inventory(inventory)?;
    serde_json::to_writer_pretty(&mut *writer, &output)?;
    writeln!(writer)?;
    Ok(())
}

fn list_rows(inventory: &Inventory) -> Vec<ListRow> {
    inventory
        .stages
        .iter()
        .filter(|stage_dir| stage_dir.stage_dir != StageDir::Pressed)
        .flat_map(|stage_dir| stage_dir.items.iter().map(ListRow::from_item))
        .collect()
}

impl ListRow {
    fn from_item(item: &InventoryItem) -> Self {
        ListRow {
            stage_label: stage_label_singular(item.stage_dir).to_string(),
            slug: item.slug.clone(),
            phase: display_optional(&item.status.current_phase, "-"),
            gate: display_optional(&item.status.current_gate, "-"),
            parse_state: item.status.parse_state,
        }
    }
}

impl ListColumnRow for ListRow {
    fn stage_label(&self) -> &str {
        &self.stage_label
    }

    fn phase(&self) -> &str {
        &self.phase
    }

    fn gate(&self) -> &str {
        &self.gate
    }

    fn slug(&self) -> &str {
        &self.slug
    }

    fn parse_state(&self) -> ParseState {
        self.parse_state
    }
}

fn display_optional(value: &Option<String>, fallback: &str) -> String {
    value.as_deref().unwrap_or(fallback).to_string()
}

#[derive(Serialize)]
pub(crate) struct JsonInventory {
    leaf_root: String,
    stages: JsonStages,
}

#[derive(Serialize)]
struct JsonStages {
    sprouts: JsonStage,
    leaves: JsonStage,
    fallen: JsonStage,
}

#[derive(Serialize)]
struct JsonStage {
    count: usize,
    items: Vec<JsonItem>,
}

#[derive(Serialize)]
struct JsonItem {
    stage: &'static str,
    slug: String,
    kind: &'static str,
    path: String,
    status: JsonStatus,
}

#[derive(Serialize)]
struct JsonStatus {
    parse_state: &'static str,
    stage: Option<String>,
    fallen_reason: Option<String>,
    current_phase: Option<String>,
    current_gate: Option<String>,
    progress_done: usize,
    progress_current: Option<usize>,
    progress_total: usize,
    progress_label: String,
    first_missing_gate: Option<String>,
    next_action: Option<String>,
    missing_fields: Vec<&'static str>,
}

impl JsonInventory {
    pub(crate) fn from_inventory(inventory: &Inventory) -> Result<Self> {
        Ok(JsonInventory {
            leaf_root: ".leaf".to_string(),
            stages: JsonStages {
                sprouts: json_stage(inventory, StageDir::Sprouts)?,
                leaves: json_stage(inventory, StageDir::Leaves)?,
                fallen: json_stage(inventory, StageDir::Fallen)?,
            },
        })
    }
}

fn json_stage(inventory: &Inventory, stage_dir: StageDir) -> Result<JsonStage> {
    let stage_inventory = inventory
        .stages
        .iter()
        .find(|inventory| inventory.stage_dir == stage_dir)
        .with_context(|| format!("missing inventory stage {}", stage_label_plural(stage_dir)))?;
    let items = stage_inventory
        .items
        .iter()
        .map(|item| JsonItem::from_item(inventory, item))
        .collect::<Result<Vec<_>>>()?;

    Ok(JsonStage {
        count: items.len(),
        items,
    })
}

impl JsonItem {
    fn from_item(inventory: &Inventory, item: &InventoryItem) -> Result<Self> {
        Ok(JsonItem {
            stage: stage_label_singular(item.stage_dir),
            slug: item.slug.clone(),
            kind: item_kind_label(item.kind),
            path: relative_leaf_path(inventory, &item.path)?,
            status: JsonStatus::from_summary(&item.status),
        })
    }
}

impl JsonStatus {
    fn from_summary(status: &StatusSummary) -> Self {
        let progress = ProgressJson::from_summary(status);
        JsonStatus {
            parse_state: parse_state_label(status.parse_state),
            stage: status.stage.clone(),
            fallen_reason: status.fallen_reason.clone(),
            current_phase: status.current_phase.clone(),
            current_gate: status.current_gate.clone(),
            progress_done: progress.done,
            progress_current: progress.current,
            progress_total: progress.total,
            progress_label: progress.label,
            first_missing_gate: status.first_missing_gate.clone(),
            next_action: status.next_action.clone(),
            missing_fields: status
                .missing_fields
                .iter()
                .copied()
                .map(StatusField::label)
                .collect(),
        }
    }
}

struct ProgressJson {
    done: usize,
    current: Option<usize>,
    total: usize,
    label: String,
}

impl ProgressJson {
    fn from_summary(status: &StatusSummary) -> Self {
        const TOTAL: usize = 10;
        let text = format!(
            "{} {}",
            status.current_phase.as_deref().unwrap_or(""),
            status.current_gate.as_deref().unwrap_or("")
        );
        if is_terminal_progress(&text) {
            return ProgressJson {
                done: TOTAL,
                current: None,
                total: TOTAL,
                label: format!("{TOTAL}/{TOTAL}"),
            };
        }
        let current = status
            .current_gate
            .as_deref()
            .and_then(crate::review::parse_gate_index);
        let done = current.map_or(0, |gate| gate.saturating_sub(1));
        ProgressJson {
            done,
            current,
            total: TOTAL,
            label: current.map_or_else(|| format!("—/{TOTAL}"), |gate| format!("{gate}/{TOTAL}")),
        }
    }
}

fn is_terminal_progress(text: &str) -> bool {
    let text = text.to_lowercase();
    ["완료", "pressed", "leaf-done", "done", "complete"]
        .iter()
        .any(|needle| text.contains(needle))
}

fn relative_leaf_path(inventory: &Inventory, path: &Path) -> Result<String> {
    let repo_root: PathBuf = inventory
        .leaf_root
        .parent()
        .context("inventory leaf root has no parent")?
        .to_path_buf();
    let relative = path
        .strip_prefix(&repo_root)
        .with_context(|| format!("path {} is outside repo root", path.display()))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn item_kind_label(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::LeafWork => "leaf_work",
        ItemKind::PressedDigest => "pressed_digest",
    }
}
