use crate::inventory::{Bucket, Inventory};
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

/// Side-effect seam for the leaf list TUI so promotion, reload, and clipboard
/// copy can be faked in tests without entering a real terminal or clipboard.
trait TuiAdapter {
    fn promote_seed(&self, slug: &str) -> Result<()>;
    fn load_inventory(&self) -> Result<Inventory>;
    fn copy_to_clipboard(&self, text: &str) -> Result<()>;
}

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(100);
const REVIEW_AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

struct RealTuiAdapter {
    repo_root: PathBuf,
}

impl TuiAdapter for RealTuiAdapter {
    fn promote_seed(&self, slug: &str) -> Result<()> {
        crate::lifecycle::promote_seed(&self.repo_root, slug).map(|_| ())
    }

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
}

pub(crate) fn run(inventory: &Inventory) -> Result<()> {
    let repo_root = repo_root_from_inventory(inventory)?;
    let adapter = RealTuiAdapter { repo_root };
    let mut app = AppState::from_inventory(inventory);
    let session = TerminalSession::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("open leaf list TUI terminal")?;

    let loop_result = event_loop(&mut terminal, &mut app, &adapter);

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
) -> Result<()> {
    let mut last_review_auto_refresh = Instant::now();
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
            }
            Event::Mouse(mouse) => {
                let size = terminal
                    .size()
                    .context("read leaf list TUI size for mouse event")?;
                let area = Rect::new(0, 0, size.width, size.height);
                let Some(input) = mouse_input(area, app, mouse) else {
                    continue;
                };

                let outcome = app.handle_mouse(input);
                if outcome == Outcome::Quit {
                    break;
                }
                handle_outcome(app, adapter, outcome)?;
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
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
        MouseEventKind::ScrollUp if app.mode() == Mode::Review => Some(MouseInput::ScrollUp),
        MouseEventKind::ScrollDown if app.mode() == Mode::Review => Some(MouseInput::ScrollDown),
        _ => None,
    }
}

fn handle_outcome<A: TuiAdapter>(app: &mut AppState, adapter: &A, outcome: Outcome) -> Result<()> {
    match outcome {
        Outcome::Continue | Outcome::Quit => {}
        Outcome::CopyRow { slug, text } => match adapter.copy_to_clipboard(&text) {
            Ok(()) => app.set_status_message(format!("copied row {slug}")),
            Err(err) => app.set_status_message(format!("copy failed: {err}")),
        },
        Outcome::CopyRows { count, text } => match adapter.copy_to_clipboard(&text) {
            Ok(()) => app.set_status_message(format!("copied {count} {}", row_word(count))),
            Err(err) => app.set_status_message(format!("copy failed: {err}")),
        },
        Outcome::Refresh => match adapter.load_inventory() {
            Ok(inventory) => {
                app.replace_inventory(&inventory);
                app.set_status_message("refreshed");
            }
            Err(err) => {
                app.set_status_message(format!("refresh failed: {err}"));
            }
        },
        Outcome::PromoteSeed { slug } => {
            if let Err(err) = adapter.promote_seed(&slug) {
                app.set_status_message(format!("promote failed: {err}"));
                return Ok(());
            }
            match adapter.load_inventory() {
                Ok(inventory) => {
                    app.replace_inventory(&inventory);
                    app.select_bucket_slug(Bucket::Leaves, &slug);
                    app.set_status_message(format!(
                        "promoted seed {slug} to .leaf/{}/{slug}/",
                        Bucket::Leaves.dir_name()
                    ));
                }
                Err(err) => {
                    app.set_status_message(format!("reload after promote failed: {err}"));
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{
        Bucket, BucketInventory, Inventory, InventoryItem, ItemKind, ParseState, PreviewSource,
        StatusSummary,
    };
    use crate::tui::app::{AppState, BucketFilter, KeyInput, ListRow, Outcome};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;

    struct RecordingTuiAdapter {
        reloaded: RefCell<Option<Inventory>>,
        error: Option<String>,
        promoted: RefCell<Vec<String>>,
        copied: RefCell<Vec<String>>,
        copy_error: Option<String>,
    }

    impl RecordingTuiAdapter {
        fn new(_repo_root: std::path::PathBuf) -> Self {
            Self {
                reloaded: RefCell::new(None),
                error: None,
                promoted: RefCell::new(Vec::new()),
                copied: RefCell::new(Vec::new()),
                copy_error: None,
            }
        }

        fn success(repo_root: std::path::PathBuf, reloaded: Inventory) -> Self {
            let adapter = Self::new(repo_root);
            *adapter.reloaded.borrow_mut() = Some(reloaded);
            adapter
        }

        fn failure(repo_root: std::path::PathBuf, message: &str) -> Self {
            Self {
                error: Some(message.to_string()),
                ..Self::new(repo_root)
            }
        }

        fn with_copy_error(mut self, message: &str) -> Self {
            self.copy_error = Some(message.to_string());
            self
        }

        fn promoted_slugs(&self) -> Vec<String> {
            self.promoted.borrow().clone()
        }

        fn copied_texts(&self) -> Vec<String> {
            self.copied.borrow().clone()
        }
    }

    impl TuiAdapter for RecordingTuiAdapter {
        fn promote_seed(&self, slug: &str) -> Result<()> {
            self.promoted.borrow_mut().push(slug.to_string());
            if let Some(message) = &self.error {
                anyhow::bail!("{message}");
            }
            Ok(())
        }

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
    }

    #[test]
    fn promote_outcome_calls_lifecycle_adapter_and_reloads_inventory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let seed = test_item(root.path(), Bucket::Seeds, "draft");
        let leaf = test_item(root.path(), Bucket::Leaves, "draft");
        let initial = test_inventory(root.path(), vec![seed]);
        let reloaded = test_inventory(root.path(), vec![leaf]);
        let mut app = AppState::from_inventory(&initial);
        let adapter = RecordingTuiAdapter::success(root.path().to_path_buf(), reloaded);

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::PromoteSeed {
                slug: "draft".to_string(),
            },
        )
        .expect("promote outcome");

        assert_eq!(adapter.promoted_slugs(), vec!["draft"]);
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Leaves));
        assert_eq!(app.selected_row().map(ListRow::slug), Some("draft"));
        assert!(app.status_line().contains("promoted seed draft"));
    }

    #[test]
    fn promote_outcome_reports_adapter_failure_without_replacing_inventory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let seed = test_item(root.path(), Bucket::Seeds, "draft");
        let initial = test_inventory(root.path(), vec![seed]);
        let mut app = AppState::from_inventory(&initial);
        let adapter = RecordingTuiAdapter::failure(
            root.path().to_path_buf(),
            "active leaf already exists: draft",
        );

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::PromoteSeed {
                slug: "draft".to_string(),
            },
        )
        .expect("failure is reported in app status, not returned");

        assert_eq!(app.selected_row().map(ListRow::slug), Some("draft"));
        assert!(app.status_line().contains("active leaf already exists"));
    }

    #[test]
    fn refresh_outcome_reloads_inventory_and_reports_refreshed() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![test_item(root.path(), Bucket::Seeds, "draft")],
        );
        let reloaded = test_inventory(
            root.path(),
            vec![
                test_item(root.path(), Bucket::Leaves, "alpha"),
                test_item(root.path(), Bucket::Leaves, "beta"),
            ],
        );
        let mut app = AppState::from_inventory(&initial);
        assert_eq!(app.row_count(), 1);
        let adapter = RecordingTuiAdapter::success(root.path().to_path_buf(), reloaded);

        handle_outcome(&mut app, &adapter, Outcome::Refresh).expect("refresh outcome");

        assert_eq!(app.row_count(), 2);
        assert!(app.status_line().contains("refreshed"));
    }

    #[test]
    fn refresh_outcome_preserves_list_when_load_fails() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let initial = test_inventory(
            root.path(),
            vec![test_item(root.path(), Bucket::Seeds, "draft")],
        );
        let mut app = AppState::from_inventory(&initial);
        // RecordingTuiAdapter::new configures no reloaded inventory, so load_inventory errors.
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(&mut app, &adapter, Outcome::Refresh)
            .expect("failure is reported in app status, not returned");

        assert_eq!(app.row_count(), 1);
        assert_eq!(app.selected_row().map(ListRow::slug), Some("draft"));
        assert!(app.status_line().contains("refresh failed"));
    }

    #[test]
    fn copy_outcome_writes_row_to_clipboard_and_reports_status() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), Bucket::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("copy outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string()
            ]
        );
        assert!(app.status_line().contains("copied row alpha"));
    }

    #[test]
    fn copy_rows_outcome_writes_joined_text_and_reports_count() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), Bucket::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRows {
                count: 2,
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | gamma | ok |"
                    .to_string(),
            },
        )
        .expect("copy rows outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |\n| leaf | learn | intent | gamma | ok |"
                    .to_string()
            ]
        );
        assert!(app.status_line().contains("copied 2 rows"));
    }

    #[test]
    fn copy_rows_outcome_reports_singular_count() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), Bucket::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf());

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRows {
                count: 1,
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("copy one selected row outcome");

        assert_eq!(
            adapter.copied_texts(),
            vec![
                "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string()
            ]
        );
        assert!(app.status_line().contains("copied 1 row"));
        assert!(!app.status_line().contains("copied 1 rows"));
    }

    #[test]
    fn copy_outcome_reports_clipboard_failure_without_exit() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf = test_item(root.path(), Bucket::Leaves, "alpha");
        let mut app = AppState::from_inventory(&test_inventory(root.path(), vec![leaf]));
        let adapter = RecordingTuiAdapter::new(root.path().to_path_buf())
            .with_copy_error("clipboard unavailable");

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::CopyRow {
                slug: "alpha".to_string(),
                text: "| BUCKET | PHASE | GATE | SLUG | STATUS |\n| --- | --- | --- | --- | --- |\n| leaf | learn | intent | alpha | ok |"
                    .to_string(),
            },
        )
        .expect("failure is reported in app status, not returned");

        assert!(app.status_line().contains("copy failed"));
        assert!(app.status_line().contains("clipboard unavailable"));
    }

    #[test]
    fn real_promote_adapter_promotes_seed_and_reloads_inventory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let leaf_root = root.path().join(".leaf");
        fs::create_dir_all(leaf_root.join("01-seeds/demo")).expect("seed dir");
        fs::create_dir_all(leaf_root.join("02-leaves")).expect("leaves dir");
        fs::create_dir_all(leaf_root.join("03-fallen")).expect("fallen dir");
        fs::create_dir_all(leaf_root.join("04-pressed")).expect("pressed dir");
        fs::write(
            leaf_root.join("01-seeds/demo/00-status.md"),
            "# Status\n\n- state: seed\n- current phase: Learn\n- current gate: Intent\n- first missing gate: Example\n- next action: promote\n",
        )
        .expect("seed status");
        let initial = crate::inventory::load(root.path()).expect("initial inventory");
        let mut app = AppState::from_inventory(&initial);
        let adapter = RealTuiAdapter {
            repo_root: root.path().to_path_buf(),
        };

        handle_outcome(
            &mut app,
            &adapter,
            Outcome::PromoteSeed {
                slug: "demo".to_string(),
            },
        )
        .expect("real promote outcome");

        assert!(leaf_root.join("02-leaves/demo").is_dir());
        assert!(!leaf_root.join("01-seeds/demo").exists());
        assert_eq!(app.active_bucket(), BucketFilter::Bucket(Bucket::Leaves));
        assert_eq!(app.selected_row().map(ListRow::slug), Some("demo"));
        assert!(app.status_line().contains("promoted seed demo"));
    }

    #[test]
    fn auto_refresh_review_polls_open_review_documents_after_interval() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "demo";
        let leaf_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
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
            vec![test_item(root.path(), Bucket::Leaves, slug)],
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

    fn test_item(root: &Path, bucket: Bucket, slug: &str) -> InventoryItem {
        let path = root.join(".leaf").join(bucket.dir_name()).join(slug);
        InventoryItem {
            bucket,
            slug: slug.to_string(),
            kind: ItemKind::LeafWork,
            path: path.clone(),
            status: StatusSummary {
                parse_state: ParseState::Ok,
                state: Some("active".to_string()),
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
                root_relative_path: format!(".leaf/{}/{slug}", bucket.dir_name()),
            }),
        }
    }

    #[test]
    fn maps_left_mouse_down_drag_and_up_to_mouse_input() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let app = AppState::from_inventory(&test_inventory(
            root.path(),
            vec![
                test_item(root.path(), Bucket::Leaves, "alpha"),
                test_item(root.path(), Bucket::Leaves, "beta"),
                test_item(root.path(), Bucket::Leaves, "gamma"),
            ],
        ));
        // Rect(0,0,80,10): table data rows start at y=4.
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 20, 4)
            ),
            Some(MouseInput::Down { visible_index: 0 })
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 2, 5)
            ),
            Some(MouseInput::Down { visible_index: 1 })
        );
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Drag(MouseButton::Left), 20, 6)
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
            vec![test_item(root.path(), Bucket::Leaves, "alpha")],
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
        // Left down on the header row is not a data row.
        assert_eq!(
            mouse_input(
                area,
                &app,
                mouse(MouseEventKind::Down(MouseButton::Left), 20, 3)
            ),
            None
        );
    }

    #[test]
    fn maps_mouse_wheel_to_review_scroll_input() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let slug = "alpha";
        let leaf_path = root
            .path()
            .join(".leaf")
            .join(Bucket::Leaves.dir_name())
            .join(slug);
        fs::create_dir_all(&leaf_path).expect("leaf dir");
        fs::write(
            leaf_path.join("00-status.md"),
            "# Status\n\n- current gate: ① Intent\n",
        )
        .expect("status");
        let inventory = test_inventory(
            root.path(),
            vec![test_item(root.path(), Bucket::Leaves, slug)],
        );
        let mut app = AppState::from_inventory(&inventory);
        assert_eq!(app.handle_key(KeyInput::Enter), Outcome::Continue);
        assert_eq!(app.mode(), Mode::Review);
        let area = Rect::new(0, 0, 80, 10);

        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::ScrollDown, 20, 4)),
            Some(MouseInput::ScrollDown)
        );
        assert_eq!(
            mouse_input(area, &app, mouse(MouseEventKind::ScrollUp, 20, 4)),
            Some(MouseInput::ScrollUp)
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
