use crate::inventory::{
    Bucket, Inventory, InventoryItem, ItemKind, ParseState, StatusField, StatusSummary,
};
use crate::list_columns::{
    LIST_COLUMNS, ListColumnRow, bucket_label_plural, bucket_label_singular, parse_state_label,
    text_table,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ListRow {
    bucket_label: String,
    slug: String,
    phase: String,
    gate: String,
    parse_state: ParseState,
}

pub(crate) fn write_text(writer: &mut impl Write, inventory: &Inventory) -> Result<()> {
    let rows = list_rows(inventory);

    writeln!(writer, "{}", text_table(&LIST_COLUMNS, &rows))?;

    let empty_buckets: Vec<_> = inventory
        .buckets
        .iter()
        .filter(|bucket| bucket.items.is_empty())
        .map(|bucket| bucket_label_plural(bucket.bucket))
        .collect();
    if !empty_buckets.is_empty() {
        writeln!(writer)?;
        writeln!(writer, "empty: {}", empty_buckets.join(", "))?;
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
        .buckets
        .iter()
        .flat_map(|bucket| bucket.items.iter().map(ListRow::from_item))
        .collect()
}

impl ListRow {
    fn from_item(item: &InventoryItem) -> Self {
        ListRow {
            bucket_label: bucket_label_singular(item.bucket).to_string(),
            slug: item.slug.clone(),
            phase: display_optional(&item.status.current_phase, "-"),
            gate: display_optional(&item.status.current_gate, "-"),
            parse_state: item.status.parse_state,
        }
    }
}

impl ListColumnRow for ListRow {
    fn bucket_label(&self) -> &str {
        &self.bucket_label
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
struct JsonInventory {
    leaf_root: String,
    buckets: JsonBuckets,
}

#[derive(Serialize)]
struct JsonBuckets {
    seeds: JsonBucket,
    leaves: JsonBucket,
    fallen: JsonBucket,
    pressed: JsonBucket,
}

#[derive(Serialize)]
struct JsonBucket {
    count: usize,
    items: Vec<JsonItem>,
}

#[derive(Serialize)]
struct JsonItem {
    bucket: &'static str,
    slug: String,
    kind: &'static str,
    path: String,
    status: JsonStatus,
}

#[derive(Serialize)]
struct JsonStatus {
    parse_state: &'static str,
    state: Option<String>,
    current_phase: Option<String>,
    current_gate: Option<String>,
    first_missing_gate: Option<String>,
    next_action: Option<String>,
    missing_fields: Vec<&'static str>,
}

impl JsonInventory {
    fn from_inventory(inventory: &Inventory) -> Result<Self> {
        Ok(JsonInventory {
            leaf_root: ".leaf".to_string(),
            buckets: JsonBuckets {
                seeds: json_bucket(inventory, Bucket::Seeds)?,
                leaves: json_bucket(inventory, Bucket::Leaves)?,
                fallen: json_bucket(inventory, Bucket::Fallen)?,
                pressed: json_bucket(inventory, Bucket::Pressed)?,
            },
        })
    }
}

fn json_bucket(inventory: &Inventory, bucket: Bucket) -> Result<JsonBucket> {
    let bucket_inventory = inventory
        .buckets
        .iter()
        .find(|inventory| inventory.bucket == bucket)
        .with_context(|| format!("missing inventory bucket {}", bucket_label_plural(bucket)))?;
    let items = bucket_inventory
        .items
        .iter()
        .map(|item| JsonItem::from_item(inventory, item))
        .collect::<Result<Vec<_>>>()?;

    Ok(JsonBucket {
        count: items.len(),
        items,
    })
}

impl JsonItem {
    fn from_item(inventory: &Inventory, item: &InventoryItem) -> Result<Self> {
        Ok(JsonItem {
            bucket: bucket_label_plural(item.bucket),
            slug: item.slug.clone(),
            kind: item_kind_label(item.kind),
            path: relative_leaf_path(inventory, &item.path)?,
            status: JsonStatus::from_summary(&item.status),
        })
    }
}

impl JsonStatus {
    fn from_summary(status: &StatusSummary) -> Self {
        JsonStatus {
            parse_state: parse_state_label(status.parse_state),
            state: status.state.clone(),
            current_phase: status.current_phase.clone(),
            current_gate: status.current_gate.clone(),
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
