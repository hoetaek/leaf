use crate::inventory::{Bucket, Inventory, InventoryItem, ParseState, PreviewSource};
use crate::list_columns::{
    LIST_COLUMNS, ListColumnRow, bucket_label_plural, bucket_label_singular, markdown_table,
};
use crate::preview::{self, Preview, PreviewLine};
use crate::review::{self, ReviewDocument, ReviewSource};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListRow {
    bucket: Bucket,
    bucket_label: String,
    slug: String,
    phase: String,
    gate: String,
    parse_state: ParseState,
    relative_path: String,
    searchable_text: String,
    preview_source: PreviewSource,
    review_source: Option<ReviewSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BucketFilter {
    All,
    Bucket(Bucket),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    List,
    RangeSelect,
    FilterInput,
    ConfirmPromote,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyInput {
    Up,
    Down,
    Left,
    Right,
    Enter,
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,
    Esc,
    Backspace,
    Char(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseInput {
    Down { visible_index: usize, toggle: bool },
    Drag { visible_index: usize },
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    Continue,
    Quit,
    PromoteSeed { slug: String },
    Refresh,
    CopyRow { slug: String, text: String },
    CopyRows { count: usize, text: String },
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
    pending_promote_slug: Option<String>,
    selected_keys: HashSet<String>,
    range_anchor: Option<usize>,
    mouse_anchor: Option<usize>,
    preview_cache: RefCell<HashMap<String, Preview>>,
    review_body_height: Cell<usize>,
    review_state: Option<ReviewState>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReviewState {
    pub(crate) source: ReviewSource,
    pub(crate) document: ReviewDocument,
    pub(crate) scroll_offset: usize,
    pub(crate) status_message: String,
}

const BUCKET_FILTERS: [BucketFilter; 5] = [
    BucketFilter::All,
    BucketFilter::Bucket(Bucket::Seeds),
    BucketFilter::Bucket(Bucket::Leaves),
    BucketFilter::Bucket(Bucket::Fallen),
    BucketFilter::Bucket(Bucket::Pressed),
];
const DEFAULT_REVIEW_BODY_HEIGHT: usize = 10;

impl ListRow {
    fn from_item(inventory: &Inventory, item: &InventoryItem) -> Self {
        let bucket_label = bucket_label_singular(item.bucket).to_string();
        let relative_path = relative_leaf_path(inventory, &item.path);
        let phase = display_optional(&item.status.current_phase, "-");
        let gate = display_optional(&item.status.current_gate, "-");
        let searchable_text = searchable_text(item, &bucket_label, &relative_path, &phase, &gate);

        ListRow {
            bucket: item.bucket,
            bucket_label,
            slug: item.slug.clone(),
            phase,
            gate,
            parse_state: item.status.parse_state,
            relative_path,
            searchable_text,
            preview_source: item.preview.clone(),
            review_source: item.review.clone(),
        }
    }

    pub(crate) fn bucket(&self) -> Bucket {
        self.bucket
    }

    pub(crate) fn slug(&self) -> &str {
        &self.slug
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

    pub(crate) fn review_source(&self) -> Option<&ReviewSource> {
        self.review_source.as_ref()
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
            pending_promote_slug: None,
            selected_keys: HashSet::new(),
            range_anchor: None,
            mouse_anchor: None,
            preview_cache: RefCell::new(HashMap::new()),
            review_body_height: Cell::new(DEFAULT_REVIEW_BODY_HEIGHT),
            review_state: None,
        };
        state.refresh_visibility_state();
        state
    }

    pub(crate) fn rows(&self) -> &[ListRow] {
        &self.rows
    }

    pub(crate) fn row_count(&self) -> usize {
        self.rows().len()
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

    pub(crate) fn review_state(&self) -> Option<&ReviewState> {
        self.review_state.as_ref()
    }

    pub(crate) fn set_review_body_height(&self, height: usize) {
        self.review_body_height.set(height);
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
            Mode::RangeSelect => self.handle_range_key(input),
            Mode::FilterInput => self.handle_filter_key(input),
            Mode::ConfirmPromote => self.handle_confirm_promote_key(input),
            Mode::Review => self.handle_review_key(input),
        }
    }

    pub(crate) fn handle_mouse(&mut self, input: MouseInput) -> Outcome {
        if matches!(
            self.mode,
            Mode::FilterInput | Mode::ConfirmPromote | Mode::Review
        ) {
            return Outcome::Continue;
        }
        match input {
            MouseInput::Down {
                visible_index,
                toggle,
            } => {
                if !self.select_visible_index(visible_index) {
                    return Outcome::Continue;
                }
                if toggle {
                    self.mouse_anchor = None;
                    self.toggle_current_row_selection();
                } else {
                    self.mouse_anchor = Some(self.selected_index);
                }
            }
            MouseInput::Drag { visible_index } => {
                let Some(anchor) = self.mouse_anchor else {
                    return Outcome::Continue;
                };
                if !self.select_visible_index(visible_index) {
                    return Outcome::Continue;
                }
                self.mark_visible_range(anchor, self.selected_index);
            }
            MouseInput::Up => {
                self.mouse_anchor = None;
            }
        }
        Outcome::Continue
    }

    fn select_visible_index(&mut self, visible_index: usize) -> bool {
        if visible_index < self.visible_count() {
            self.selected_index = visible_index;
            true
        } else {
            false
        }
    }

    fn mark_visible_range(&mut self, anchor: usize, current: usize) {
        let visible_keys: Vec<String> = self
            .visible_rows()
            .iter()
            .map(|row| row.relative_path().to_string())
            .collect();
        let lo = anchor.min(current);
        let hi = anchor.max(current);
        self.selected_keys.clear();
        for index in lo..=hi {
            if let Some(key) = visible_keys.get(index) {
                self.selected_keys.insert(key.clone());
            }
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
            KeyInput::Char(' ') => self.toggle_current_row_selection(),
            KeyInput::Char('v') => self.begin_range_select(),
            KeyInput::Char('a') => self.toggle_all_visible_selection(),
            KeyInput::Char('y') => return self.copy_marked_or_current_row(),
            KeyInput::Char('P') => self.begin_promote(),
            KeyInput::Char('r') => return Outcome::Refresh,
            KeyInput::Enter => self.open_review(),
            KeyInput::Char('q') => return Outcome::Quit,
            KeyInput::Esc => return self.clear_selection_or_quit(),
            KeyInput::Backspace
            | KeyInput::PageUp
            | KeyInput::PageDown
            | KeyInput::HalfPageUp
            | KeyInput::HalfPageDown
            | KeyInput::Char(_) => {}
        }
        Outcome::Continue
    }

    fn handle_range_key(&mut self, input: KeyInput) -> Outcome {
        match input {
            KeyInput::Up | KeyInput::Char('k') => self.move_selection_up(),
            KeyInput::Down | KeyInput::Char('j') => self.move_selection_down(),
            KeyInput::Char('a') => {
                self.toggle_all_visible_selection();
                self.range_anchor = None;
                self.mode = Mode::List;
            }
            KeyInput::Char('y') => {
                self.commit_range_select();
                return self.copy_marked_or_current_row();
            }
            KeyInput::Char('v') | KeyInput::Esc => self.commit_range_select(),
            KeyInput::Char('q') => return Outcome::Quit,
            _ => {}
        }
        Outcome::Continue
    }

    fn handle_review_key(&mut self, input: KeyInput) -> Outcome {
        match input {
            KeyInput::Down | KeyInput::Char('j') => self.scroll_review_down(1),
            KeyInput::Up | KeyInput::Char('k') => self.scroll_review_up(1),
            KeyInput::PageDown => self.scroll_review_down(self.review_page_step()),
            KeyInput::PageUp => self.scroll_review_up(self.review_page_step()),
            KeyInput::HalfPageDown | KeyInput::Char('d') => {
                self.scroll_review_down(self.review_half_page_step())
            }
            KeyInput::HalfPageUp | KeyInput::Char('u') => {
                self.scroll_review_up(self.review_half_page_step())
            }
            KeyInput::Char('g') => self.scroll_review_top(),
            KeyInput::Char('G') => self.scroll_review_bottom(),
            KeyInput::Char('r') => self.refresh_review(),
            KeyInput::Esc | KeyInput::Char('q') => {
                self.review_state = None;
                self.mode = Mode::List;
            }
            _ => {}
        }
        Outcome::Continue
    }

    fn open_review(&mut self) {
        let Some(source) = self
            .selected_row()
            .and_then(|row| row.review_source())
            .cloned()
        else {
            self.status_line = "review is only available for leaf work rows".to_string();
            return;
        };

        match review::build(&source) {
            Ok(document) => {
                self.review_state = Some(ReviewState {
                    source,
                    document,
                    scroll_offset: 0,
                    status_message: String::new(),
                });
                self.mode = Mode::Review;
            }
            Err(err) => {
                self.status_line = format!("review failed: {err}");
            }
        }
    }

    fn refresh_review(&mut self) {
        let Some(state) = &self.review_state else {
            return;
        };
        let source = state.source.clone();
        let body_height = self.review_body_height.get();
        match review::build(&source) {
            Ok(document) => {
                let scroll_offset = state
                    .scroll_offset
                    .min(max_review_scroll(&document, body_height));
                self.review_state = Some(ReviewState {
                    source,
                    document,
                    scroll_offset,
                    status_message: "refreshed from source".to_string(),
                });
            }
            Err(err) => {
                if let Some(state) = &mut self.review_state {
                    state.status_message = format!("refresh failed: {err}");
                }
            }
        }
    }

    pub(crate) fn refresh_review_if_changed(&mut self) -> bool {
        let Some(state) = &self.review_state else {
            return false;
        };
        let source = state.source.clone();
        let current_document = state.document.clone();
        let current_scroll_offset = state.scroll_offset;
        let body_height = self.review_body_height.get();
        match review::build(&source) {
            Ok(document) if document != current_document => {
                let scroll_offset =
                    current_scroll_offset.min(max_review_scroll(&document, body_height));
                self.review_state = Some(ReviewState {
                    source,
                    document,
                    scroll_offset,
                    status_message: "updated from source".to_string(),
                });
                true
            }
            Ok(_) => false,
            Err(err) => {
                if let Some(state) = &mut self.review_state {
                    state.status_message = format!("auto refresh failed: {err}");
                }
                false
            }
        }
    }

    fn review_page_step(&self) -> usize {
        self.review_body_height.get().max(1)
    }

    fn review_half_page_step(&self) -> usize {
        (self.review_page_step() / 2).max(1)
    }

    fn scroll_review_down(&mut self, amount: usize) {
        let body_height = self.review_body_height.get();
        if let Some(state) = &mut self.review_state {
            let max_scroll = max_review_scroll(&state.document, body_height);
            let current_scroll = state.scroll_offset.min(max_scroll);
            state.scroll_offset = current_scroll.saturating_add(amount).min(max_scroll);
        }
    }

    fn scroll_review_up(&mut self, amount: usize) {
        let body_height = self.review_body_height.get();
        if let Some(state) = &mut self.review_state {
            let max_scroll = max_review_scroll(&state.document, body_height);
            state.scroll_offset = state.scroll_offset.min(max_scroll).saturating_sub(amount);
        }
    }

    fn scroll_review_top(&mut self) {
        if let Some(state) = &mut self.review_state {
            state.scroll_offset = 0;
        }
    }

    fn scroll_review_bottom(&mut self) {
        let body_height = self.review_body_height.get();
        if let Some(state) = &mut self.review_state {
            state.scroll_offset = max_review_scroll(&state.document, body_height);
        }
    }

    fn copy_marked_or_current_row(&mut self) -> Outcome {
        let marked = self.marked_copy_rows();
        match marked.len() {
            0 => match self.selected_row() {
                Some(row) => Outcome::CopyRow {
                    slug: row.slug().to_string(),
                    text: markdown_copy_table(&[row]),
                },
                None => {
                    self.status_line = "no row selected to copy".to_string();
                    Outcome::Continue
                }
            },
            count => Outcome::CopyRows {
                count,
                text: markdown_copy_table(&marked),
            },
        }
    }

    fn marked_copy_rows(&self) -> Vec<&ListRow> {
        self.visible_rows()
            .into_iter()
            .enumerate()
            .filter(|(index, _)| self.visible_row_is_marked(*index))
            .map(|(_, row)| row)
            .collect()
    }

    fn toggle_current_row_selection(&mut self) {
        let Some(key) = self
            .selected_row()
            .map(|row| row.relative_path().to_string())
        else {
            return;
        };
        if !self.selected_keys.remove(&key) {
            self.selected_keys.insert(key);
        }
    }

    fn toggle_all_visible_selection(&mut self) {
        let visible_keys: Vec<String> = self
            .visible_rows()
            .iter()
            .map(|row| row.relative_path().to_string())
            .collect();
        if visible_keys.is_empty() {
            return;
        }
        let all_marked = visible_keys
            .iter()
            .all(|key| self.selected_keys.contains(key));
        if all_marked {
            for key in &visible_keys {
                self.selected_keys.remove(key);
            }
        } else {
            for key in visible_keys {
                self.selected_keys.insert(key);
            }
        }
    }

    fn begin_range_select(&mut self) {
        let Some(key) = self
            .selected_row()
            .map(|row| row.relative_path().to_string())
        else {
            return;
        };
        self.selected_keys.clear();
        self.selected_keys.insert(key);
        self.range_anchor = Some(self.selected_index);
        self.mode = Mode::RangeSelect;
    }

    fn commit_range_select(&mut self) {
        let visible_keys: Vec<String> = self
            .visible_rows()
            .iter()
            .map(|row| row.relative_path().to_string())
            .collect();
        if let Some(anchor) = self.range_anchor {
            let lo = anchor.min(self.selected_index);
            let hi = anchor.max(self.selected_index);
            self.selected_keys.clear();
            for index in lo..=hi {
                if let Some(key) = visible_keys.get(index) {
                    self.selected_keys.insert(key.clone());
                }
            }
        }
        self.range_anchor = None;
        self.mode = Mode::List;
    }

    fn clear_selection_or_quit(&mut self) -> Outcome {
        if self.selected_keys.is_empty() {
            Outcome::Quit
        } else {
            self.selected_keys.clear();
            self.range_anchor = None;
            self.status_line = "selection cleared".to_string();
            Outcome::Continue
        }
    }

    pub(crate) fn selected_row_count(&self) -> usize {
        let visible_len = self.visible_rows().len();
        (0..visible_len)
            .filter(|index| self.visible_row_is_marked(*index))
            .count()
    }

    pub(crate) fn visible_row_is_marked(&self, visible_index: usize) -> bool {
        let visible = self.visible_rows();
        let Some(row) = visible.get(visible_index) else {
            return false;
        };
        if self.selected_keys.contains(row.relative_path()) {
            return true;
        }
        if self.mode == Mode::RangeSelect {
            if let Some(anchor) = self.range_anchor {
                let lo = anchor.min(self.selected_index);
                let hi = anchor.max(self.selected_index);
                if visible_index >= lo && visible_index <= hi {
                    return true;
                }
            }
        }
        false
    }

    fn prune_hidden_selection(&mut self) {
        if self.selected_keys.is_empty() {
            return;
        }
        let visible_keys: HashSet<String> = self
            .visible_rows()
            .iter()
            .map(|row| row.relative_path().to_string())
            .collect();
        self.selected_keys.retain(|key| visible_keys.contains(key));
    }

    fn begin_promote(&mut self) {
        let target = self
            .selected_row()
            .filter(|row| row.bucket() == Bucket::Seeds)
            .map(|row| row.slug().to_string());
        match target {
            Some(slug) => {
                self.status_line = format!("Promote seed {slug}? y confirm  n/Esc cancel");
                self.pending_promote_slug = Some(slug);
                self.mode = Mode::ConfirmPromote;
            }
            None => {
                self.status_line = "promote is only available for seed rows".to_string();
            }
        }
    }

    fn handle_confirm_promote_key(&mut self, input: KeyInput) -> Outcome {
        match input {
            KeyInput::Char('y') => {
                self.mode = Mode::List;
                match self.pending_promote_slug.take() {
                    Some(slug) => Outcome::PromoteSeed { slug },
                    None => Outcome::Continue,
                }
            }
            KeyInput::Char('n') | KeyInput::Esc => {
                self.pending_promote_slug = None;
                self.mode = Mode::List;
                self.status_line = "promote cancelled".to_string();
                Outcome::Continue
            }
            _ => Outcome::Continue,
        }
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
            KeyInput::Up
            | KeyInput::Down
            | KeyInput::Left
            | KeyInput::Right
            | KeyInput::Enter
            | KeyInput::PageUp
            | KeyInput::PageDown
            | KeyInput::HalfPageUp
            | KeyInput::HalfPageDown => {}
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
        self.prune_hidden_selection();
        self.clamp_range_anchor();
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

    fn clamp_range_anchor(&mut self) {
        let visible_count = self.visible_count();
        if self
            .range_anchor
            .is_some_and(|anchor| anchor >= visible_count)
        {
            self.range_anchor = None;
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

    pub(crate) fn replace_inventory(&mut self, inventory: &Inventory) {
        self.rows = inventory
            .buckets
            .iter()
            .flat_map(|bucket| {
                bucket
                    .items
                    .iter()
                    .map(|item| ListRow::from_item(inventory, item))
            })
            .collect();
        self.preview_cache.borrow_mut().clear();
        self.pending_promote_slug = None;
        self.selected_keys.clear();
        self.range_anchor = None;
        self.mouse_anchor = None;
        self.review_state = None;
        self.mode = Mode::List;
        self.refresh_visibility_state();
    }

    pub(crate) fn select_bucket_slug(&mut self, bucket: Bucket, slug: &str) -> bool {
        self.active_bucket = BucketFilter::Bucket(bucket);
        self.refresh_visibility_state();
        let position = self
            .visible_rows()
            .iter()
            .position(|row| row.bucket() == bucket && row.slug() == slug);
        match position {
            Some(index) => {
                self.selected_index = index;
                self.update_status_line();
                true
            }
            None => false,
        }
    }

    pub(crate) fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_line = message.into();
    }
}

fn searchable_text(
    item: &InventoryItem,
    bucket_label: &str,
    relative_path: &str,
    phase: &str,
    gate: &str,
) -> String {
    let mut parts = vec![
        bucket_label.to_string(),
        bucket_label_plural(item.bucket).to_string(),
        item.slug.clone(),
        relative_path.to_string(),
        phase.to_string(),
        gate.to_string(),
    ];

    if let Some(next_action) = &item.status.next_action {
        parts.push(next_action.clone());
    }

    parts.join(" ")
}

fn markdown_copy_table(rows: &[&ListRow]) -> String {
    markdown_table(&LIST_COLUMNS, rows)
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

fn display_optional(value: &Option<String>, fallback: &str) -> String {
    value.as_deref().unwrap_or(fallback).to_string()
}

fn row_word(count: usize) -> &'static str {
    if count == 1 { "row" } else { "rows" }
}

fn max_review_scroll(document: &ReviewDocument, body_height: usize) -> usize {
    document.lines.len().saturating_sub(body_height)
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
                "learn",
                "intent",
                "write examples",
            ],
        );
        assert!(!rows[0].searchable_text().contains("ready"));

        assert_eq!(rows[1].bucket(), Bucket::Leaves);
        assert_eq!(rows[1].bucket_label(), "leaf");
        assert_eq!(rows[1].relative_path(), ".leaf/02-leaves/beta-leaf");
        assert_eq!(rows[1].parse_state(), ParseState::Partial);

        assert_eq!(rows[2].bucket(), Bucket::Fallen);
        assert_eq!(rows[2].bucket_label(), "fallen");
        assert_eq!(rows[2].phase(), "-");
        assert_eq!(rows[2].gate(), "-");
        assert_eq!(rows[2].relative_path(), ".leaf/03-fallen/gamma-fallen");

        assert_eq!(rows[3].bucket(), Bucket::Pressed);
        assert_eq!(rows[3].bucket_label(), "pressed");
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
        write_preview_status(root.path(), Bucket::Leaves, "first", "첫 번째 미리보기");
        let inventory = inventory_with_root(root.path(), vec![first, second]);
        let mut app = AppState::from_inventory(&inventory);

        write_preview_status(root.path(), Bucket::Leaves, "second", "늦게 생긴 미리보기");
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

    #[test]
    fn tui_app_p_on_seed_enters_confirm_promote_and_y_emits_promote_outcome() {
        let inventory = inventory_with_items(vec![leaf_item(
            Bucket::Seeds,
            "draft",
            complete_leaf_status(),
        )]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char('P')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::ConfirmPromote);
        assert!(app.status_line().contains("Promote seed draft?"));
        assert!(app.status_line().contains("y confirm"));
        assert!(app.status_line().contains("n/Esc cancel"));

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::PromoteSeed {
                slug: "draft".to_string()
            }
        );
        assert_eq!(app.mode(), Mode::List);

        app.handle_key(KeyInput::Char('P'));
        assert_eq!(app.handle_key(KeyInput::Char('n')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::List);
        assert!(app.status_line().contains("cancelled"));

        app.handle_key(KeyInput::Char('P'));
        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Continue);
        assert_eq!(app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_p_on_non_seed_reports_status_without_mutation() {
        let inventory = inventory_with_items(vec![leaf_item(
            Bucket::Leaves,
            "active",
            complete_leaf_status(),
        )]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char('P')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::List);
        assert!(app.status_line().contains("only available for seed"));
    }

    #[test]
    fn tui_app_r_in_list_mode_emits_refresh_outcome() {
        let inventory = inventory_with_items(vec![leaf_item(
            Bucket::Leaves,
            "active",
            complete_leaf_status(),
        )]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char('r')), Outcome::Refresh);
        assert_eq!(app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_r_in_filter_mode_is_filter_text() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('/'));
        assert_eq!(app.handle_key(KeyInput::Char('r')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::FilterInput);
        assert_eq!(app.filter(), "r");
    }

    #[test]
    fn tui_app_enter_opens_review_mode_for_leaf_work_row() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        std::fs::create_dir_all(
            root.path()
                .join(".leaf")
                .join(Bucket::Leaves.dir_name())
                .join(slug)
                .join("01-Learn"),
        )
        .expect("learn dir");
        std::fs::write(
            root.path()
                .join(".leaf")
                .join(Bucket::Leaves.dir_name())
                .join(slug)
                .join("01-Learn/01-intent.md"),
            "# Intent\n\n- read this\n",
        )
        .expect("intent");
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        assert_eq!(app.mode(), Mode::Review);
        let review = app.review_state().expect("review state");
        assert_eq!(review.document.root_relative_path, ".leaf/02-leaves/demo");
        assert!(review.document.visible_text().contains("00-status.md"));
    }

    #[test]
    fn tui_app_enter_on_pressed_digest_row_reports_not_reviewable() {
        let inventory = inventory_with_items(vec![pressed_item("digest", complete_leaf_status())]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        assert_eq!(app.mode(), Mode::List);
        assert!(app.status_line().contains("review is only available"));
    }

    #[test]
    fn tui_app_review_refresh_rebuilds_document_from_disk() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let intent_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        std::fs::write(&intent_path, "# Intent\n\nold text\n").expect("old intent");
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        assert!(
            app.review_state()
                .unwrap()
                .document
                .visible_text()
                .contains("old text")
        );

        std::fs::write(&intent_path, "# Intent\n\nnew text\n").expect("new intent");
        assert_eq!(app.handle_key(KeyInput::Char('r')), Outcome::Continue);

        assert!(
            app.review_state()
                .unwrap()
                .document
                .visible_text()
                .contains("new text")
        );
    }

    #[test]
    fn tui_app_review_auto_refresh_rebuilds_document_when_sources_change() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let intent_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        std::fs::write(&intent_path, "# Intent\n\nold text\n").expect("old intent");
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        app.set_review_body_height(8);
        assert!(
            app.review_state()
                .unwrap()
                .document
                .visible_text()
                .contains("old text")
        );

        std::fs::write(&intent_path, "# Intent\n\nnew text\n").expect("new intent");

        assert!(app.refresh_review_if_changed());
        let review = app.review_state().expect("review state");
        assert!(review.document.visible_text().contains("new text"));
        assert!(review.status_message.contains("updated from source"));
    }

    #[test]
    fn tui_app_review_page_keys_use_rendered_body_height() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let intent_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        let body = (1..=20)
            .map(|line| format!("intent line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&intent_path, format!("# Intent\n\n{body}\n")).expect("intent");
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        app.set_review_body_height(4);
        assert_eq!(app.handle_key(KeyInput::PageDown), Outcome::Continue);

        assert_eq!(app.review_state().unwrap().scroll_offset, 4);

        assert_eq!(app.handle_key(KeyInput::PageUp), Outcome::Continue);
        assert_eq!(app.review_state().unwrap().scroll_offset, 0);
    }

    #[test]
    fn tui_app_review_half_page_keys_use_half_rendered_body_height() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let intent_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug)
            .join("01-Learn/01-intent.md");
        std::fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        let body = (1..=20)
            .map(|line| format!("intent line {line:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&intent_path, format!("# Intent\n\n{body}\n")).expect("intent");
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        app.set_review_body_height(7);
        assert_eq!(app.handle_key(KeyInput::Char('d')), Outcome::Continue);
        assert_eq!(app.review_state().unwrap().scroll_offset, 3);

        assert_eq!(app.handle_key(KeyInput::HalfPageDown), Outcome::Continue);
        assert_eq!(app.review_state().unwrap().scroll_offset, 6);

        assert_eq!(app.handle_key(KeyInput::Char('u')), Outcome::Continue);
        assert_eq!(app.review_state().unwrap().scroll_offset, 3);

        assert_eq!(app.handle_key(KeyInput::HalfPageUp), Outcome::Continue);
        assert_eq!(app.review_state().unwrap().scroll_offset, 0);
    }

    #[test]
    fn tui_app_review_back_returns_to_list_with_selection_preserved() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Continue);

        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row().map(ListRow::slug), Some(slug));
    }

    #[test]
    fn tui_app_review_q_returns_to_list_with_selection_preserved() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        write_preview_status(
            root.path(),
            Bucket::Leaves,
            slug,
            "- current gate: ① Intent\n",
        );
        let item = leaf_item_at(root.path(), Bucket::Leaves, slug, complete_leaf_status());
        let inventory = inventory_with_root(root.path(), vec![item]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        assert_eq!(app.handle_key(KeyInput::Char('q')), Outcome::Continue);

        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row().map(ListRow::slug), Some(slug));
    }

    #[test]
    fn tui_app_replace_inventory_selects_promoted_leaf_and_clears_preview_cache() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let seed = leaf_item_at(root.path(), Bucket::Seeds, "draft", complete_leaf_status());
        write_preview_status(root.path(), Bucket::Seeds, "draft", "old seed preview");
        let seed_inventory = inventory_with_root(root.path(), vec![seed]);
        let mut app = AppState::from_inventory(&seed_inventory);
        assert!(
            preview_text(&app.selected_preview().expect("seed preview"))
                .contains("old seed preview")
        );

        let leaf = leaf_item_at(root.path(), Bucket::Leaves, "draft", complete_leaf_status());
        write_preview_status(root.path(), Bucket::Leaves, "draft", "new leaf preview");
        let leaf_inventory = inventory_with_root(root.path(), vec![leaf]);

        app.replace_inventory(&leaf_inventory);
        app.select_bucket_slug(Bucket::Leaves, "draft");
        app.set_status_message("promoted seed draft to .leaf/02-leaves/draft/");

        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Leaves));
        assert_eq!(app.selected_row().map(ListRow::slug), Some("draft"));
        assert!(
            preview_text(&app.selected_preview().expect("leaf preview"))
                .contains("new leaf preview")
        );
        assert!(app.status_line().contains("promoted seed draft"));
    }

    #[test]
    fn tui_app_copy_row_outcome_copies_selected_row() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            }
        );
        assert_eq!(app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_copy_row_with_no_selection_reports_status() {
        let inventory = inventory_with_items(Vec::new());
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char('y')), Outcome::Continue);
        assert!(app.status_line().contains("no row selected to copy"));
    }

    #[test]
    fn tui_app_copy_row_y_in_filter_mode_is_filter_text() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('/'));
        assert_eq!(app.handle_key(KeyInput::Char('y')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::FilterInput);
        assert_eq!(app.filter(), "y");
    }

    #[test]
    fn tui_app_space_toggles_current_row_selection() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char(' ')), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 1);
        assert!(app.visible_row_is_marked(0));

        app.handle_key(KeyInput::Down);
        assert_eq!(app.handle_key(KeyInput::Char(' ')), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 2);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(!app.visible_row_is_marked(2));

        assert_eq!(app.handle_key(KeyInput::Char(' ')), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 1);
        assert!(app.visible_row_is_marked(0));
        assert!(!app.visible_row_is_marked(1));
        assert_eq!(app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_y_copies_all_multi_selected_rows_in_visible_order() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char(' '));
        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Char(' '));
        assert_eq!(app.selected_row_count(), 2);

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::CopyRows {
                count: 2,
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | gamma | ok |"
                    .to_string(),
            }
        );
        assert_eq!(app.mode(), Mode::List);
    }

    #[test]
    fn tui_app_y_without_multi_selection_keeps_current_row_copy_fallback() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            }
        );
    }

    #[test]
    fn tui_app_v_starts_range_mode_and_j_k_extend_selected_range() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(app.handle_key(KeyInput::Char('v')), Outcome::Continue);
        assert_eq!(app.mode(), Mode::RangeSelect);
        assert_eq!(app.selected_row_count(), 1);
        assert!(app.visible_row_is_marked(0));

        app.handle_key(KeyInput::Char('j'));
        app.handle_key(KeyInput::Char('j'));
        assert_eq!(app.selected_index(), 2);
        assert_eq!(app.selected_row_count(), 3);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));

        app.handle_key(KeyInput::Char('k'));
        assert_eq!(app.selected_index(), 1);
        assert_eq!(app.selected_row_count(), 2);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(!app.visible_row_is_marked(2));

        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Continue);
        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row_count(), 2);
    }

    #[test]
    fn tui_app_v_starts_fresh_range_clearing_prior_selection() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char(' '));
        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Down);
        assert_eq!(app.handle_key(KeyInput::Char('v')), Outcome::Continue);

        assert_eq!(app.mode(), Mode::RangeSelect);
        assert_eq!(app.selected_index(), 2);
        assert_eq!(app.selected_row_count(), 1);
        assert!(!app.visible_row_is_marked(0));
        assert!(!app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));
    }

    #[test]
    fn tui_app_range_mode_v_exits_preserving_selected_range() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('v'));
        app.handle_key(KeyInput::Char('j'));
        assert_eq!(app.handle_key(KeyInput::Char('v')), Outcome::Continue);

        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row_count(), 2);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(!app.visible_row_is_marked(2));
    }

    #[test]
    fn tui_app_range_mode_y_copies_range_and_returns_to_list() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('v'));
        app.handle_key(KeyInput::Char('j'));

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::CopyRows {
                count: 2,
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | beta | ok |"
                    .to_string(),
            }
        );
        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row_count(), 2);
    }

    #[test]
    fn tui_app_range_mode_q_quits() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('v'));

        assert_eq!(app.handle_key(KeyInput::Char('q')), Outcome::Quit);
    }

    #[test]
    fn tui_app_range_mode_a_toggles_all_visible_and_returns_to_list() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('v'));
        assert_eq!(app.handle_key(KeyInput::Char('a')), Outcome::Continue);

        assert_eq!(app.mode(), Mode::List);
        assert_eq!(app.selected_row_count(), 3);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));
    }

    #[test]
    fn tui_app_a_toggles_all_current_visible_rows() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('/'));
        app.handle_key(KeyInput::Char('a'));
        app.handle_key(KeyInput::Esc);
        assert_eq!(visible_slugs(&app), vec!["alpha", "beta", "gamma"]);

        assert_eq!(app.handle_key(KeyInput::Char('a')), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 3);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));

        assert_eq!(app.handle_key(KeyInput::Char('a')), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 0);
    }

    #[test]
    fn tui_app_filter_or_bucket_change_prunes_hidden_selected_rows() {
        let inventory = inventory_with_items(vec![
            leaf_item(Bucket::Seeds, "seed-a", complete_leaf_status()),
            leaf_item(Bucket::Leaves, "leaf-b", complete_leaf_status()),
        ]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char(' '));
        app.handle_key(KeyInput::Down);
        app.handle_key(KeyInput::Char(' '));
        assert_eq!(app.selected_row_count(), 2);

        app.handle_key(KeyInput::Right);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Seeds));
        assert_eq!(visible_slugs(&app), vec!["seed-a"]);
        assert_eq!(app.selected_row_count(), 1);
        assert!(app.visible_row_is_marked(0));
    }

    #[test]
    fn tui_app_esc_clears_selection_before_quitting_list_mode() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char(' '));
        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 0);
        assert_eq!(app.mode(), Mode::List);
        assert!(app.status_line().contains("selection cleared"));

        assert_eq!(app.handle_key(KeyInput::Esc), Outcome::Quit);
    }

    #[test]
    fn tui_app_multi_select_keys_are_filter_text_in_filter_mode() {
        let inventory = inventory_with_slugs(&["alpha"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_key(KeyInput::Char('/'));
        app.handle_key(KeyInput::Char('y'));
        app.handle_key(KeyInput::Char('a'));
        app.handle_key(KeyInput::Char('v'));
        app.handle_key(KeyInput::Char(' '));

        assert_eq!(app.mode(), Mode::FilterInput);
        assert_eq!(app.filter(), "yav ");
        assert_eq!(app.selected_row_count(), 0);
    }

    #[test]
    fn tui_app_mouse_row_click_moves_cursor_without_selecting() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(
            app.handle_mouse(MouseInput::Down {
                visible_index: 1,
                toggle: false,
            }),
            Outcome::Continue
        );

        assert_eq!(app.selected_index(), 1);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("beta"));
        assert_eq!(app.selected_row_count(), 0);
    }

    #[test]
    fn tui_app_mouse_sel_click_toggles_row_selection() {
        let inventory = inventory_with_slugs(&["alpha", "beta"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(
            app.handle_mouse(MouseInput::Down {
                visible_index: 1,
                toggle: true,
            }),
            Outcome::Continue
        );
        assert_eq!(app.selected_index(), 1);
        assert_eq!(app.selected_row_count(), 1);
        assert!(app.visible_row_is_marked(1));
        assert!(!app.visible_row_is_marked(0));

        assert_eq!(
            app.handle_mouse(MouseInput::Down {
                visible_index: 1,
                toggle: true,
            }),
            Outcome::Continue
        );
        assert_eq!(app.selected_row_count(), 0);
    }

    #[test]
    fn tui_app_mouse_drag_marks_range_from_press_to_current_row() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma", "delta"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_mouse(MouseInput::Down {
            visible_index: 0,
            toggle: false,
        });
        app.handle_mouse(MouseInput::Drag { visible_index: 1 });
        assert_eq!(
            app.handle_mouse(MouseInput::Drag { visible_index: 2 }),
            Outcome::Continue
        );

        assert_eq!(app.selected_index(), 2);
        assert_eq!(app.selected_row_count(), 3);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));
        assert!(!app.visible_row_is_marked(3));

        assert_eq!(app.handle_mouse(MouseInput::Up), Outcome::Continue);
        assert_eq!(app.selected_row_count(), 3);
    }

    #[test]
    fn tui_app_mouse_reverse_drag_marks_range_upward() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_mouse(MouseInput::Down {
            visible_index: 2,
            toggle: false,
        });
        app.handle_mouse(MouseInput::Drag { visible_index: 0 });

        assert_eq!(app.selected_index(), 0);
        assert_eq!(app.selected_row_count(), 3);
        assert!(app.visible_row_is_marked(0));
        assert!(app.visible_row_is_marked(1));
        assert!(app.visible_row_is_marked(2));
    }

    #[test]
    fn tui_app_y_copies_rows_marked_by_mouse_drag() {
        let inventory = inventory_with_slugs(&["alpha", "beta", "gamma"]);
        let mut app = AppState::from_inventory(&inventory);

        app.handle_mouse(MouseInput::Down {
            visible_index: 0,
            toggle: false,
        });
        app.handle_mouse(MouseInput::Drag { visible_index: 1 });
        app.handle_mouse(MouseInput::Up);

        assert_eq!(
            app.handle_key(KeyInput::Char('y')),
            Outcome::CopyRows {
                count: 2,
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | beta | ok |"
                    .to_string(),
            }
        );
    }

    #[test]
    fn tui_app_mouse_out_of_range_index_is_ignored() {
        let inventory = inventory_with_slugs(&["alpha", "beta"]);
        let mut app = AppState::from_inventory(&inventory);

        assert_eq!(
            app.handle_mouse(MouseInput::Down {
                visible_index: 9,
                toggle: false,
            }),
            Outcome::Continue
        );
        assert_eq!(app.selected_index(), 0);
        assert_eq!(app.selected_row_count(), 0);

        assert_eq!(
            app.handle_mouse(MouseInput::Down {
                visible_index: 9,
                toggle: true,
            }),
            Outcome::Continue
        );
        assert_eq!(app.selected_row_count(), 0);
    }

    #[test]
    fn tui_app_mouse_is_ignored_in_filter_and_confirm_modes() {
        let inventory = inventory_with_items(vec![leaf_item(
            Bucket::Seeds,
            "draft",
            complete_leaf_status(),
        )]);

        let mut filter_app = AppState::from_inventory(&inventory);
        filter_app.handle_key(KeyInput::Char('/'));
        assert_eq!(
            filter_app.handle_mouse(MouseInput::Down {
                visible_index: 0,
                toggle: true,
            }),
            Outcome::Continue
        );
        assert_eq!(filter_app.mode(), Mode::FilterInput);
        assert_eq!(filter_app.selected_row_count(), 0);

        let mut confirm_app = AppState::from_inventory(&inventory);
        confirm_app.handle_key(KeyInput::Char('P'));
        assert_eq!(confirm_app.mode(), Mode::ConfirmPromote);
        assert_eq!(
            confirm_app.handle_mouse(MouseInput::Down {
                visible_index: 0,
                toggle: true,
            }),
            Outcome::Continue
        );
        assert_eq!(confirm_app.mode(), Mode::ConfirmPromote);
        assert_eq!(confirm_app.selected_row_count(), 0);
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
            review: Some(crate::review::ReviewSource::LeafWork {
                root_path: path,
                root_relative_path: format!(".leaf/{}/{slug}", bucket_dir(bucket)),
            }),
        }
    }

    fn write_preview_status(root: &Path, bucket: Bucket, slug: &str, body: &str) {
        let status_path = root
            .join(".leaf")
            .join(bucket_dir(bucket))
            .join(slug)
            .join("00-status.md");
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
                PreviewLine::ListItem { marker, spans } => {
                    format!("{marker} {}", preview_span_text(spans))
                }
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

    fn preview_span_text(spans: &[crate::preview::PreviewSpan]) -> String {
        spans
            .iter()
            .map(|span| match span {
                crate::preview::PreviewSpan::Plain(text)
                | crate::preview::PreviewSpan::Bold(text)
                | crate::preview::PreviewSpan::Code(text) => text.as_str(),
            })
            .collect()
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
            review: None,
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
