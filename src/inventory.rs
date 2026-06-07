// The inventory projection is consumed by the forthcoming `leaf list` renderer,
// which lands in a later slice; until then this crate-internal API has no caller.
#![allow(dead_code)]

use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

/// A read-only projection of the `.leaf/` workspace, grouped into its four buckets.
#[derive(Debug)]
pub(crate) struct Inventory {
    pub(crate) leaf_root: PathBuf,
    pub(crate) buckets: Vec<BucketInventory>,
}

#[derive(Debug)]
pub(crate) struct BucketInventory {
    pub(crate) bucket: Bucket,
    pub(crate) items: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bucket {
    Seeds,
    Leaves,
    Fallen,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    LeafWork,
    PressedDigest,
}

#[derive(Debug)]
pub(crate) struct InventoryItem {
    pub(crate) bucket: Bucket,
    pub(crate) slug: String,
    pub(crate) kind: ItemKind,
    pub(crate) path: PathBuf,
    pub(crate) status: StatusSummary,
    pub(crate) preview: PreviewSource,
}

#[derive(Debug)]
pub(crate) struct StatusSummary {
    pub(crate) parse_state: ParseState,
    pub(crate) state: Option<String>,
    pub(crate) current_phase: Option<String>,
    pub(crate) current_gate: Option<String>,
    pub(crate) first_missing_gate: Option<String>,
    pub(crate) next_action: Option<String>,
    pub(crate) missing_fields: Vec<StatusField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseState {
    Ok,
    Partial,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusField {
    State,
    CurrentPhase,
    CurrentGate,
    FirstMissingGate,
    NextAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreviewSource {
    LeafWork {
        status_path: PathBuf,
        intent_path: PathBuf,
        unknowns_path: PathBuf,
        criteria_path: PathBuf,
    },
    PressedDigest {
        digest_path: PathBuf,
    },
}

const BUCKETS: [Bucket; 4] = [
    Bucket::Seeds,
    Bucket::Leaves,
    Bucket::Fallen,
    Bucket::Pressed,
];

impl Bucket {
    fn dir_name(self) -> &'static str {
        match self {
            Bucket::Seeds => "seeds",
            Bucket::Leaves => "leaves",
            Bucket::Fallen => "fallen",
            Bucket::Pressed => "pressed",
        }
    }
}

/// Read the `.leaf/` workspace under `repo_root` and project it into an [`Inventory`].
///
/// This never creates directories or files. A missing or non-directory `.leaf/`
/// is an error; missing bucket directories under an existing `.leaf/` are treated
/// as empty buckets.
pub(crate) fn load(repo_root: &Path) -> Result<Inventory> {
    let leaf_root = repo_root.join(".leaf");
    match fs::symlink_metadata(&leaf_root) {
        Ok(metadata) if metadata.file_type().is_dir() => {}
        Ok(_) => bail!(
            "path exists but is not a directory: {}",
            leaf_root.display()
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!(".leaf/ is not initialized in this git repository\nhint: run `leaf init`");
        }
        Err(err) => {
            return Err(err).map_err(|err| {
                anyhow::Error::new(err)
                    .context(format!("failed to inspect {}", leaf_root.display()))
            });
        }
    }

    let buckets = BUCKETS
        .iter()
        .map(|&bucket| load_bucket(&leaf_root, bucket))
        .collect::<Result<Vec<_>>>()?;

    Ok(Inventory { leaf_root, buckets })
}

fn load_bucket(leaf_root: &Path, bucket: Bucket) -> Result<BucketInventory> {
    let bucket_dir = leaf_root.join(bucket.dir_name());
    let entries = match fs::read_dir(&bucket_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BucketInventory {
                bucket,
                items: Vec::new(),
            });
        }
        Err(err) => {
            return Err(err).map_err(|err| {
                anyhow::Error::new(err).context(format!("failed to read {}", bucket_dir.display()))
            });
        }
    };

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            anyhow::Error::new(err)
                .context(format!("failed to read entry in {}", bucket_dir.display()))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            anyhow::Error::new(err).context(format!("failed to inspect {}", entry.path().display()))
        })?;

        if let Some(item) = project_entry(bucket, file_type, entry.path()) {
            items.push(item);
        }
    }

    items.sort_by(|left, right| left.slug.cmp(&right.slug));
    Ok(BucketInventory { bucket, items })
}

fn project_entry(bucket: Bucket, file_type: fs::FileType, path: PathBuf) -> Option<InventoryItem> {
    match bucket {
        Bucket::Pressed => {
            if !file_type.is_file() {
                return None;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return None;
            }
            let slug = path.file_stem()?.to_str()?.to_string();
            Some(load_pressed_item(bucket, slug, path))
        }
        Bucket::Seeds | Bucket::Leaves | Bucket::Fallen => {
            if !file_type.is_dir() {
                return None;
            }
            let slug = path.file_name()?.to_str()?.to_string();
            Some(load_directory_item(bucket, slug, path))
        }
    }
}

fn load_directory_item(bucket: Bucket, slug: String, path: PathBuf) -> InventoryItem {
    let status_path = path.join("00-status.md");
    let status = match fs::read_to_string(&status_path) {
        Ok(content) => parse_status_summary(&content, bucket),
        Err(_) => StatusSummary::error(),
    };

    let preview = PreviewSource::LeafWork {
        status_path,
        intent_path: path.join("01-Learn/01-intent.md"),
        unknowns_path: path.join("01-Learn/02-unknowns.md"),
        criteria_path: path.join("02-Example/03-criteria.md"),
    };

    InventoryItem {
        bucket,
        slug,
        kind: ItemKind::LeafWork,
        path,
        status,
        preview,
    }
}

fn load_pressed_item(bucket: Bucket, slug: String, path: PathBuf) -> InventoryItem {
    let status = match fs::read_to_string(&path) {
        Ok(_) => parse_status_summary("", bucket),
        Err(_) => StatusSummary::error(),
    };

    let preview = PreviewSource::PressedDigest {
        digest_path: path.clone(),
    };

    InventoryItem {
        bucket,
        slug,
        kind: ItemKind::PressedDigest,
        path,
        status,
        preview,
    }
}

impl StatusSummary {
    fn error() -> Self {
        StatusSummary {
            parse_state: ParseState::Error,
            state: None,
            current_phase: None,
            current_gate: None,
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        }
    }
}

/// Expected fields per bucket, in display order.
fn expected_fields(bucket: Bucket) -> &'static [StatusField] {
    match bucket {
        Bucket::Seeds | Bucket::Leaves => &[
            StatusField::State,
            StatusField::CurrentPhase,
            StatusField::CurrentGate,
            StatusField::FirstMissingGate,
            StatusField::NextAction,
        ],
        Bucket::Fallen => &[StatusField::State],
        Bucket::Pressed => &[],
    }
}

/// Parse the recognized `- key: value` lines out of a status document.
///
/// Keys are matched case-insensitively with internal whitespace collapsed.
/// `Pressed` digests carry no status fields and always parse as [`ParseState::Ok`].
pub(crate) fn parse_status_summary(content: &str, bucket: Bucket) -> StatusSummary {
    if matches!(bucket, Bucket::Pressed) {
        return StatusSummary {
            parse_state: ParseState::Ok,
            state: None,
            current_phase: None,
            current_gate: None,
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        };
    }

    let mut state = None;
    let mut current_phase = None;
    let mut current_gate = None;
    let mut first_missing_gate = None;
    let mut next_action = None;

    for line in content.lines() {
        let Some((key, value)) = parse_field_line(line) else {
            continue;
        };
        match key.as_str() {
            "state" => state = Some(value),
            "current phase" => current_phase = Some(value),
            "current gate" => current_gate = Some(value),
            "first missing gate" => first_missing_gate = Some(value),
            "next action" => next_action = Some(value),
            _ => {}
        }
    }

    let value_for = |field: StatusField| match field {
        StatusField::State => &state,
        StatusField::CurrentPhase => &current_phase,
        StatusField::CurrentGate => &current_gate,
        StatusField::FirstMissingGate => &first_missing_gate,
        StatusField::NextAction => &next_action,
    };

    let missing_fields: Vec<StatusField> = expected_fields(bucket)
        .iter()
        .copied()
        .filter(|&field| value_for(field).is_none())
        .collect();

    let parse_state = if missing_fields.is_empty() {
        ParseState::Ok
    } else {
        ParseState::Partial
    };

    StatusSummary {
        parse_state,
        state,
        current_phase,
        current_gate,
        first_missing_gate,
        next_action,
        missing_fields,
    }
}

/// Parse one `- key: value` line into a normalized `(key, value)` pair.
///
/// Returns `None` for any line not of that exact shape.
fn parse_field_line(line: &str) -> Option<(String, String)> {
    let rest = line.trim_start().strip_prefix("- ")?;
    let (raw_key, raw_value) = rest.split_once(':')?;
    let key = normalize_key(raw_key);
    if key.is_empty() {
        return None;
    }
    Some((key, raw_value.trim().to_string()))
}

fn normalize_key(raw_key: &str) -> String {
    raw_key
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn full_status() -> &'static str {
        "# Leaf Status\n\n\
         - state: active\n\
         - current phase: Architect\n\
         - current gate: G3\n\
         - first missing gate: G4\n\
         - next action: write design\n"
    }

    #[test]
    fn inventory_load_errors_when_leaf_root_is_missing() {
        let root = assert_fs::TempDir::new().expect("temp repo");

        let err = load(root.path()).expect_err("missing .leaf must error");

        let message = format!("{err}");
        assert!(
            message.contains(".leaf/ is not initialized in this git repository"),
            "got: {message}"
        );
        assert!(message.contains("leaf init"), "got: {message}");
    }

    #[test]
    fn inventory_load_errors_when_leaf_is_not_a_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf")
            .write_str("not a directory\n")
            .expect("leaf file");

        let err = load(root.path()).expect_err("non-directory .leaf must error");

        assert!(
            format!("{err}").contains("path exists but is not a directory"),
            "got: {err}"
        );
    }

    #[test]
    fn inventory_load_returns_four_buckets_in_order_even_when_empty() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");

        let inventory = load(root.path()).expect("load inventory");

        assert_eq!(inventory.leaf_root, root.path().join(".leaf"));
        assert_eq!(inventory.buckets.len(), 4);
        assert_eq!(inventory.buckets[0].bucket, Bucket::Seeds);
        assert_eq!(inventory.buckets[1].bucket, Bucket::Leaves);
        assert_eq!(inventory.buckets[2].bucket, Bucket::Fallen);
        assert_eq!(inventory.buckets[3].bucket, Bucket::Pressed);
        for bucket in &inventory.buckets {
            assert!(bucket.items.is_empty(), "expected empty bucket");
        }
    }

    #[test]
    fn inventory_load_does_not_create_missing_bucket_directories() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");

        load(root.path()).expect("load inventory");

        assert!(!root.path().join(".leaf/seeds").exists());
        assert!(!root.path().join(".leaf/leaves").exists());
        assert!(!root.path().join(".leaf/fallen").exists());
        assert!(!root.path().join(".leaf/pressed").exists());
    }

    #[test]
    fn inventory_load_lists_only_directories_in_seeds_sorted_by_slug() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/seeds/zebra/00-status.md")
            .write_str(full_status())
            .expect("zebra");
        root.child(".leaf/seeds/apple/00-status.md")
            .write_str(full_status())
            .expect("apple");
        root.child(".leaf/seeds/loose.md")
            .write_str("stray file\n")
            .expect("loose file");

        let inventory = load(root.path()).expect("load inventory");

        let slugs: Vec<_> = inventory.buckets[0]
            .items
            .iter()
            .map(|item| item.slug.as_str())
            .collect();
        assert_eq!(slugs, vec!["apple", "zebra"]);
    }

    #[test]
    fn inventory_load_pressed_lists_only_md_files() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/seeds").create_dir_all().expect("seeds");
        root.child(".leaf/leaves").create_dir_all().expect("leaves");
        root.child(".leaf/fallen").create_dir_all().expect("fallen");
        root.child(".leaf/pressed/note.txt")
            .write_str("not a digest\n")
            .expect("note");
        root.child(".leaf/pressed/real.md")
            .write_str("# Real\n")
            .expect("digest");

        let inventory = load(root.path()).expect("load inventory");

        assert_eq!(inventory.buckets[3].items.len(), 1);
        assert_eq!(inventory.buckets[3].items[0].slug, "real");
    }

    #[test]
    fn inventory_leaf_item_has_leafwork_kind_and_preview_paths() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/leaves/demo/00-status.md")
            .write_str(full_status())
            .expect("status");

        let inventory = load(root.path()).expect("load inventory");
        let item = &inventory.buckets[1].items[0];

        assert_eq!(item.bucket, Bucket::Leaves);
        assert_eq!(item.slug, "demo");
        assert_eq!(item.kind, ItemKind::LeafWork);
        assert_eq!(item.path, root.path().join(".leaf/leaves/demo"));
        assert_eq!(item.status.parse_state, ParseState::Ok);
        assert_eq!(item.status.state.as_deref(), Some("active"));
        assert_eq!(item.status.current_phase.as_deref(), Some("Architect"));
        assert_eq!(item.status.next_action.as_deref(), Some("write design"));

        match &item.preview {
            PreviewSource::LeafWork {
                status_path,
                intent_path,
                unknowns_path,
                criteria_path,
            } => {
                let base = root.path().join(".leaf/leaves/demo");
                assert_eq!(status_path, &base.join("00-status.md"));
                assert_eq!(intent_path, &base.join("01-Learn/01-intent.md"));
                assert_eq!(unknowns_path, &base.join("01-Learn/02-unknowns.md"));
                assert_eq!(criteria_path, &base.join("02-Example/03-criteria.md"));
            }
            other => panic!("expected LeafWork preview, got {other:?}"),
        }
    }

    #[test]
    fn inventory_leaf_item_without_status_is_visible_with_error_state() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/leaves/no-status/01-Learn")
            .create_dir_all()
            .expect("dir without status");

        let inventory = load(root.path()).expect("load inventory");
        let item = &inventory.buckets[1].items[0];

        assert_eq!(item.slug, "no-status");
        assert_eq!(item.kind, ItemKind::LeafWork);
        assert_eq!(item.status.parse_state, ParseState::Error);
        assert!(item.status.state.is_none());
        assert!(item.status.current_phase.is_none());
        assert!(item.status.next_action.is_none());
        assert!(item.status.missing_fields.is_empty());
    }

    #[test]
    fn inventory_pressed_digest_has_digest_kind_and_preview() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/pressed/summary.md")
            .write_str("# Summary\n")
            .expect("digest");

        let inventory = load(root.path()).expect("load inventory");
        let item = &inventory.buckets[3].items[0];

        assert_eq!(item.slug, "summary");
        assert_eq!(item.bucket, Bucket::Pressed);
        assert_eq!(item.kind, ItemKind::PressedDigest);
        assert_eq!(item.status.parse_state, ParseState::Ok);
        assert!(item.status.state.is_none());

        match &item.preview {
            PreviewSource::PressedDigest { digest_path } => {
                assert_eq!(digest_path, &root.path().join(".leaf/pressed/summary.md"));
            }
            other => panic!("expected PressedDigest preview, got {other:?}"),
        }
    }

    #[test]
    fn inventory_parse_status_summary_ok_when_all_expected_present() {
        let summary = parse_status_summary(full_status(), Bucket::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert!(summary.missing_fields.is_empty());
        assert_eq!(summary.state.as_deref(), Some("active"));
        assert_eq!(summary.current_phase.as_deref(), Some("Architect"));
        assert_eq!(summary.current_gate.as_deref(), Some("G3"));
        assert_eq!(summary.first_missing_gate.as_deref(), Some("G4"));
        assert_eq!(summary.next_action.as_deref(), Some("write design"));
    }

    #[test]
    fn inventory_parse_status_summary_partial_lists_missing_fields() {
        let content = "- state: seed\n- current phase: Learn\n";

        let summary = parse_status_summary(content, Bucket::Seeds);

        assert_eq!(summary.parse_state, ParseState::Partial);
        assert_eq!(summary.state.as_deref(), Some("seed"));
        assert_eq!(summary.current_phase.as_deref(), Some("Learn"));
        assert!(summary.current_gate.is_none());
        assert!(!summary.missing_fields.contains(&StatusField::State));
        assert!(!summary.missing_fields.contains(&StatusField::CurrentPhase));
        assert!(summary.missing_fields.contains(&StatusField::CurrentGate));
        assert!(
            summary
                .missing_fields
                .contains(&StatusField::FirstMissingGate)
        );
        assert!(summary.missing_fields.contains(&StatusField::NextAction));
    }

    #[test]
    fn inventory_parse_status_summary_normalizes_keys_and_ignores_unknown() {
        let content = "- State:  active\n\
                       - Current   Phase: Learn\n\
                       - CURRENT GATE: G1\n\
                       - First Missing Gate: G2\n\
                       - Next Action: do it: now\n\
                       - random key: whatever\n";

        let summary = parse_status_summary(content, Bucket::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.state.as_deref(), Some("active"));
        assert_eq!(summary.current_phase.as_deref(), Some("Learn"));
        assert_eq!(summary.current_gate.as_deref(), Some("G1"));
        assert_eq!(summary.first_missing_gate.as_deref(), Some("G2"));
        assert_eq!(summary.next_action.as_deref(), Some("do it: now"));
    }

    #[test]
    fn inventory_parse_status_summary_fallen_expects_only_state() {
        let summary = parse_status_summary("- state: fallen\n", Bucket::Fallen);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.state.as_deref(), Some("fallen"));
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_fallen_partial_when_state_missing() {
        let summary = parse_status_summary("- current phase: x\n", Bucket::Fallen);

        assert_eq!(summary.parse_state, ParseState::Partial);
        assert_eq!(summary.missing_fields, vec![StatusField::State]);
    }

    #[test]
    fn inventory_parse_status_summary_pressed_is_ok_with_no_fields() {
        let summary = parse_status_summary("anything at all\n- state: x\n", Bucket::Pressed);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert!(summary.state.is_none());
        assert!(summary.current_phase.is_none());
        assert!(summary.missing_fields.is_empty());
    }
}
