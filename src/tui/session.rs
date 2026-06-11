use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};

pub(crate) trait TerminalEffects {
    fn enter(&self) -> Result<()>;
    fn set_mouse_capture(&self, enabled: bool) -> Result<()>;
    fn leave(&self) -> Result<()>;
}

pub(crate) struct CrosstermEffects;

impl TerminalEffects for CrosstermEffects {
    fn enter(&self) -> Result<()> {
        enter_crossterm_terminal(&RealCrosstermSideEffects)
    }

    fn set_mouse_capture(&self, enabled: bool) -> Result<()> {
        set_crossterm_mouse_capture(&RealCrosstermSideEffects, enabled)
    }

    fn leave(&self) -> Result<()> {
        leave_crossterm_terminal(&RealCrosstermSideEffects)
    }
}

trait CrosstermSideEffects {
    fn enable_raw_mode(&self) -> Result<()>;
    fn enter_alternate_screen(&self) -> Result<()>;
    fn enable_mouse_capture(&self) -> Result<()>;
    fn flush_enter(&self) -> Result<()>;
    fn disable_mouse_capture(&self) -> Result<()>;
    fn leave_alternate_screen(&self) -> Result<()>;
    fn flush_leave(&self) -> Result<()>;
    fn disable_raw_mode(&self) -> Result<()>;
}

struct RealCrosstermSideEffects;

impl CrosstermSideEffects for RealCrosstermSideEffects {
    fn enable_raw_mode(&self) -> Result<()> {
        terminal::enable_raw_mode().context("enable terminal raw mode")
    }

    fn enter_alternate_screen(&self) -> Result<()> {
        execute!(io::stdout(), EnterAlternateScreen).context("enter alternate screen")
    }

    fn enable_mouse_capture(&self) -> Result<()> {
        execute!(io::stdout(), EnableMouseCapture).context("enable mouse capture")
    }

    fn flush_enter(&self) -> Result<()> {
        io::stdout().flush().context("flush alternate screen enter")
    }

    fn disable_mouse_capture(&self) -> Result<()> {
        execute!(io::stdout(), DisableMouseCapture).context("disable mouse capture")
    }

    fn leave_alternate_screen(&self) -> Result<()> {
        execute!(io::stdout(), LeaveAlternateScreen).context("leave alternate screen")
    }

    fn flush_leave(&self) -> Result<()> {
        io::stdout().flush().context("flush alternate screen leave")
    }

    fn disable_raw_mode(&self) -> Result<()> {
        terminal::disable_raw_mode().context("disable terminal raw mode")
    }
}

fn enter_crossterm_terminal(side_effects: &impl CrosstermSideEffects) -> Result<()> {
    side_effects.enable_raw_mode()?;
    if let Err(err) = side_effects.enter_alternate_screen() {
        let _ = side_effects.disable_raw_mode();
        return Err(err);
    }
    if let Err(err) = side_effects.enable_mouse_capture() {
        let _ = side_effects.leave_alternate_screen();
        let _ = side_effects.disable_raw_mode();
        return Err(err);
    }
    if let Err(err) = side_effects.flush_enter() {
        let _ = side_effects.disable_mouse_capture();
        let _ = side_effects.leave_alternate_screen();
        let _ = side_effects.disable_raw_mode();
        return Err(err);
    }
    Ok(())
}

fn set_crossterm_mouse_capture(
    side_effects: &impl CrosstermSideEffects,
    enabled: bool,
) -> Result<()> {
    if enabled {
        side_effects.enable_mouse_capture()
    } else {
        side_effects.disable_mouse_capture()
    }
}

fn leave_crossterm_terminal(side_effects: &impl CrosstermSideEffects) -> Result<()> {
    let screen_result = side_effects
        .leave_alternate_screen()
        .and_then(|()| side_effects.flush_leave());
    let raw_result = side_effects.disable_raw_mode();

    // Attempt every cleanup step, then surface the first failure.
    screen_result.and(raw_result)
}

pub(crate) struct TerminalSession<E: TerminalEffects = CrosstermEffects> {
    effects: E,
    active: bool,
    mouse_capture_enabled: bool,
}

impl TerminalSession<CrosstermEffects> {
    pub(crate) fn enter() -> Result<Self> {
        Self::with_effects(CrosstermEffects)
    }
}

impl<E: TerminalEffects> TerminalSession<E> {
    pub(crate) fn with_effects(effects: E) -> Result<Self> {
        effects.enter()?;
        Ok(Self {
            effects,
            active: true,
            mouse_capture_enabled: true,
        })
    }

    pub(crate) fn set_mouse_capture(&mut self, enabled: bool) -> Result<()> {
        if self.mouse_capture_enabled == enabled {
            return Ok(());
        }

        self.effects.set_mouse_capture(enabled)?;
        self.mouse_capture_enabled = enabled;
        Ok(())
    }

    pub(crate) fn close(mut self) -> Result<()> {
        if self.active {
            let mouse_result = if self.mouse_capture_enabled {
                let result = self.effects.set_mouse_capture(false);
                if result.is_ok() {
                    self.mouse_capture_enabled = false;
                }
                result
            } else {
                Ok(())
            };
            self.active = false;
            let leave_result = self.effects.leave();
            mouse_result.and(leave_result)
        } else {
            Ok(())
        }
    }
}

impl<E: TerminalEffects> Drop for TerminalSession<E> {
    fn drop(&mut self) {
        if self.active {
            if self.mouse_capture_enabled {
                let _ = self.effects.set_mouse_capture(false);
                self.mouse_capture_enabled = false;
            }
            self.active = false;
            let _ = self.effects.leave();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingEffects {
        log: Arc<Mutex<Vec<&'static str>>>,
        fail_mouse_capture: bool,
        fail_leave: bool,
    }

    impl TerminalEffects for RecordingEffects {
        fn enter(&self) -> Result<()> {
            self.log.lock().unwrap().push("enter");
            Ok(())
        }

        fn set_mouse_capture(&self, enabled: bool) -> Result<()> {
            self.log.lock().unwrap().push(if enabled {
                "enable_mouse_capture"
            } else {
                "disable_mouse_capture"
            });
            if self.fail_mouse_capture {
                Err(anyhow!("mouse capture failed"))
            } else {
                Ok(())
            }
        }

        fn leave(&self) -> Result<()> {
            self.log.lock().unwrap().push("leave");
            if self.fail_leave {
                Err(anyhow!("leave failed"))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct RecordingCrosstermSideEffects {
        log: Arc<Mutex<Vec<&'static str>>>,
        fail_enter_flush: bool,
        fail_mouse_capture: bool,
    }

    impl CrosstermSideEffects for RecordingCrosstermSideEffects {
        fn enable_raw_mode(&self) -> Result<()> {
            self.log.lock().unwrap().push("enable_raw_mode");
            Ok(())
        }

        fn enter_alternate_screen(&self) -> Result<()> {
            self.log.lock().unwrap().push("enter_alternate_screen");
            Ok(())
        }

        fn enable_mouse_capture(&self) -> Result<()> {
            self.log.lock().unwrap().push("enable_mouse_capture");
            if self.fail_mouse_capture {
                Err(anyhow!("mouse capture failed"))
            } else {
                Ok(())
            }
        }

        fn flush_enter(&self) -> Result<()> {
            self.log.lock().unwrap().push("flush_enter");
            if self.fail_enter_flush {
                Err(anyhow!("flush failed"))
            } else {
                Ok(())
            }
        }

        fn disable_mouse_capture(&self) -> Result<()> {
            self.log.lock().unwrap().push("disable_mouse_capture");
            Ok(())
        }

        fn leave_alternate_screen(&self) -> Result<()> {
            self.log.lock().unwrap().push("leave_alternate_screen");
            Ok(())
        }

        fn flush_leave(&self) -> Result<()> {
            self.log.lock().unwrap().push("flush_leave");
            Ok(())
        }

        fn disable_raw_mode(&self) -> Result<()> {
            self.log.lock().unwrap().push("disable_raw_mode");
            Ok(())
        }
    }

    #[test]
    fn session_set_mouse_capture_disables_and_enables_idempotently() {
        let effects = RecordingEffects::default();
        let log = Arc::clone(&effects.log);
        let mut session = TerminalSession::with_effects(effects).unwrap();

        session
            .set_mouse_capture(false)
            .expect("disable mouse capture");
        session
            .set_mouse_capture(false)
            .expect("repeat disable is no-op");
        session
            .set_mouse_capture(true)
            .expect("enable mouse capture");
        session
            .set_mouse_capture(true)
            .expect("repeat enable is no-op");
        session.close().expect("close terminal");

        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "enter",
                "disable_mouse_capture",
                "enable_mouse_capture",
                "disable_mouse_capture",
                "leave",
            ]
        );
    }

    #[test]
    fn session_close_after_prior_disable_does_not_disable_mouse_twice() {
        let effects = RecordingEffects::default();
        let log = Arc::clone(&effects.log);
        let mut session = TerminalSession::with_effects(effects).unwrap();

        session
            .set_mouse_capture(false)
            .expect("disable mouse capture");
        session.close().expect("close terminal");

        assert_eq!(
            *log.lock().unwrap(),
            vec!["enter", "disable_mouse_capture", "leave"]
        );
    }

    #[test]
    fn session_mouse_capture_failure_keeps_previous_state_for_cleanup() {
        let effects = RecordingEffects {
            log: Arc::new(Mutex::new(Vec::new())),
            fail_mouse_capture: true,
            fail_leave: false,
        };
        let log = Arc::clone(&effects.log);
        let mut session = TerminalSession::with_effects(effects).unwrap();

        let err = session
            .set_mouse_capture(false)
            .expect_err("disable failure should surface");
        assert_eq!(err.to_string(), "mouse capture failed");
        let err = session
            .close()
            .expect_err("close should surface cleanup failure");
        assert_eq!(err.to_string(), "mouse capture failed");

        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "enter",
                "disable_mouse_capture",
                "disable_mouse_capture",
                "leave"
            ]
        );
    }

    #[test]
    fn session_enters_on_create_and_disables_mouse_capture_before_leaving_on_drop() {
        let effects = RecordingEffects::default();
        let log = Arc::clone(&effects.log);

        {
            let _session = TerminalSession::with_effects(effects).unwrap();
            assert_eq!(*log.lock().unwrap(), vec!["enter"]);
        }

        assert_eq!(
            *log.lock().unwrap(),
            vec!["enter", "disable_mouse_capture", "leave"]
        );
    }

    #[test]
    fn session_close_reports_cleanup_error_and_drop_does_not_leave_again() {
        let effects = RecordingEffects {
            log: Arc::new(Mutex::new(Vec::new())),
            fail_mouse_capture: false,
            fail_leave: true,
        };
        let log = Arc::clone(&effects.log);

        {
            let session = TerminalSession::with_effects(effects).unwrap();
            let err = session
                .close()
                .expect_err("cleanup error should be returned");

            assert_eq!(err.to_string(), "leave failed");
            assert_eq!(
                *log.lock().unwrap(),
                vec!["enter", "disable_mouse_capture", "leave"]
            );
        }

        assert_eq!(
            *log.lock().unwrap(),
            vec!["enter", "disable_mouse_capture", "leave"]
        );
    }

    #[test]
    fn enter_flush_failure_cleans_up_alternate_screen_and_raw_mode() {
        let effects = RecordingCrosstermSideEffects {
            log: Arc::new(Mutex::new(Vec::new())),
            fail_enter_flush: true,
            ..RecordingCrosstermSideEffects::default()
        };
        let log = Arc::clone(&effects.log);

        let err = enter_crossterm_terminal(&effects).expect_err("flush failure should error");

        assert_eq!(err.to_string(), "flush failed");
        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "enable_raw_mode",
                "enter_alternate_screen",
                "enable_mouse_capture",
                "flush_enter",
                "disable_mouse_capture",
                "leave_alternate_screen",
                "disable_raw_mode"
            ]
        );
    }

    #[test]
    fn enter_enables_mouse_capture_before_flush() {
        let effects = RecordingCrosstermSideEffects::default();
        let log = Arc::clone(&effects.log);

        enter_crossterm_terminal(&effects).expect("enter terminal");

        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "enable_raw_mode",
                "enter_alternate_screen",
                "enable_mouse_capture",
                "flush_enter",
            ]
        );
    }

    #[test]
    fn enter_mouse_capture_failure_cleans_up_screen_and_raw_mode() {
        let effects = RecordingCrosstermSideEffects {
            log: Arc::new(Mutex::new(Vec::new())),
            fail_mouse_capture: true,
            ..RecordingCrosstermSideEffects::default()
        };
        let log = Arc::clone(&effects.log);

        let err =
            enter_crossterm_terminal(&effects).expect_err("mouse capture failure should error");

        assert_eq!(err.to_string(), "mouse capture failed");
        assert_eq!(
            *log.lock().unwrap(),
            vec![
                "enable_raw_mode",
                "enter_alternate_screen",
                "enable_mouse_capture",
                "leave_alternate_screen",
                "disable_raw_mode",
            ]
        );
    }

    #[test]
    fn leave_restores_screen_and_raw_mode_after_mouse_capture_is_handled_by_session() {
        let effects = RecordingCrosstermSideEffects::default();
        let log = Arc::clone(&effects.log);

        leave_crossterm_terminal(&effects).expect("leave terminal");

        assert_eq!(
            *log.lock().unwrap(),
            vec!["leave_alternate_screen", "flush_leave", "disable_raw_mode",]
        );
    }
}
