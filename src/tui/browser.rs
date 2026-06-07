use crate::inventory::Inventory;
use crate::tui::app::{AppState, KeyInput, Outcome};
use crate::tui::render::draw;
use crate::tui::session::TerminalSession;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;

pub(crate) fn run(inventory: &Inventory) -> Result<()> {
    let mut app = AppState::from_inventory(inventory);
    let session = TerminalSession::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("open leaf list TUI terminal")?;

    loop {
        terminal
            .draw(|frame| draw(frame, &app))
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

                if app.handle_key(input) == Outcome::Quit {
                    break;
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    let cursor_result = terminal
        .show_cursor()
        .context("restore leaf list TUI cursor");
    let close_result = session.close().context("restore leaf list TUI terminal");
    cursor_result?;
    close_result?;
    Ok(())
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
    use crate::tui::app::KeyInput;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
