use crate::fs_ext::{DirectoryStatus, directory_status};
use anyhow::{Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

/// A read-only projection of the `.leaf/` workspace, grouped into lifecycle stages.
#[derive(Debug)]
pub(crate) struct Inventory {
    pub(crate) leaf_root: PathBuf,
    pub(crate) stages: Vec<StageInventory>,
}

#[derive(Debug)]
pub(crate) struct StageInventory {
    pub(crate) stage_dir: StageDir,
    pub(crate) items: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageDir {
    Sprouts,
    Leaves,
    Fallen,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    Sprout,
    Leaf,
    Fallen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    LeafWork,
    PressedDigest,
}

#[derive(Debug)]
pub(crate) struct InventoryItem {
    pub(crate) stage_dir: StageDir,
    pub(crate) slug: String,
    pub(crate) kind: ItemKind,
    pub(crate) path: PathBuf,
    pub(crate) status: StatusSummary,
    pub(crate) preview: PreviewSource,
    pub(crate) review: Option<crate::review::ReviewSource>,
}

#[derive(Debug)]
pub(crate) struct StatusSummary {
    pub(crate) parse_state: ParseState,
    pub(crate) stage: Option<String>,
    pub(crate) legacy_state: Option<String>,
    pub(crate) fallen_reason: Option<String>,
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
    Stage,
    FallenReason,
    CurrentPhase,
    CurrentGate,
    FirstMissingGate,
    NextAction,
}

impl StatusField {
    /// The canonical snake_case label for this field, shared by list output and
    /// doctor diagnostics so missing-field messages stay in sync.
    pub(crate) fn label(self) -> &'static str {
        match self {
            StatusField::Stage => "stage",
            StatusField::FallenReason => "fallen_reason",
            StatusField::CurrentPhase => "current_phase",
            StatusField::CurrentGate => "current_gate",
            StatusField::FirstMissingGate => "first_missing_gate",
            StatusField::NextAction => "next_action",
        }
    }
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

pub(crate) const STAGE_DIRS: [StageDir; 3] =
    [StageDir::Sprouts, StageDir::Leaves, StageDir::Fallen];

pub(crate) const OLD_NUMBERED_STAGE_DIRS: [StageDir; 4] = [
    StageDir::Sprouts,
    StageDir::Leaves,
    StageDir::Fallen,
    StageDir::Pressed,
];

pub(crate) const STAGES: [Stage; 3] = [Stage::Sprout, Stage::Leaf, Stage::Fallen];

impl Stage {
    pub(crate) fn dir_name(self) -> &'static str {
        match self {
            Stage::Sprout => "01-sprouts",
            Stage::Leaf => "02-leaves",
            Stage::Fallen => "03-fallen",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Stage::Sprout => "sprout",
            Stage::Leaf => "leaf",
            Stage::Fallen => "fallen",
        }
    }
}

impl StageDir {
    /// The canonical on-disk stage directory name.
    pub(crate) fn dir_name(self) -> &'static str {
        match self {
            StageDir::Sprouts => Stage::Sprout.dir_name(),
            StageDir::Leaves => Stage::Leaf.dir_name(),
            StageDir::Fallen => Stage::Fallen.dir_name(),
            StageDir::Pressed => "04-pressed",
        }
    }

    /// The old numbered directory name, used only for diagnostics and one-time migration.
    pub(crate) fn old_numbered_dir_name(self) -> Option<&'static str> {
        match self {
            StageDir::Sprouts => Some("01-seeds"),
            StageDir::Leaves => None,
            StageDir::Fallen => None,
            StageDir::Pressed => Some("04-pressed"),
        }
    }
}

/// Read the `.leaf/` workspace under `repo_root` and project it into an [`Inventory`].
///
/// This never creates directories or files. A missing or non-directory `.leaf/`
/// is an error; missing stage directories under an existing `.leaf/` are treated
/// as empty stages.
pub(crate) fn load(repo_root: &Path) -> Result<Inventory> {
    let leaf_root = repo_root.join(".leaf");
    match directory_status(&leaf_root)? {
        DirectoryStatus::Directory => {}
        DirectoryStatus::NotDirectory => bail!(
            "path exists but is not a directory: {}",
            leaf_root.display()
        ),
        DirectoryStatus::Missing => {
            bail!(".leaf/ is not initialized in this git repository\nhint: run `leaf init`");
        }
    }

    let stages = STAGE_DIRS
        .iter()
        .map(|&stage_dir| load_stage(&leaf_root, stage_dir))
        .collect::<Result<Vec<_>>>()?;

    Ok(Inventory { leaf_root, stages })
}

fn load_stage(leaf_root: &Path, stage_dir: StageDir) -> Result<StageInventory> {
    let stage_dir_path = leaf_root.join(stage_dir.dir_name());
    let entries = match fs::read_dir(&stage_dir_path) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(StageInventory {
                stage_dir,
                items: Vec::new(),
            });
        }
        Err(err) => {
            return Err(err).map_err(|err| {
                anyhow::Error::new(err)
                    .context(format!("failed to read {}", stage_dir_path.display()))
            });
        }
    };

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            anyhow::Error::new(err).context(format!(
                "failed to read entry in {}",
                stage_dir_path.display()
            ))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            anyhow::Error::new(err).context(format!("failed to inspect {}", entry.path().display()))
        })?;

        if let Some(item) = project_entry(stage_dir, file_type, entry.path()) {
            items.push(item);
        }
    }

    items.sort_by(item_display_order);
    Ok(StageInventory { stage_dir, items })
}

fn item_display_order(left: &InventoryItem, right: &InventoryItem) -> std::cmp::Ordering {
    gate_order(left.status.current_gate.as_deref())
        .cmp(&gate_order(right.status.current_gate.as_deref()))
        .then_with(|| left.slug.cmp(&right.slug))
}

fn gate_order(gate: Option<&str>) -> (usize, String) {
    match gate.and_then(parse_gate_index) {
        Some(index) => (index, String::new()),
        None => (
            usize::MAX,
            gate.unwrap_or("").trim().to_lowercase().to_string(),
        ),
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

fn project_entry(
    stage_dir: StageDir,
    file_type: fs::FileType,
    path: PathBuf,
) -> Option<InventoryItem> {
    match stage_dir {
        StageDir::Pressed => {
            if !file_type.is_file() {
                return None;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return None;
            }
            let slug = path.file_stem()?.to_str()?.to_string();
            Some(load_pressed_item(stage_dir, slug, path))
        }
        StageDir::Sprouts | StageDir::Leaves | StageDir::Fallen => {
            if !file_type.is_dir() {
                return None;
            }
            let slug = path.file_name()?.to_str()?.to_string();
            Some(load_directory_item(stage_dir, slug, path))
        }
    }
}

fn load_directory_item(stage_dir: StageDir, slug: String, path: PathBuf) -> InventoryItem {
    let status_path = path.join("00-status.md");
    let status = match fs::read_to_string(&status_path) {
        Ok(content) => parse_status_summary(&content, stage_dir),
        Err(_) => StatusSummary::error(),
    };

    let preview = PreviewSource::LeafWork {
        status_path,
        intent_path: path.join("01-Learn/01-intent.md"),
        unknowns_path: path.join("01-Learn/02-unknowns.md"),
        criteria_path: path.join("02-Example/03-criteria.md"),
    };
    let root_relative_path = format!(".leaf/{}/{}", stage_dir.dir_name(), slug);

    InventoryItem {
        stage_dir,
        slug,
        kind: ItemKind::LeafWork,
        path: path.clone(),
        status,
        preview,
        review: Some(crate::review::ReviewSource::LeafWork {
            root_path: path,
            root_relative_path,
        }),
    }
}

fn load_pressed_item(stage_dir: StageDir, slug: String, path: PathBuf) -> InventoryItem {
    let status = match fs::read_to_string(&path) {
        Ok(_) => parse_status_summary("", stage_dir),
        Err(_) => StatusSummary::error(),
    };

    let preview = PreviewSource::PressedDigest {
        digest_path: path.clone(),
    };

    InventoryItem {
        stage_dir,
        slug,
        kind: ItemKind::PressedDigest,
        path,
        status,
        preview,
        review: None,
    }
}

impl StatusSummary {
    fn error() -> Self {
        StatusSummary {
            parse_state: ParseState::Error,
            stage: None,
            legacy_state: None,
            fallen_reason: None,
            current_phase: None,
            current_gate: None,
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        }
    }
}

/// Expected fields per stage_dir, in display order.
fn expected_fields(stage_dir: StageDir) -> &'static [StatusField] {
    match stage_dir {
        StageDir::Sprouts | StageDir::Leaves => &[
            StatusField::Stage,
            StatusField::CurrentPhase,
            StatusField::CurrentGate,
            StatusField::FirstMissingGate,
            StatusField::NextAction,
        ],
        StageDir::Fallen => &[StatusField::Stage, StatusField::FallenReason],
        StageDir::Pressed => &[],
    }
}

/// Parse the recognized `- key: value` lines out of a status document.
///
/// Keys are matched case-insensitively with internal whitespace collapsed.
/// `Pressed` digests carry no status fields and always parse as [`ParseState::Ok`].
pub(crate) fn parse_status_summary(content: &str, stage_dir: StageDir) -> StatusSummary {
    if matches!(stage_dir, StageDir::Pressed) {
        return StatusSummary {
            parse_state: ParseState::Ok,
            stage: None,
            legacy_state: None,
            fallen_reason: None,
            current_phase: None,
            current_gate: None,
            first_missing_gate: None,
            next_action: None,
            missing_fields: Vec::new(),
        };
    }

    let mut stage = None;
    let mut legacy_state = None;
    let mut fallen_reason = None;
    let mut current_phase = None;
    let mut current_gate = None;
    let mut first_missing_gate = None;
    let mut next_action = None;

    for line in content.lines() {
        // Only the status preamble is canonical. A second-level-or-deeper
        // heading (`##`, `###`, …) ends it, so later sections like
        // `## Previous Status` in a fallen file cannot override the real
        // stage. The single-`#` document title does not match.
        if line.trim_start().starts_with("##") {
            break;
        }
        let Some((key, value)) = parse_field_line(line) else {
            continue;
        };
        match key.as_str() {
            "stage" => stage = Some(value),
            "state" => legacy_state = Some(value),
            "fallen reason" => fallen_reason = Some(value),
            "fall reason" => fallen_reason = Some(value),
            "current phase" => current_phase = Some(value),
            "current gate" => current_gate = Some(value),
            "first missing gate" => first_missing_gate = Some(value),
            "next action" => next_action = Some(value),
            _ => {}
        }
    }

    let projected_stage = stage.clone().or_else(|| {
        legacy_state
            .as_deref()
            .and_then(|value| project_legacy_state(stage_dir, value))
    });

    let value_for = |field: StatusField| match field {
        StatusField::Stage => &projected_stage,
        StatusField::FallenReason => &fallen_reason,
        StatusField::CurrentPhase => &current_phase,
        StatusField::CurrentGate => &current_gate,
        StatusField::FirstMissingGate => &first_missing_gate,
        StatusField::NextAction => &next_action,
    };

    let missing_fields: Vec<StatusField> = expected_fields(stage_dir)
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
        stage: projected_stage,
        legacy_state,
        fallen_reason,
        current_phase,
        current_gate,
        first_missing_gate,
        next_action,
        missing_fields,
    }
}

fn project_legacy_state(stage_dir: StageDir, legacy_state: &str) -> Option<String> {
    let normalized = legacy_state.trim().to_lowercase();
    let stage = match normalized.as_str() {
        "seed" | "active" => Stage::Sprout,
        "leaf" | "complete" | "completed" => Stage::Leaf,
        "fallen" => Stage::Fallen,
        _ => match stage_dir {
            StageDir::Sprouts => Stage::Sprout,
            StageDir::Leaves => Stage::Sprout,
            StageDir::Fallen => Stage::Fallen,
            StageDir::Pressed => return None,
        },
    };
    Some(stage.label().to_string())
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

/// Presence and fill state of the why / what / wireframe triple in a status
/// preamble. `missing` holds triple keys with no `- key:` line at all;
/// `unfilled` holds keys whose value is still a scaffold `TODO` placeholder or
/// empty. A `none — …` value is a valid Learn-close answer (understanding-only)
/// and counts as neither. `leaf doctor` uses this to guarantee the triple the
/// detail header and preview surface is actually present and filled — the status
/// parser proper deliberately ignores these keys, so this is the one place that
/// looks at them for diagnosis.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TripleState {
    pub(crate) missing: Vec<&'static str>,
    pub(crate) unfilled: Vec<&'static str>,
}

pub(crate) fn status_triple_state(content: &str) -> TripleState {
    let mut why = None;
    let mut what = None;
    let mut wireframe = None;
    for line in content.lines() {
        // Only the preamble is canonical; a `##` heading ends it (matches
        // parse_status_summary so `## Previous Status` can't leak triple lines).
        if line.trim_start().starts_with("##") {
            break;
        }
        let Some((key, value)) = parse_field_line(line) else {
            continue;
        };
        match key.as_str() {
            "why" => why = Some(value),
            "what" => what = Some(value),
            "wireframe" => wireframe = Some(value),
            _ => {}
        }
    }

    let mut state = TripleState::default();
    for (label, value) in [("why", &why), ("what", &what), ("wireframe", &wireframe)] {
        match value {
            None => state.missing.push(label),
            Some(v) => {
                let trimmed = v.trim();
                if trimmed.is_empty() || trimmed.starts_with("TODO") {
                    state.unfilled.push(label);
                }
            }
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    fn full_status() -> &'static str {
        "# Leaf Status\n\n\
         - stage: leaf\n\
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
    fn inventory_load_returns_stages_in_order_even_when_empty() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");

        let inventory = load(root.path()).expect("load inventory");

        assert_eq!(inventory.leaf_root, root.path().join(".leaf"));
        assert_eq!(inventory.stages.len(), 3);
        assert_eq!(inventory.stages[0].stage_dir, StageDir::Sprouts);
        assert_eq!(inventory.stages[1].stage_dir, StageDir::Leaves);
        assert_eq!(inventory.stages[2].stage_dir, StageDir::Fallen);
        for stage_dir in &inventory.stages {
            assert!(stage_dir.items.is_empty(), "expected empty stage_dir");
        }
    }

    #[cfg(unix)]
    #[test]
    fn inventory_load_accepts_leaf_root_symlink_to_directory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child("leaf-store")
            .create_dir_all()
            .expect("leaf store");
        std::os::unix::fs::symlink(root.path().join("leaf-store"), root.path().join(".leaf"))
            .expect("leaf symlink");

        let inventory = load(root.path()).expect("load inventory");

        assert_eq!(inventory.leaf_root, root.path().join(".leaf"));
        assert_eq!(inventory.stages.len(), 3);
    }

    #[test]
    fn inventory_load_does_not_create_missing_stage_directories() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf").create_dir_all().expect("leaf root");

        load(root.path()).expect("load inventory");

        assert!(!root.path().join(".leaf/01-sprouts").exists());
        assert!(!root.path().join(".leaf/02-leaves").exists());
        assert!(!root.path().join(".leaf/03-fallen").exists());
        assert!(!root.path().join(".leaf/04-pressed").exists());
    }

    #[test]
    fn inventory_load_lists_only_directories_in_sprouts_sorted_by_gate_then_slug() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-sprouts/third/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - stage: sprout\n\
                 - current phase: Example\n\
                 - current gate: ③ Criteria\n\
                 - first missing gate: ④ Wireframe\n\
                 - next action: write criteria\n",
            )
            .expect("third");
        root.child(".leaf/01-sprouts/second-zebra/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - stage: sprout\n\
                 - current phase: Learn\n\
                 - current gate: ② Unknowns\n\
                 - first missing gate: ③ Criteria\n\
                 - next action: resolve unknowns\n",
            )
            .expect("second zebra");
        root.child(".leaf/01-sprouts/second-apple/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - stage: sprout\n\
                 - current phase: Learn\n\
                 - current gate: G2\n\
                 - first missing gate: G3\n\
                 - next action: resolve unknowns\n",
            )
            .expect("second apple");
        root.child(".leaf/01-sprouts/first/00-status.md")
            .write_str(
                "# Leaf Status\n\n\
                 - stage: sprout\n\
                 - current phase: Learn\n\
                 - current gate: 1 Intent\n\
                 - first missing gate: ② Unknowns\n\
                 - next action: clarify intent\n",
            )
            .expect("first");
        root.child(".leaf/01-sprouts/loose.md")
            .write_str("stray file\n")
            .expect("loose file");

        let inventory = load(root.path()).expect("load inventory");

        let slugs: Vec<_> = inventory.stages[0]
            .items
            .iter()
            .map(|item| item.slug.as_str())
            .collect();
        assert_eq!(
            slugs,
            vec!["first", "second-apple", "second-zebra", "third"]
        );
    }

    #[test]
    fn inventory_load_ignores_top_level_pressed_digests() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/01-sprouts")
            .create_dir_all()
            .expect("sprouts");
        root.child(".leaf/02-leaves")
            .create_dir_all()
            .expect("leaves");
        root.child(".leaf/03-fallen")
            .create_dir_all()
            .expect("fallen");
        root.child(".leaf/04-pressed/note.txt")
            .write_str("not a digest\n")
            .expect("note");
        root.child(".leaf/04-pressed/real.md")
            .write_str("# Real\n")
            .expect("digest");

        let inventory = load(root.path()).expect("load inventory");

        assert_eq!(inventory.stages.len(), 3);
        assert!(
            inventory
                .stages
                .iter()
                .all(|stage_dir| stage_dir.stage_dir != StageDir::Pressed)
        );
        assert!(
            inventory
                .stages
                .iter()
                .all(|stage_dir| stage_dir.items.is_empty())
        );
    }

    #[test]
    fn inventory_leaf_item_has_leafwork_kind_and_preview_paths() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        root.child(".leaf/02-leaves/demo/00-status.md")
            .write_str(full_status())
            .expect("status");

        let inventory = load(root.path()).expect("load inventory");
        let item = &inventory.stages[1].items[0];

        assert_eq!(item.stage_dir, StageDir::Leaves);
        assert_eq!(item.slug, "demo");
        assert_eq!(item.kind, ItemKind::LeafWork);
        assert_eq!(item.path, root.path().join(".leaf/02-leaves/demo"));
        assert_eq!(item.status.parse_state, ParseState::Ok);
        assert_eq!(item.status.stage.as_deref(), Some("leaf"));
        assert!(item.status.legacy_state.is_none());
        assert_eq!(item.status.current_phase.as_deref(), Some("Architect"));
        assert_eq!(item.status.next_action.as_deref(), Some("write design"));

        match &item.preview {
            PreviewSource::LeafWork {
                status_path,
                intent_path,
                unknowns_path,
                criteria_path,
            } => {
                let base = root.path().join(".leaf/02-leaves/demo");
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
        root.child(".leaf/02-leaves/no-status/01-Learn")
            .create_dir_all()
            .expect("dir without status");

        let inventory = load(root.path()).expect("load inventory");
        let item = &inventory.stages[1].items[0];

        assert_eq!(item.slug, "no-status");
        assert_eq!(item.kind, ItemKind::LeafWork);
        assert_eq!(item.status.parse_state, ParseState::Error);
        assert!(item.status.legacy_state.is_none());
        assert!(item.status.current_phase.is_none());
        assert!(item.status.next_action.is_none());
        assert!(item.status.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_ok_when_all_expected_present() {
        let summary = parse_status_summary(full_status(), StageDir::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert!(summary.missing_fields.is_empty());
        assert_eq!(summary.stage.as_deref(), Some("leaf"));
        assert!(summary.legacy_state.is_none());
        assert_eq!(summary.current_phase.as_deref(), Some("Architect"));
        assert_eq!(summary.current_gate.as_deref(), Some("G3"));
        assert_eq!(summary.first_missing_gate.as_deref(), Some("G4"));
        assert_eq!(summary.next_action.as_deref(), Some("write design"));
    }

    #[test]
    fn inventory_parse_status_summary_partial_lists_missing_fields() {
        let content = "- state: seed\n- current phase: Learn\n";

        let summary = parse_status_summary(content, StageDir::Sprouts);

        assert_eq!(summary.parse_state, ParseState::Partial);
        assert_eq!(summary.stage.as_deref(), Some("sprout"));
        assert_eq!(summary.legacy_state.as_deref(), Some("seed"));
        assert_eq!(summary.current_phase.as_deref(), Some("Learn"));
        assert!(summary.current_gate.is_none());
        assert!(!summary.missing_fields.contains(&StatusField::Stage));
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
    fn inventory_parse_status_summary_prefers_stage_and_keeps_state_as_fallback_only() {
        let content = "- stage: sprout\n\
                       - current phase: Learn\n\
                       - current gate: ① Intent\n\
                       - first missing gate: ① Intent\n\
                       - next action: draft intent\n";

        let summary = parse_status_summary(content, StageDir::Sprouts);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.stage.as_deref(), Some("sprout"));
        assert!(summary.legacy_state.is_none());
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_projects_old_state_as_compatibility_stage() {
        let content = "- state: active\n\
                       - current phase: Architect\n\
                       - current gate: ⑤ Design\n\
                       - first missing gate: ⑥ Critic\n\
                       - next action: continue\n";

        let summary = parse_status_summary(content, StageDir::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.stage.as_deref(), Some("sprout"));
        assert_eq!(summary.legacy_state.as_deref(), Some("active"));
    }

    #[test]
    fn inventory_parse_status_summary_normalizes_keys_and_ignores_unknown() {
        let content = "- State:  active\n\
                       - Current   Phase: Learn\n\
                       - CURRENT GATE: G1\n\
                       - First Missing Gate: G2\n\
                       - Next Action: do it: now\n\
                       - random key: whatever\n";

        let summary = parse_status_summary(content, StageDir::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.legacy_state.as_deref(), Some("active"));
        assert_eq!(summary.current_phase.as_deref(), Some("Learn"));
        assert_eq!(summary.current_gate.as_deref(), Some("G1"));
        assert_eq!(summary.first_missing_gate.as_deref(), Some("G2"));
        assert_eq!(summary.next_action.as_deref(), Some("do it: now"));
    }

    #[test]
    fn inventory_status_triple_state_classifies_missing_unfilled_and_valid() {
        // all three real → clean
        let ok =
            status_triple_state("- why: 막는다\n- what: 스킬\n- wireframe: 한 건\n- stage: leaf\n");
        assert_eq!(ok, TripleState::default());

        // none — is a valid Learn-close answer → neither missing nor unfilled
        let none_ok = status_triple_state(
            "- why: 실제 이유\n- what: none — 이해 전용\n- wireframe: none — 이해 전용\n",
        );
        assert_eq!(none_ok, TripleState::default());

        // absent lines → missing; TODO/empty → unfilled
        let mixed =
            status_triple_state("- why: TODO state the problem\n- what:   \n- stage: sprout\n");
        assert_eq!(mixed.missing, vec!["wireframe"]);
        assert_eq!(mixed.unfilled, vec!["why", "what"]);

        // a `## Overview` heading ends the preamble; later `- why:` is ignored
        let preamble_only = status_triple_state("- stage: leaf\n\n## Overview\n- why: 본문 밖\n");
        assert_eq!(preamble_only.missing, vec!["why", "what", "wireframe"]);
    }

    #[test]
    fn inventory_parse_status_summary_fallen_expects_stage_and_reason() {
        let summary = parse_status_summary(
            "- stage: fallen\n- fallen reason: completed\n",
            StageDir::Fallen,
        );

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.stage.as_deref(), Some("fallen"));
        assert_eq!(summary.fallen_reason.as_deref(), Some("completed"));
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_fallen_partial_when_required_fields_missing() {
        let summary = parse_status_summary("- current phase: x\n", StageDir::Fallen);

        assert_eq!(summary.parse_state, ParseState::Partial);
        assert_eq!(
            summary.missing_fields,
            vec![StatusField::Stage, StatusField::FallenReason]
        );
    }

    #[test]
    fn inventory_parse_status_summary_pressed_is_ok_with_no_fields() {
        let summary = parse_status_summary("anything at all\n- state: x\n", StageDir::Pressed);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert!(summary.legacy_state.is_none());
        assert!(summary.current_phase.is_none());
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_ignores_fields_after_section_heading() {
        // A fallen file keeps the canonical stage and fallen reason in its
        // preamble, then embeds the prior active status under a `## Previous
        // Status` section. Only the preamble is canonical; the copied
        // `- state: active`/phase/gate lines below the heading must NOT
        // override it.
        let content = "# Leaf Status\n\n\
                       - stage: fallen\n\
                       - fallen reason: completed\n\
                       \n\
                       ## Previous Status\n\
                       \n\
                       - state: active\n\
                       - current phase: Architect\n\
                       - current gate: G3\n";

        let summary = parse_status_summary(content, StageDir::Fallen);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.stage.as_deref(), Some("fallen"));
        assert_eq!(summary.fallen_reason.as_deref(), Some("completed"));
        assert!(summary.legacy_state.is_none());
        assert!(summary.current_phase.is_none());
        assert!(summary.current_gate.is_none());
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_title_does_not_end_preamble() {
        // The single-`#` document title must NOT be treated as a section break;
        // all five fields below it are still canonical for an active leaf, and a
        // trailing `## Gate progress` section with junk is ignored.
        let content = "# Leaf Status\n\n\
                       - state: active\n\
                       - current phase: Architect\n\
                       - current gate: G3\n\
                       - first missing gate: G4\n\
                       - next action: write design\n\
                       \n\
                       ## Gate progress\n\
                       - state: fallen\n";

        let summary = parse_status_summary(content, StageDir::Leaves);

        assert_eq!(summary.parse_state, ParseState::Ok);
        assert_eq!(summary.legacy_state.as_deref(), Some("active"));
        assert_eq!(summary.current_phase.as_deref(), Some("Architect"));
        assert_eq!(summary.current_gate.as_deref(), Some("G3"));
        assert!(summary.missing_fields.is_empty());
    }

    #[test]
    fn inventory_parse_status_summary_heading_before_fields_yields_no_fields() {
        // Boundary: a `##` heading before any field line ends the preamble
        // immediately, so no canonical field is captured.
        let content = "## Previous Status\n\
                       - state: active\n\
                       - current phase: Architect\n";

        let summary = parse_status_summary(content, StageDir::Leaves);

        assert!(summary.legacy_state.is_none());
        assert!(summary.current_phase.is_none());
        assert_eq!(summary.parse_state, ParseState::Partial);
    }
}
