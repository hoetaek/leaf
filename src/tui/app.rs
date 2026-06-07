use crate::inventory::{Bucket, Inventory, InventoryItem, ItemKind, ParseState, PreviewSource};
use crate::preview::{self, Preview, PreviewLine};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListRow {
    bucket: Bucket,
    bucket_label: String,
    slug: String,
    state: String,
    phase: String,
    gate: String,
    parse_state: ParseState,
    relative_path: String,
    searchable_text: String,
    preview_source: PreviewSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BucketFilter {
    All,
    Bucket(Bucket),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    List,
    FilterInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyInput {
    Up,
    Down,
    Left,
    Right,
    Esc,
    Backspace,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    Continue,
    Quit,
}

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    rows: Vec<ListRow>,
    active_bucket: BucketFilter,
    filter: String,
    selected_index: usize,
    preview_open: bool,
    mode: Mode,
    status_line: String,
    preview_cache: RefCell<HashMap<String, Preview>>,
}

const BUCKET_FILTERS: [BucketFilter; 5] = [
    BucketFilter::All,
    BucketFilter::Bucket(Bucket::Seeds),
    BucketFilter::Bucket(Bucket::Leaves),
    BucketFilter::Bucket(Bucket::Fallen),
    BucketFilter::Bucket(Bucket::Pressed),
];

impl ListRow {
    fn from_item(inventory: &Inventory, item: &InventoryItem) -> Self {
        let bucket_label = bucket_label_singular(item.bucket).to_string();
        let relative_path = relative_leaf_path(inventory, &item.path);
        let state = display_state(item);
        let phase = display_optional(&item.status.current_phase, "-");
        let gate = display_optional(&item.status.current_gate, "-");
        let searchable_text =
            searchable_text(item, &bucket_label, &relative_path, &state, &phase, &gate);

        ListRow {
            bucket: item.bucket,
            bucket_label,
            slug: item.slug.clone(),
            state,
            phase,
            gate,
            parse_state: item.status.parse_state,
            relative_path,
            searchable_text,
            preview_source: item.preview.clone(),
        }
    }

    pub(crate) fn bucket(&self) -> Bucket {
        self.bucket
    }

    pub(crate) fn bucket_label(&self) -> &str {
        &self.bucket_label
    }

    pub(crate) fn slug(&self) -> &str {
        &self.slug
    }

    pub(crate) fn state(&self) -> &str {
        &self.state
    }

    pub(crate) fn phase(&self) -> &str {
        &self.phase
    }

    pub(crate) fn gate(&self) -> &str {
        &self.gate
    }

    pub(crate) fn parse_state(&self) -> ParseState {
        self.parse_state
    }

    pub(crate) fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub(crate) fn searchable_text(&self) -> &str {
        &self.searchable_text
    }

    pub(crate) fn preview_source(&self) -> &PreviewSource {
        &self.preview_source
    }
}

impl AppState {
    pub(crate) fn from_inventory(inventory: &Inventory) -> Self {
        let rows = inventory
            .buckets
            .iter()
            .flat_map(|bucket| {
                bucket
                    .items
                    .iter()
                    .map(|item| ListRow::from_item(inventory, item))
            })
            .collect();

        let mut state = AppState {
            rows,
            active_bucket: BucketFilter::All,
            filter: String::new(),
            selected_index: 0,
            preview_open: true,
            mode: Mode::List,
            status_line: String::new(),
            preview_cache: RefCell::new(HashMap::new()),
        };
        state.refresh_visibility_state();
        state
    }

    pub(crate) fn rows(&self) -> &[ListRow] {
        &self.rows
    }

    pub(crate) fn visible_rows(&self) -> Vec<&ListRow> {
        self.rows
            .iter()
            .filter(|row| self.row_is_visible(row))
            .collect()
    }

    pub(crate) fn selected_row(&self) -> Option<&ListRow> {
        self.visible_rows().get(self.selected_index).copied()
    }

    pub(crate) fn active_bucket(&self) -> BucketFilter {
        self.active_bucket
    }

    pub(crate) fn filter(&self) -> &str {
        &self.filter
    }

    pub(crate) fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub(crate) fn preview_open(&self) -> bool {
        self.preview_open
    }

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn status_line(&self) -> &str {
        &self.status_line
    }

    pub(crate) fn selected_preview(&self) -> Option<Preview> {
        let row = self.selected_row()?;
        if let Some(preview) = self
            .preview_cache
            .borrow()
            .get(row.relative_path())
            .cloned()
        {
            return Some(preview);
        }

        let preview =
            preview::build_from_source(row.slug(), row.preview_source()).unwrap_or_else(|err| {
                Preview {
                    title: row.slug().to_string(),
                    lines: vec![PreviewLine::Plain(format!(
                        "Unable to build preview: {err}"
                    ))],
                }
            });
        self.preview_cache
            .borrow_mut()
            .insert(row.relative_path().to_string(), preview.clone());
        Some(preview)
    }

    pub(crate) fn handle_key(&mut self, input: KeyInput) -> Outcome {
        match self.mode {
            Mode::List => self.handle_list_key(input),
            Mode::FilterInput => self.handle_filter_key(input),
        }
    }

    fn handle_list_key(&mut self, input: KeyInput) -> Outcome {
        match input {
            KeyInput::Up | KeyInput::Char('k') => self.move_selection_up(),
            KeyInput::Down | KeyInput::Char('j') => self.move_selection_down(),
            KeyInput::Left | KeyInput::Char('h') => self.move_bucket_left(),
            KeyInput::Right | KeyInput::Char('l') => self.move_bucket_right(),
            KeyInput::Char('/') => self.mode = Mode::FilterInput,
            KeyInput::Char('p') => self.preview_open = !self.preview_open,
            KeyInput::Esc | KeyInput::Char('q') => return Outcome::Quit,
            KeyInput::Backspace | KeyInput::Char(_) => {}
        }
        Outcome::Continue
    }

    fn handle_filter_key(&mut self, input: KeyInput) -> Outcome {
        match input {
            KeyInput::Esc => self.mode = Mode::List,
            KeyInput::Backspace => {
                self.filter.pop();
                self.refresh_visibility_state();
            }
            KeyInput::Char(ch) => {
                self.filter.push(ch);
                self.refresh_visibility_state();
            }
            KeyInput::Up | KeyInput::Down | KeyInput::Left | KeyInput::Right => {}
        }
        Outcome::Continue
    }

    fn move_selection_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
        self.clamp_selected_index();
    }

    fn move_selection_down(&mut self) {
        let visible_count = self.visible_count();
        if visible_count == 0 {
            self.selected_index = 0;
            return;
        }
        self.selected_index = (self.selected_index + 1).min(visible_count - 1);
    }

    fn move_bucket_left(&mut self) {
        self.shift_bucket_filter(-1);
    }

    fn move_bucket_right(&mut self) {
        self.shift_bucket_filter(1);
    }

    fn shift_bucket_filter(&mut self, delta: isize) {
        let current_index = BUCKET_FILTERS
            .iter()
            .position(|filter| *filter == self.active_bucket)
            .unwrap_or(0);
        let next_index = current_index
            .saturating_add_signed(delta)
            .min(BUCKET_FILTERS.len() - 1);
        self.active_bucket = BUCKET_FILTERS[next_index];
        self.refresh_visibility_state();
    }

    fn refresh_visibility_state(&mut self) {
        self.clamp_selected_index();
        self.update_status_line();
    }

    fn clamp_selected_index(&mut self) {
        let visible_count = self.visible_count();
        if visible_count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= visible_count {
            self.selected_index = visible_count - 1;
        }
    }

    fn visible_count(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| self.row_is_visible(row))
            .count()
    }

    fn row_is_visible(&self, row: &ListRow) -> bool {
        self.bucket_matches(row) && self.filter_matches(row)
    }

    fn bucket_matches(&self, row: &ListRow) -> bool {
        match self.active_bucket {
            BucketFilter::All => true,
            BucketFilter::Bucket(bucket) => row.bucket == bucket,
        }
    }

    fn filter_matches(&self, row: &ListRow) -> bool {
        self.filter.is_empty()
            || row
                .searchable_text()
                .to_lowercase()
                .contains(&self.filter.to_lowercase())
    }

    fn update_status_line(&mut self) {
        let visible_count = self.visible_count();
        let total_count = self.rows.len();
        self.status_line = if self.active_bucket == BucketFilter::All && self.filter.is_empty() {
            format!("{total_count} {}", row_word(total_count))
        } else {
            format!("{visible_count} of {total_count} {}", row_word(total_count))
        };
    }
}

fn searchable_text(
    item: &InventoryItem,
    bucket_label: &str,
    relative_path: &str,
    state: &str,
    phase: &str,
    gate: &str,
) -> String {
    let mut parts = vec![
        bucket_label.to_string(),
        bucket_label_plural(item.bucket).to_string(),
        item.slug.clone(),
        relative_path.to_string(),
        state.to_string(),
        phase.to_string(),
        gate.to_string(),
    ];

    if let Some(next_action) = &item.status.next_action {
        parts.push(next_action.clone());
    }

    parts.join(" ")
}

fn relative_leaf_path(inventory: &Inventory, path: &Path) -> String {
    if let Some(repo_root) = inventory.leaf_root.parent() {
        if let Ok(relative_path) = path.strip_prefix(repo_root) {
            return normalize_path(relative_path);
        }
    }

    if let Ok(relative_path) = path.strip_prefix(&inventory.leaf_root) {
        let normalized = normalize_path(relative_path);
        if normalized.is_empty() {
            ".leaf".to_string()
        } else {
            format!(".leaf/{normalized}")
        }
    } else {
        normalize_path(path)
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_state(item: &InventoryItem) -> String {
    match (&item.status.state, item.kind) {
        (Some(state), _) => state.clone(),
        (None, ItemKind::PressedDigest) => "-".to_string(),
        (None, ItemKind::LeafWork) => "?".to_string(),
    }
}

fn display_optional(value: &Option<String>, fallback: &str) -> String {
    value.as_deref().unwrap_or(fallback).to_string()
}

fn bucket_label_singular(bucket: Bucket) -> &'static str {
    match bucket {
        Bucket::Seeds => "seed",
        Bucket::Leaves => "leaf",
        Bucket::Fallen => "fallen",
        Bucket::Pressed => "pressed",
    }
}

fn bucket_label_plural(bucket: Bucket) -> &'static str {
    match bucket {
        Bucket::Seeds => "seeds",
        Bucket::Leaves => "leaves",
        Bucket::Fallen => "fallen",
        Bucket::Pressed => "pressed",
    }
}

fn row_word(count: usize) -> &'static str {
    if count == 1 { "row" } else { "rows" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        Bucket, BucketInventory, Inventory, InventoryItem, ItemKind, ParseState, PreviewSource,
        StatusSummary,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn tui_app_rows_from_inventory_include_buckets_paths_and_search_text() {
        let inventory = inventory_with_items(vec![
            leaf_item(
                Bucket::Seeds,
                "alpha-seed",
                status(
                    ParseState::Ok,
                    Some("ready"),
                    Some("learn"),
                    Some("intent"),
                    Some("write examples"),
                ),
            ),
            leaf_item(
                Bucket::Leaves,
                "beta-leaf",
                status(
                    ParseState::Partial,
                    Some("active"),
                    Some("example"),
                    Some("criteria"),
                    Some("fill gate"),
                ),
            ),
            leaf_item(
                Bucket::Fallen,
                "gamma-fallen",
                status(ParseState::Error, None, None, None, None),
            ),
            pressed_item(
                "delta-pressed",
                status(ParseState::Ok, None, None, None, None),
            ),
        ]);

        let app = AppState::from_inventory(&inventory);
        let rows = app.rows();

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].bucket(), Bucket::Seeds);
        assert_eq!(rows[0].bucket_label(), "seed");
        assert_eq!(rows[0].slug(), "alpha-seed");
        assert_eq!(rows[0].state(), "ready");
        assert_eq!(rows[0].phase(), "learn");
        assert_eq!(rows[0].gate(), "intent");
        assert_eq!(rows[0].parse_state(), ParseState::Ok);
        assert_eq!(rows[0].relative_path(), ".leaf/01-seeds/alpha-seed");
        assert_searchable(
            rows[0].searchable_text(),
            &[
                "seed",
                "alpha-seed",
                ".leaf/01-seeds/alpha-seed",
                "ready",
                "learn",
                "intent",
                "write examples",
            ],
        );

        assert_eq!(rows[1].bucket(), Bucket::Leaves);
        assert_eq!(rows[1].bucket_label(), "leaf");
        assert_eq!(rows[1].relative_path(), ".leaf/02-leaves/beta-leaf");
        assert_eq!(rows[1].parse_state(), ParseState::Partial);

        assert_eq!(rows[2].bucket(), Bucket::Fallen);
        assert_eq!(rows[2].bucket_label(), "fallen");
        assert_eq!(rows[2].state(), "?");
        assert_eq!(rows[2].phase(), "-");
        assert_eq!(rows[2].gate(), "-");
        assert_eq!(rows[2].relative_path(), ".leaf/03-fallen/gamma-fallen");

        assert_eq!(rows[3].bucket(), Bucket::Pressed);
        assert_eq!(rows[3].bucket_label(), "pressed");
        assert_eq!(rows[3].state(), "-");
        assert_eq!(rows[3].phase(), "-");
        assert_eq!(rows[3].gate(), "-");
        assert_eq!(rows[3].relative_path(), ".leaf/04-pressed/delta-pressed.md");
    }

    #[test]
    fn tui_app_movement_clamps_at_boundaries() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.selected_index(), 0);
        assert_eq!(app.handle_key(KeyInput::Up), Outcome::Continue);
        assert_eq!(app.handle_key(KeyInput::Char('k')), Outcome::Continue);
        assert_eq!(app.selected_index(), 0);

        assert_eq!(app.handle_key(KeyInput::Down), Outcome::Continue);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("beta"));
        assert_eq!(app.handle_key(KeyInput::Char('j')), Outcome::Continue);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("gamma"));

        assert_eq!(app.handle_key(KeyInput::Down), Outcome::Continue);
        assert_eq!(app.handle_key(KeyInput::Char('j')), Outcome::Continue);
        assert_eq!(app.selected_index(), 2);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("gamma"));

        assert_eq!(app.handle_key(KeyInput::Up), Outcome::Continue);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("beta"));
    }

    #[test]
    fn tui_app_bucket_navigation_filters_rows_and_clamps_selection() {
        let inventory = inventory_with_items(vec![
            leaf_item(Bucket::Seeds, "seed-a", complete_leaf_status()),
            leaf_item(Bucket::Leaves, "leaf-b", complete_leaf_status()),
            leaf_item(Bucket::Fallen, "fallen-c", complete_leaf_status()),
            pressed_item("pressed-d", status(ParseState::Ok, None, None, None, None)),
        ]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Down);
        assert_eq!(app.selected_index(), 3);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("pressed-d"));

        assert_eq!(app.handle_key(KeyInput::Left), Outcome::Continue);
        assert_eq!(app.active_bucket(), BucketFilter::All);
        assert_eq!(app.selected_index(), 3);

        assert_eq!(app.handle_key(KeyInput::Right), Outcome::Continue);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Seeds));
        assert_eq!(app.selected_index(), 0);
        assert_eq!(visible_slugs(&app), vec!["seed-a"]);

        assert_eq!(app.handle_key(KeyInput::Char('l')), Outcome::Continue);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Leaves));
        assert_eq!(visible_slugs(&app), vec!["leaf-b"]);

        app.handle_key(KeyInput::Char('l'));
        app.handle_key(KeyInput::Char('l'));
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Pressed));
        assert_eq!(visible_slugs(&app), vec!["pressed-d"]);

        assert_eq!(app.handle_key(KeyInput::Right), Outcome::Continue);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Pressed));

        assert_eq!(app.handle_key(KeyInput::Char('h')), Outcome::Continue);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Fallen));
        assert_eq!(visible_slugs(&app), vec!["fallen-c"]);
    }

    #[test]
    fn tui_app_filter_input_narrows_rows_backspace_works_and_esc_exits_filter_mode() {
        let inventory = inventory_with_slugs(&["alpha", "alpine", "beta"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Down);
        assert_eq!(app.selected_index(), 2);

        assert_eq!(app.handle_key(KeyInput::Char('/')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::FilterInput);
        for ch in "alpha".chars() {
            assert_eq!(app.handle_key(KeyInput::Char(ch)), Outcome::Continue);
        }

        assert_eq!(app.filter(), "alpha");
        assert_eq!(visible_slugs(&app), vec!["alpha"]);
        assert_eq!(app.selected_index(), 0);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("alpha"));

        assert_eq!(app.handle_key(KeyInput::Backspace), Outcome::Continue);
        assert_eq!(app.handle_key(KeyInput::Backspace), Outcome::Continue);
        assert_eq!(app.filter(), "alp");
        assert_eq!(visible_slugs(&app), vec!["alpha", "alpine"]);

        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Continue);
        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.filter(), "alp");
    }

    #[test]
    fn tui_app_preview_toggle_changes_preview_open() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        assert!(app.preview_open());
        assert_eq!(app.handle_key(KeyInput::Char('p')), Outcome::Continue);
        assert!(!app.preview_open());
        assert_eq!(app.handle_key(KeyInput::Char('p')), Outcome::Continue);
        assert!(app.preview_open());
    }

    #[test]
    fn tui_app_builds_selected_preview_lazily() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let first = leaf_item_at(root.path(), Bucket::Leaves, "first", complete_leaf_status());
        let second = leaf_item_at(
            root.path(),
            Bucket::Leaves,
            "second",
            complete_leaf_status(),
        );
        write_preview_status(root.path(), "first", "첫 번째 미리보기");
        let inventory = inventory_with_root(root.path(), vec![first, second]);
        let mut app = AppState::from_inventory(&inventory);

        write_preview_status(root.path(), "second", "늦게 생긴 미리보기");
        app.handle_key(KeyInput::Down);
        let preview = app
            .selected_preview()
            .expect("selected preview should be available");
        let preview_text = preview_text(&preview);

        assert!(
            preview_text.contains("늦게 생긴 미리보기"),
            "preview should be read when selected, got {preview_text:?}"
        );
    }

    #[test]
    fn tui_app_q_and_list_mode_esc_quit() {
        let inventory = inventory_with_slugs(&["alpha"]);

        let mut q_app = AppState::from_inventory(&inventory);
        assert_eq!(q_app.handle_key(KeyInput::Char('q')), Outcome::Quit);

        let mut esc_app = AppState::from_inventory(&inventory);
        assert_eq!(esc_app.handle_key(KeyInput::Esc), Outcome::Quit);

        let mut filter_app = AppState::from_inventory(&inventory);
        assert_eq!(
            filter_app.handle_key(KeyInput::Char('/')),
            Outcome::Continue
        );
        assert_eq!(filter_app.handle_key(KeyInput::Esc), Outcome::Continue);
        assert_eq!(filter_app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_empty_inventory_and_empty_filter_result_do_not_panic() {
        let empty_inventory = inventory_with_items(Vec::new());
        let mut empty_app = AppState::from_inventory(&empty_inventory);

        assert!(empty_app.visible_rows().is_empty());
        assert_eq!(empty_app.selected_row().map(ListRow::slug), None);
        assert_eq!(empty_app.handle_key(KeyInput::Down), Outcome::Continue);
        assert_eq!(empty_app.handle_key(KeyInput::Up), Outcome::Continue);
        assert_eq!(empty_app.selected_row().map(ListRow::slug), None);

        let inventory = inventory_with_slugs(&["alpha"]);
        let mut filtered_app = AppState::from_inventory(&inventory);
        filtered_app.handle_key(KeyInput::Char('/'));
        for ch in "zzz".chars() {
            filtered_app.handle_key(KeyInput::Char(ch));
        }

        assert!(filtered_app.visible_rows().is_empty());
        assert_eq!(filtered_app.selected_row().map(ListRow::slug), None);
        assert_eq!(filtered_app.handle_key(KeyInput::Down), Outcome::Continue);
        assert_eq!(
            filtered_app.handle_key(KeyInput::Backspace),
            Outcome::Continue
        );
    }

    fn inventory_with_slugs(slugs: &[&str]) -> Inventory {
        inventory_with_items(
            slugs
                .iter()
                .map(|slug| leaf_item(Bucket::Leaves, slug, complete_leaf_status()))
                .collect(),
        )
    }

    fn inventory_with_items(items: Vec<InventoryItem>) -> Inventory {
        inventory_with_root(&repo_root(), items)
    }

    fn inventory_with_root(root: &Path, items: Vec<InventoryItem>) -> Inventory {
        let mut seeds = Vec::new();
        let mut leaves = Vec::new();
        let mut fallen = Vec::new();
        let mut pressed = Vec::new();

        for item in items {
            match item.bucket {
                Bucket::Seeds => seeds.push(item),
                Bucket::Leaves => leaves.push(item),
                Bucket::Fallen => fallen.push(item),
                Bucket::Pressed => pressed.push(item),
            }
        }

        Inventory {
            leaf_root: root.join(".leaf"),
            buckets: vec![
                BucketInventory {
                    bucket: Bucket::Seeds,
                    items: seeds,
                },
                BucketInventory {
                    bucket: Bucket::Leaves,
                    items: leaves,
                },
                BucketInventory {
                    bucket: Bucket::Fallen,
                    items: fallen,
                },
                BucketInventory {
                    bucket: Bucket::Pressed,
                    items: pressed,
                },
            ],
        }
    }

    fn leaf_item(bucket: Bucket, slug: &str, status: StatusSummary) -> InventoryItem {
        leaf_item_at(&repo_root(), bucket, slug, status)
    }

    fn leaf_item_at(
        root: &Path,
        bucket: Bucket,
        slug: &str,
        status: StatusSummary,
    ) -> InventoryItem {
        let path = root.join(".leaf").join(bucket_dir(bucket)).join(slug);
        InventoryItem {
            bucket,
            slug: slug.to_string(),
            kind: ItemKind::LeafWork,
            path: path.clone(),
            status,
            preview: PreviewSource::LeafWork {
                status_path: path.join("00-status.md"),
                intent_path: path.join("01-Learn/01-intent.md"),
                unknowns_path: path.join("01-Learn/02-unknowns.md"),
                criteria_path: path.join("02-Example/03-criteria.md"),
            },
        }
    }

    fn write_preview_status(root: &Path, slug: &str, body: &str) {
        let status_path = root.join(".leaf/02-leaves").join(slug).join("00-status.md");
        std::fs::create_dir_all(status_path.parent().unwrap()).expect("preview dir");
        std::fs::write(status_path, format!("# Status\n\n{body}\n")).expect("preview status");
    }

    fn preview_text(preview: &Preview) -> String {
        preview
            .lines
            .iter()
            .map(|line| match line {
                PreviewLine::Heading(text) | PreviewLine::Code(text) | PreviewLine::Plain(text) => {
                    text.clone()
                }
                PreviewLine::Checkbox { text, .. } => text.clone(),
                PreviewLine::Styled(spans) => spans
                    .iter()
                    .map(|span| match span {
                        crate::preview::PreviewSpan::Plain(text)
                        | crate::preview::PreviewSpan::Bold(text)
                        | crate::preview::PreviewSpan::Code(text) => text.as_str(),
                    })
                    .collect(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn pressed_item(slug: &str, status: StatusSummary) -> InventoryItem {
        let path = repo_root()
            .join(".leaf")
            .join(Bucket::Pressed.dir_name())
            .join(format!("{slug}.md"));
        InventoryItem {
            bucket: Bucket::Pressed,
            slug: slug.to_string(),
            kind: ItemKind::PressedDigest,
            path: path.clone(),
            status,
            preview: PreviewSource::PressedDigest { digest_path: path },
        }
    }

    fn complete_leaf_status() -> StatusSummary {
        status(
            ParseState::Ok,
            Some("active"),
            Some("learn"),
            Some("intent"),
            Some("write next"),
        )
    }

    fn status(
        parse_state: ParseState,
        state: Option<&str>,
        current_phase: Option<&str>,
        current_gate: Option<&str>,
        next_action: Option<&str>,
    ) -> StatusSummary {
        StatusSummary {
            parse_state,
            state: state.map(str::to_string),
            current_phase: current_phase.map(str::to_string),
            current_gate: current_gate.map(str::to_string),
            first_missing_gate: None,
            next_action: next_action.map(str::to_string),
            missing_fields: Vec::new(),
        }
    }

    fn repo_root() -> PathBuf {
        Path::new("/tmp/leaf-repo").to_path_buf()
    }

    fn bucket_dir(bucket: Bucket) -> &'static str {
        bucket.dir_name()
    }

    fn visible_slugs(app: &AppState) -> Vec<&str> {
        app.visible_rows().iter().map(|row| row.slug()).collect()
    }

    fn assert_searchable(searchable_text: &str, expected_parts: &[&str]) {
        for part in expected_parts {
            assert!(
                searchable_text.contains(part),
                "searchable text {searchable_text:?} should contain {part:?}"
            );
        }
    }
}
