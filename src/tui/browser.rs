use crate::inventory::{Bucket, Inventory};
use crate::tui::app::{AppState, KeyInput, Outcome};
use crate::tui::render::draw;
use crate::tui::session::TerminalSession;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Mutation seam for the leaf list TUI so promotion and reload can be faked in tests
/// without entering a real terminal.
trait PromoteAdapter {
    fn promote_seed(&self, slug: &str) -> Result<()>;
    fn load_inventory(&self) -> Result<Inventory>;
}

struct RealPromoteAdapter {
    repo_root: PathBuf,
}

impl PromoteAdapter for RealPromoteAdapter {
    fn promote_seed(&self, slug: &str) -> Result<()> {
        crate::lifecycle::promote_seed(&self.repo_root, slug).map(|_| ())
    }

    fn load_inventory(&self) -> Result<Inventory> {
        crate::inventory::load(&self.repo_root)
    }
}

pub(crate) fn run(inventory: &Inventory) -> Result<()> {
    let repo_root = repo_root_from_inventory(inventory)?;
    let adapter = RealPromoteAdapter { repo_root };
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

fn event_loop<A: PromoteAdapter>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    adapter: &A,
) -> Result<()> {
    loop {
        terminal
            .draw(|frame| draw(frame, app))
            .context("draw leaf list TUI")?;

        if !event::poll(Duration::from_millis(100)).context("poll leaf list TUI event")? {
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
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}

fn handle_outcome<A: PromoteAdapter>(
    app: &mut AppState,
    adapter: &A,
    outcome: Outcome,
) -> Result<()> {
    match outcome {
        Outcome::Continue | Outcome::Quit => {}
        Outcome::PromoteSeed { slug } => {
            if let Err(err) = adapter.promote_seed(&slug) {
                app.set_status_message(format!("promote failed: {err}"));
                return Ok(());
            }
            match adapter.load_inventory() {
                Ok(inventory) => {
                    app.replace_inventory(&inventory);
                    app.select_bucket_slug(Bucket::Leaves, &slug);
                    app.set_status_message(format!("promoted seed {slug} to .leaf/leaves/{slug}/"));
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
    use std::path::Path;

    struct RecordingPromoteAdapter {
        reloaded: RefCell<Option<Inventory>>,
        error: Option<String>,
        promoted: RefCell<Vec<String>>,
    }

    impl RecordingPromoteAdapter {
        fn success(_repo_root: std::path::PathBuf, reloaded: Inventory) -> Self {
            Self {
                reloaded: RefCell::new(Some(reloaded)),
                error: None,
                promoted: RefCell::new(Vec::new()),
            }
        }

        fn failure(_repo_root: std::path::PathBuf, message: &str) -> Self {
            Self {
                reloaded: RefCell::new(None),
                error: Some(message.to_string()),
                promoted: RefCell::new(Vec::new()),
            }
        }

        fn promoted_slugs(&self) -> Vec<String> {
            self.promoted.borrow().clone()
        }
    }

    impl PromoteAdapter for RecordingPromoteAdapter {
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
    }

    #[test]
    fn promote_outcome_calls_lifecycle_adapter_and_reloads_inventory() {
        let root = assert_fs::TempDir::new().expect("temp repo");
        let seed = test_item(root.path(), Bucket::Seeds, "draft");
        let leaf = test_item(root.path(), Bucket::Leaves, "draft");
        let initial = test_inventory(root.path(), vec![seed]);
        let reloaded = test_inventory(root.path(), vec![leaf]);
        let mut app = AppState::from_inventory(&initial);
        let adapter = RecordingPromoteAdapter::success(root.path().to_path_buf(), reloaded);

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
        let adapter = RecordingPromoteAdapter::failure(
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
        let dir = match bucket {
            Bucket::Seeds => "seeds",
            Bucket::Leaves => "leaves",
            Bucket::Fallen => "fallen",
            Bucket::Pressed => "pressed",
        };
        let path = root.join(".leaf").join(dir).join(slug);
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
            key_input(key(KeyCode::Char('q'))),
            Some(KeyInput::Char('q'))
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
}
