use crate::inventory::Inventory;
use crate::review::ReviewSource;
use crate::tui::app::{AppState, KeyInput, Mode, MouseInput, Outcome};
use crate::tui::render::draw;
use crate::tui::session::TerminalSession;
use anyhow::{Context, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Side-effect seam for the leaf list TUI so reload and clipboard copy can be
/// faked in tests without entering a real terminal or clipboard.
trait TuiAdapter {
    fn load_inventory(&self) -> Result<Inventory>;
    fn copy_to_clipboard(&self, text: &str) -> Result<()>;
    fn fall(&self, slug: &str, reason: &str) -> Result<()>;
}

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const REVIEW_AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(400);

struct RealTuiAdapter {
    repo_root: PathBuf,
}

impl TuiAdapter for RealTuiAdapter {
    fn load_inventory(&self) -> Result<Inventory> {
        crate::inventory::load(&self.repo_root)
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<()> {
        let mut clipboard = arboard::Clipboard::new().context("open clipboard")?;
        clipboard
            .set_text(text.to_string())
            .context("copy row to clipboard")?;
        Ok(())
    }

    fn fall(&self, slug: &str, reason: &str) -> Result<()> {
        crate::lifecycle::fall_leaf(&self.repo_root, slug, reason)?;
        Ok(())
    }
}

pub(crate) fn run(inventory: &Inventory) -> Result<()> {
    run_app(inventory, AppState::from_inventory(inventory))
}

pub(crate) fn run_review(inventory: &Inventory, source: ReviewSource) -> Result<()> {
    let app = AppState::from_inventory_with_review_source(inventory, source)
        .context("open leaf review reader")?;
    run_app(inventory, app)
}

fn run_app(inventory: &Inventory, mut app: AppState) -> Result<()> {
    let repo_root = repo_root_from_inventory(inventory)?;
    let adapter = RealTuiAdapter { repo_root };
    let mut session = TerminalSession::enter()?;
    sync_mouse_capture(&mut session, app.mode())?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("open leaf list TUI terminal")?;

    let loop_result = event_loop(&mut terminal, &mut app, &adapter, &mut session);

    let cursor_result = terminal
        .show_cursor()
        .context("restore leaf list TUI cursor");
    let close_result = session.close().context("restore leaf list TUI terminal");
    loop_result?;
    cursor_result?;
    close_result?;
    Ok(())
}

fn event_loop<A: TuiAdapter>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    adapter: &A,
    session: &mut TerminalSession,
) -> Result<()> {
    let mut last_review_auto_refresh = Instant::now();
    let mut click_tracker = ClickTracker::default();
    loop {
        maybe_auto_refresh_review(app, &mut last_review_auto_refresh, Instant::now());

        terminal
            .draw(|frame| draw(frame, app))
            .context("draw leaf list TUI")?;

        if !event::poll(EVENT_POLL_INTERVAL).context("poll leaf list TUI event")? {
            continue;
        }

        match event::read().context("read leaf list TUI event")? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                click_tracker.reset();
                if is_ctrl_c(key) {
                    break;
                }

                let Some(input) = key_input(key) else {
                    continue;
                };

                let outcome = app.handle_key(input);
                if outcome == Outcome::Quit {
                    break;
                }
                handle_outcome(app, adapter, outcome)?;
                sync_mouse_capture(session, app.mode())?;
            }
            Event::Mouse(mouse) => {
                let size = terminal
                    .size()
                    .context("read leaf list TUI size for mouse event")?;
                let area = Rect::new(0, 0, size.width, size.height);
                let Some(input) = mouse_input(area, app, mouse) else {
                    continue;
                };
                let input =
                    upgrade_double_click(&mut click_tracker, input, mouse.row, Instant::now());

                let outcome = app.handle_mouse(input);
                if outcome == Outcome::Quit {
                    break;
                }
                handle_outcome(app, adapter, outcome)?;
                sync_mouse_capture(session, app.mode())?;
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}

fn mode_wants_mouse_capture(mode: Mode) -> bool {
    mode != Mode::Review
}

fn sync_mouse_capture(session: &mut TerminalSession, mode: Mode) -> Result<()> {
    session
        .set_mouse_capture(mode_wants_mouse_capture(mode))
        .context("sync leaf list TUI mouse capture")
}

fn maybe_auto_refresh_review(
    app: &mut AppState,
    last_review_auto_refresh: &mut Instant,
    now: Instant,
) -> bool {
    if app.mode() != Mode::Review {
        return false;
    }
    if now.saturating_duration_since(*last_review_auto_refresh) < REVIEW_AUTO_REFRESH_INTERVAL {
        return false;
    }

    *last_review_auto_refresh = now;
    app.refresh_review_if_changed()
}

fn mouse_input(area: Rect, app: &AppState, mouse: MouseEvent) -> Option<MouseInput> {
    if app.mode() == Mode::Review {
        return None;
    }

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            crate::tui::render::table_mouse_target(area, app, mouse.column, mouse.row)
                .map(|visible_index| MouseInput::Down { visible_index })
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            crate::tui::render::table_mouse_target(area, app, mouse.column, mouse.row)
                .map(|visible_index| MouseInput::Drag { visible_index })
        }
        MouseEventKind::Up(MouseButton::Left) => Some(MouseInput::Up),
        _ => None,
    }
}

#[derive(Default)]
struct ClickTracker {
    last_down: Option<(Instant, u16, usize)>,
}

impl ClickTracker {
    fn reset(&mut self) {
        self.last_down = None;
    }
}

fn upgrade_double_click(
    tracker: &mut ClickTracker,
    input: MouseInput,
    screen_row: u16,
    now: Instant,
) -> MouseInput {
    match input {
        MouseInput::Down { visible_index } => {
            if let Some((at, last_screen_row, last_visible_index)) = tracker.last_down
                && last_screen_row == screen_row
                && now.duration_since(at) <= DOUBLE_CLICK_INTERVAL
            {
                tracker.last_down = None;
                MouseInput::DoubleClick {
                    visible_index: last_visible_index,
                }
            } else {
                tracker.last_down = Some((now, screen_row, visible_index));
                MouseInput::Down { visible_index }
            }
        }
        MouseInput::Drag { .. } => {
            tracker.reset();
            input
        }
        _ => input,
    }
}

fn handle_outcome<A: TuiAdapter>(app: &mut AppState, adapter: &A, outcome: Outcome) -> Result<()> {
    match outcome {
        Outcome::Continue | Outcome::Quit => {}
        Outcome::CopyRow { slug, text } => match adapter.copy_to_clipboard(&text) {
            Ok(()) => app.set_notice(format!("copied row {slug}")),
            Err(err) => app.set_notice(format!("copy failed: {err}")),
        },
        Outcome::CopyRows { count, text } => match adapter.copy_to_clipboard(&text) {
            Ok(()) => app.set_notice(format!("copied {count} {}", row_word(count))),
            Err(err) => app.set_notice(format!("copy failed: {err}")),
        },
        Outcome::Refresh => match adapter.load_inventory() {
            Ok(inventory) => {
                app.replace_inventory(&inventory);
                app.set_notice("refreshed");
            }
            Err(err) => {
                app.set_notice(format!("refresh failed: {err}"));
            }
        },
        Outcome::FallRows { slugs, reason } => {
            let mut succeeded = 0usize;
            let mut failures: Vec<String> = Vec::new();
            for slug in &slugs {
                match adapter.fall(slug, &reason) {
                    Ok(()) => succeeded += 1,
                    Err(err) => failures.push(format!("{slug}: {err}")),
                }
            }

            let mut notice = format!("fell {succeeded} {}", item_word(succeeded));
            if !failures.is_empty() {
                notice.push_str(&format!(
                    ", {} failed ({})",
                    failures.len(),
                    failures.join("; ")
                ));
            }
            match adapter.load_inventory() {
                Ok(inventory) => app.replace_inventory(&inventory),
                Err(err) => notice.push_str(&format!("; reload failed: {err}")),
            }
            app.set_notice(notice);
        }
    }
    Ok(())
}

fn repo_root_from_inventory(inventory: &Inventory) -> Result<PathBuf> {
    inventory
        .leaf_root
        .parent()
        .map(Path::to_path_buf)
        .context("leaf root has no parent repository directory")
}

fn key_input(key: KeyEvent) -> Option<KeyInput> {
    match key.code {
        KeyCode::Up => Some(KeyInput::Up),
        KeyCode::Down => Some(KeyInput::Down),
        KeyCode::Left => Some(KeyInput::Left),
        KeyCode::Right => Some(KeyInput::Right),
        KeyCode::Enter => Some(KeyInput::Enter),
        KeyCode::PageUp => Some(KeyInput::PageUp),
        KeyCode::PageDown => Some(KeyInput::PageDown),
        KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers == KeyModifiers::CONTROL => {
            Some(KeyInput::HalfPageDown)
        }
        KeyCode::Char('u') | KeyCode::Char('U') if key.modifiers == KeyModifiers::CONTROL => {
            Some(KeyInput::HalfPageUp)
        }
        KeyCode::Esc => Some(KeyInput::Esc),
        KeyCode::Backspace => Some(KeyInput::Backspace),
        KeyCode::Char(ch) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
            Some(KeyInput::Char(ch))
        }
        _ => None,
    }
}

fn is_ctrl_c(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn row_word(count: usize) -> &'static str {
    if count == 1 { "row" } else { "rows" }
}

fn item_word(count: usize) -> &'static str {
    if count == 1 { "item" } else { "items" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        Inventory, InventoryItem, ItemKind, ParseState, PreviewSource, StageDir, StageInventory,
        StatusSummary,
    };
    use crate::tui::app::{AppState, KeyInput, ListRow, Outcome};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;

    struct RecordingTuiAdapter {
        reloaded: RefCell<Option<Inventory>>,
        copied: RefCell<Vec<String>>,
        copy_error: Option<String>,
        falled: RefCell<Vec<(String, String)>>,
        fall_error_slug: Option<String>,
    }

    impl RecordingTuiAdapter {
        fn new(_repo_root: std::path::PathBuf) -> Self {
            Self {
                reloaded: RefCell::new(None),
                copied: RefCell::new(Vec::new()),
                copy_error: None,
                falled: RefCell::new(Vec::new()),
                fall_error_slug: None,
            }
        }

        fn success(repo_root: std::path::PathBuf, reloaded: Inventory) -> Self {
            let adapter = Self::new(repo_root);
            *adapter.reloaded.borrow_mut() = Some(reloaded);
            adapter
        }

        fn with_copy_error(mut self, message: &str) -> Self {
            self.copy_error = Some(message.to_string());
            self
        }

        fn with_fall_error(mut self, slug: &str) -> Self {
            self.fall_error_slug = Some(slug.to_string());
            self
        }

        fn copied_texts(&self) -> Vec<String> {
            self.copied.borrow().clone()
        }

        fn falled_calls(&self) -> Vec<(String, String)> {
            self.falled.borrow().clone()
        }
    }

    impl TuiAdapter for RecordingTuiAdapter {
        fn load_inventory(&self) -> Result<Inventory> {
            self.reloaded
                .borrow_mut()
                .take()
                .context("no reloaded inventory configured")
        }

        fn copy_to_clipboard(&self, text: &str) -> Result<()> {
            self.copied.borrow_mut().push(text.to_string());
            if let Some(message) = &self.copy_error {
                anyhow::bail!("{message}");
            }
            Ok(())
        }

        fn fall(&self, slug: &str, reason: &str) -> Result<()> {
            self.falled
                .borrow_mut()
                .push((slug.to_string(), reason.to_string()));
            if self.fall_error_slug.as_deref() == Some(slug) {
                anyhow::bail!("already fallen: {slug}");
            }
            Ok(())
        }
    }

    #[test]
    fn fall_rows_outcome_falls_each_slug_reloads_and_reports_count() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![
                test_item(root.path(), StageDir::Sprouts, "alpha"),
                test_item(root.path(), StageDir::Sprouts, "beta"),
            ],
        );
        let reloaded = test_inventory(root.path(), vec![]);
        let mut app = AppState::from_inventory(&initial);
        let adapter = RecordingTuiAdapter::success(root.path().to_path_buf(), reloaded);

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::FallRows {
                slugs: vec!["alpha".to_string(), "beta".to_string()],
                reason: "cleanup".to_string(),
            },
        )
        .expect("fall rows outcome");

        assert_eq!(
            adapter.falled_calls(),
            vec![
                ("alpha".to_string(), "cleanup".to_string()),
                ("beta".to_string(), "cleanup".to_string()),
            ]
        );
        assert_eq!(app.row_count(), 0);
        assert_eq!(app.notice(), "fell 2 items");
    }

    #[test]
    fn fall_rows_outcome_reports_partial_failure_and_still_reloads() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![
                test_item(root.path(), StageDir::Sprouts, "alpha"),
                test_item(root.path(), StageDir::Sprouts, "beta"),
            ],
        );
        let reloaded = test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Sprouts, "beta")],
        );
        let mut app = AppState::from_inventory(&initial);
        let adapter = RecordingTuiAdapter::success(root.path().to_path_buf(), reloaded)
            .with_fall_error("beta");

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::FallRows {
                slugs: vec!["alpha".to_string(), "beta".to_string()],
                reason: "cleanup".to_string(),
            },
        )
        .expect("fall rows outcome");

        assert_eq!(app.row_count(), 1);
        assert!(app.notice().contains("fell 1 item"));
        assert!(app.notice().contains("1 failed"));
        assert!(app.notice().contains("beta"));
    }

    #[test]
    fn refresh_outcome_reloads_inventory_and_reports_refreshed() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Sprouts, "draft")],
        );
        let reloaded = test_inventory(
            root.path(),
            vec![
                test_item(root.path(), StageDir::Leaves, "alpha"),
                test_item(root.path(), StageDir::Leaves, "beta"),
            ],
        );
        let mut app = AppState::from_inventory(&initial);
        assert_eq!(app.row_count(), 1);
        let adapter = RecordingTuiAdapter::success(root.path().to_path_buf(), reloaded);

        handle_outcome(&mut app, &adapter, Outcome::Refresh).expect("refresh outcome");

        assert_eq!(app.row_count(), 2);
        assert_eq!(app.notice(), "refreshed");
        assert!(!app.status_line().contains("refreshed"));
    }

    #[test]
    fn refresh_outcome_preserves_list_when_load_fails() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Sprouts, "draft")],
        );
        let mut app = AppState::from_inventory(&initial);
        // RecordingTuiAdapter::new configures no reloaded inventory, so load_inventory errors.
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(&mut app, &adapter, Outcome::Refresh)
            .expect("failure is reported in app status, not returned");

        assert_eq!(app.row_count(), 1);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("draft"));
        assert!(app.notice().contains("refresh failed"));
        assert!(!app.status_line().contains("refresh failed"));
    }

    #[test]
    fn copy_outcome_writes_row_to_clipboard_and_reports_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), StageDir::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("copy outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string()
            ]
        );
        assert_eq!(app.notice(), "copied row alpha");
        assert!(!app.status_line().contains("copied row alpha"));
    }

    #[test]
    fn copy_rows_outcome_writes_joined_text_and_reports_count() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), StageDir::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRows {
                count: 2,
                text: "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | gamma | ok |"
                    .to_string(),
            },
        )
        .expect("copy rows outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | gamma | ok |"
                    .to_string()
            ]
        );
        assert_eq!(app.notice(), "copied 2 rows");
        assert!(!app.status_line().contains("copied 2 rows"));
    }

    #[test]
    fn copy_rows_outcome_reports_singular_count() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), StageDir::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRows {
                count: 1,
                text: "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("copy one selected row outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string()
            ]
        );
        assert_eq!(app.notice(), "copied 1 row");
        assert!(!app.notice().contains("copied 1 rows"));
        assert!(!app.status_line().contains("copied 1 row"));
    }

    #[test]
    fn copy_outcome_reports_clipboard_failure_without_exit() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), StageDir::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf())
            .with_copy_error("clipboard unavailable");

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| STAGE | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("failure is reported in app status, not returned");

        assert!(app.notice().contains("copy failed"));
        assert!(app.notice().contains("clipboard unavailable"));
        assert!(!app.status_line().contains("copy failed"));
        assert!(!app.status_line().contains("clipboard unavailable"));
    }

    #[test]
    fn auto_refresh_review_polls_open_review_documents_after_interval() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        let leaf_path = root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        let intent_path = leaf_path.join("01-Learn/01-intent.md");
        fs::create_dir_all(intent_path.parent().unwrap()).expect("intent dir");
        fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        fs::write(&intent_path, "# Intent\n\nold text\n").expect("old intent");
        let inventory = test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Leaves, slug)],
        );
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);

        let start = std::time::Instant::now();
        let mut last_refresh = start;
        fs::write(&intent_path, "# Intent\n\nnew text\n").expect("new intent");

        assert!(!maybe_auto_refresh_review(
            &mut app,
            &mut last_refresh,
            start + REVIEW_AUTO_REFRESH_INTERVAL - Duration::from_millis(1),
        ));
        assert!(
            app.review_state()
                .unwrap()
                .document
                .visible_text()
                .contains("old text")
        );

        assert!(maybe_auto_refresh_review(
            &mut app,
            &mut last_refresh,
            start + REVIEW_AUTO_REFRESH_INTERVAL,
        ));
        assert!(
            app.review_state()
                .unwrap()
                .document
                .visible_text()
                .contains("new text")
        );
    }

    fn test_inventory(root: &Path, items: Vec<InventoryItem>) -> Inventory {
        let mut sprouts = Vec::new();
        let mut leaves = Vec::new();
        let mut fallen = Vec::new();
        let mut pressed = Vec::new();

        for item in items {
            match item.stage_dir {
                StageDir::Sprouts => sprouts.push(item),
                StageDir::Leaves => leaves.push(item),
                StageDir::Fallen => fallen.push(item),
                StageDir::Pressed => pressed.push(item),
            }
        }

        Inventory {
            leaf_root: root.join(".leaf"),
            stages: vec![
                StageInventory {
                    stage_dir: StageDir::Sprouts,
                    items: sprouts,
                },
                StageInventory {
                    stage_dir: StageDir::Leaves,
                    items: leaves,
                },
                StageInventory {
                    stage_dir: StageDir::Fallen,
                    items: fallen,
                },
                StageInventory {
                    stage_dir: StageDir::Pressed,
                    items: pressed,
                },
            ],
        }
    }

    fn test_item(root: &Path, stage_dir: StageDir, slug: &str) -> InventoryItem {
        let path = root.join(".leaf").join(stage_dir.dir_name()).join(slug);
        InventoryItem {
            stage_dir,
            slug: slug.to_string(),
            kind: ItemKind::LeafWork,
            path: path.clone(),
            status: StatusSummary {
                parse_state: ParseState::Ok,
                stage: Some("sprout".to_string()),
                legacy_state: Some("active".to_string()),
                fallen_reason: None,
                current_phase: Some("learn".to_string()),
                current_gate: Some("intent".to_string()),
                first_missing_gate: None,
                next_action: Some("write next".to_string()),
                missing_fields: Vec::new(),
            },
            preview: PreviewSource::LeafWork {
                status_path: path.join("00-status.md"),
                intent_path: path.join("01-Learn/01-intent.md"),
                unknowns_path: path.join("01-Learn/02-unknowns.md"),
                criteria_path: path.join("02-Example/03-criteria.md"),
            },
            review: Some(crate::review::ReviewSource::LeafWork {
                root_path: path,
                root_relative_path: format!(".leaf/{}/{slug}", stage_dir.dir_name()),
            }),
        }
    }

    #[test]
    fn upgrade_double_click_promotes_second_click_on_same_row_within_interval() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        assert_eq!(
            upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0),
            MouseInput::Down { visible_index: 1 }
        );
        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(300),
            ),
            MouseInput::DoubleClick { visible_index: 1 }
        );
    }

    #[test]
    fn upgrade_double_click_promotes_same_screen_row_with_first_visible_index() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        assert_eq!(
            upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 6 }, 4, t0),
            MouseInput::Down { visible_index: 6 }
        );
        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 3 },
                4,
                t0 + Duration::from_millis(300),
            ),
            MouseInput::DoubleClick { visible_index: 6 }
        );
    }

    #[test]
    fn upgrade_double_click_accepts_click_at_exact_interval_boundary() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 0 }, 4, t0);

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 0 },
                4,
                t0 + DOUBLE_CLICK_INTERVAL,
            ),
            MouseInput::DoubleClick { visible_index: 0 }
        );
    }

    #[test]
    fn upgrade_double_click_keeps_slow_second_click_as_single() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + DOUBLE_CLICK_INTERVAL + Duration::from_millis(1),
            ),
            MouseInput::Down { visible_index: 1 }
        );
    }

    #[test]
    fn upgrade_double_click_keeps_click_on_different_row_as_single() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 2 },
                5,
                t0 + Duration::from_millis(100),
            ),
            MouseInput::Down { visible_index: 2 }
        );
    }

    #[test]
    fn upgrade_double_click_reset_clears_pending_click() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);
        tracker.reset();

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(100),
            ),
            MouseInput::Down { visible_index: 1 }
        );
    }

    #[test]
    fn upgrade_double_click_resets_after_promotion_so_triple_click_starts_fresh() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);
        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(100),
            ),
            MouseInput::DoubleClick { visible_index: 1 }
        );

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(200),
            ),
            MouseInput::Down { visible_index: 1 }
        );
    }

    #[test]
    fn upgrade_double_click_passes_up_through_and_keeps_click_sequence() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);
        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Up,
                4,
                t0 + Duration::from_millis(50),
            ),
            MouseInput::Up
        );

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(200),
            ),
            MouseInput::DoubleClick { visible_index: 1 }
        );
    }

    #[test]
    fn upgrade_double_click_resets_on_drag() {
        let mut tracker = ClickTracker::default();
        let t0 = Instant::now();

        upgrade_double_click(&mut tracker, MouseInput::Down { visible_index: 1 }, 4, t0);
        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Drag { visible_index: 2 },
                4,
                t0 + Duration::from_millis(50),
            ),
            MouseInput::Drag { visible_index: 2 }
        );

        assert_eq!(
            upgrade_double_click(
                &mut tracker,
                MouseInput::Down { visible_index: 1 },
                4,
                t0 + Duration::from_millis(100),
            ),
            MouseInput::Down { visible_index: 1 }
        );
    }

    #[test]
    fn maps_left_mouse_down_drag_and_up_to_mouse_input() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let app = AppState::from_inventory(&test_inventory(
            root.path(),
            vec![
                test_item(root.path(), StageDir::Leaves, "alpha"),
                test_item(root.path(), StageDir::Leaves, "beta"),
                test_item(root.path(), StageDir::Leaves, "gamma"),
            ],
        ));
        // Rect(0,0,80,10): header y=0..1, notice y=2, table data rows start at y=5.
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 20, 5)
            ),
            Some(MouseInput::Down { visible_index: 0 })
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 2, 6)
            ),
            Some(MouseInput::Down { visible_index: 1 })
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Drag(MouseButton::Left), 20, 7)
            ),
            Some(MouseInput::Drag { visible_index: 2 })
        );
        // Up maps even when the row is outside the table area.
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Up(MouseButton::Left), 20, 0)
            ),
            Some(MouseInput::Up)
        );
    }

    #[test]
    fn ignores_non_left_mouse_and_non_table_coordinates() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let app = AppState::from_inventory(&test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Leaves, "alpha")],
        ));
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Right), 20, 4)
            ),
            None
        );
        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::ScrollDown, 20, 4)),
            None
        );
        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::ScrollUp, 20, 4)),
            None
        );
        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::Moved, 20, 4)),
            None
        );
        // Left down on the table header row is not a data row.
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 20, 4)
            ),
            None
        );
    }

    #[test]
    fn mode_wants_mouse_capture_only_outside_review() {
        assert!(mode_wants_mouse_capture(Mode::List));
        assert!(mode_wants_mouse_capture(Mode::RangeSelect));
        assert!(mode_wants_mouse_capture(Mode::FilterInput));
        assert!(!mode_wants_mouse_capture(Mode::Review));
    }

    #[test]
    fn review_mode_mouse_input_is_ignored_for_native_terminal_selection() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "alpha";
        let leaf_path = root
            .path()
            .join(".leaf")
            .join(StageDir::Leaves.dir_name())
            .join(slug);
        fs::create_dir_all(&leaf_path).expect("leaf dir");
        fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        let inventory = test_inventory(
            root.path(),
            vec![test_item(root.path(), StageDir::Leaves, slug)],
        );
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        assert_eq!(app.mode(), Mode::Review);
        let area = Rect::new(0, 0, 80, 12);

        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::ScrollDown, 20, 4)),
            None
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 20, 4)
            ),
            None
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Drag(MouseButton::Left), 20, 4)
            ),
            None
        );
    }

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn maps_navigation_filter_preview_and_quit_keys() {
        assert_eq!(key_input(key(KeyCode::Up)), Some(KeyInput::Up));
        assert_eq!(key_input(key(KeyCode::Down)), Some(KeyInput::Down));
        assert_eq!(key_input(key(KeyCode::Left)), Some(KeyInput::Left));
        assert_eq!(key_input(key(KeyCode::Right)), Some(KeyInput::Right));
        assert_eq!(key_input(key(KeyCode::Esc)), Some(KeyInput::Esc));
        assert_eq!(
            key_input(key(KeyCode::Backspace)),
            Some(KeyInput::Backspace)
        );
        assert_eq!(
            key_input(key(KeyCode::Char('/'))),
            Some(KeyInput::Char('/'))
        );
        assert_eq!(
            key_input(key(KeyCode::Char('p'))),
            Some(KeyInput::Char('p'))
        );
        assert_eq!(
            key_input(key(KeyCode::Char(' '))),
            Some(KeyInput::Char(' '))
        );
        assert_eq!(
            key_input(key(KeyCode::Char('a'))),
            Some(KeyInput::Char('a'))
        );
        assert_eq!(
            key_input(key(KeyCode::Char('v'))),
            Some(KeyInput::Char('v'))
        );
        assert_eq!(
            key_input(key(KeyCode::Char('q'))),
            Some(KeyInput::Char('q'))
        );
    }

    #[test]
    fn key_input_maps_review_reader_navigation_keys() {
        assert_eq!(key_input(key(KeyCode::Enter)), Some(KeyInput::Enter));
        assert_eq!(key_input(key(KeyCode::PageUp)), Some(KeyInput::PageUp));
        assert_eq!(key_input(key(KeyCode::PageDown)), Some(KeyInput::PageDown));
        assert_eq!(
            key_input(ctrl_key(KeyCode::Char('d'))),
            Some(KeyInput::HalfPageDown)
        );
        assert_eq!(
            key_input(ctrl_key(KeyCode::Char('u'))),
            Some(KeyInput::HalfPageUp)
        );
    }

    #[test]
    fn ctrl_c_is_not_sent_to_app_state() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert!(is_ctrl_c(key));
        assert_eq!(key_input(key), None);
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }
}
