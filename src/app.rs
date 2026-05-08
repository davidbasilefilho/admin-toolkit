use std::io;
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::state::{AppState, Focus, InputKind, Screen};
use crate::ui;
use crate::windows_ops::{ApplyOutcome, RealWindowsOps, WindowsOps};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub fn run_app() -> io::Result<()> {
    let ops = RealWindowsOps::new();
    let snapshot = ops.snapshot()?;
    let mut app = App::new(snapshot, ops);

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

struct App<Ops: WindowsOps> {
    state: AppState,
    ops: Ops,
}

impl<Ops: WindowsOps> App<Ops> {
    fn new(snapshot: crate::state::SystemSnapshot, ops: Ops) -> Self {
        Self {
            state: AppState::new(snapshot),
            ops,
        }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, &self.state))?;

            if event::poll(POLL_INTERVAL)? {
                let event = event::read()?;
                if self.handle_event(event)? {
                    return Ok(());
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> io::Result<bool> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key.code),
            Event::Resize(_, _) => Ok(false),
            _ => Ok(false),
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match self.state.screen {
            Screen::Blocked => match code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(true),
                _ => Ok(false),
            },
            Screen::Edit => self.handle_edit_key(code),
            Screen::Input(kind) => self.handle_input_key(code, kind),
            Screen::Confirm => self.handle_confirm_key(code),
            Screen::Result => self.handle_result_key(code),
        }
    }

    fn handle_edit_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Up => {
                self.state.move_focus_previous();
                Ok(false)
            }
            KeyCode::Down => {
                self.state.move_focus_next();
                Ok(false)
            }
            KeyCode::Char(' ') => {
                self.state.toggle_focused();
                Ok(false)
            }
            KeyCode::Char('e') => {
                self.maybe_begin_input();
                Ok(false)
            }
            KeyCode::Enter => {
                if self.state.can_confirm() {
                    self.state.screen = Screen::Confirm;
                    self.state.status = String::from("Review staged changes.");
                } else if matches!(self.state.focus, Focus::Hostname) && self.state.hostname_enabled
                {
                    self.state.begin_input(InputKind::Hostname);
                } else if matches!(self.state.focus, Focus::Password) && self.state.password_enabled
                {
                    self.state.begin_input(InputKind::Password);
                } else {
                    self.state.status =
                        String::from("Enable actions and fill required values first.");
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn handle_input_key(&mut self, code: KeyCode, kind: InputKind) -> io::Result<bool> {
        match code {
            KeyCode::Esc => {
                self.state.cancel_input();
                Ok(false)
            }
            KeyCode::Enter => {
                self.state.commit_input(kind);
                Ok(false)
            }
            KeyCode::Backspace => {
                self.state.input_buffer.pop();
                Ok(false)
            }
            KeyCode::Char(ch) => {
                self.state.input_buffer.push(ch);
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn handle_confirm_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => {
                self.state.screen = Screen::Edit;
                self.state.status = String::from("Edit mode.");
                Ok(false)
            }
            KeyCode::Enter => {
                self.apply_staged_changes()?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn handle_result_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Enter => {
                self.state.screen = Screen::Edit;
                self.state.status = String::from("Edit mode.");
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn maybe_begin_input(&mut self) {
        match self.state.focus {
            Focus::Hostname if self.state.hostname_enabled => {
                self.state.begin_input(InputKind::Hostname)
            }
            Focus::Password if self.state.password_enabled => {
                self.state.begin_input(InputKind::Password)
            }
            _ => {}
        }
    }

    fn apply_staged_changes(&mut self) -> io::Result<()> {
        let Some(plan) = self.state.selected_plan() else {
            self.state.status = String::from("No actions selected.");
            return Ok(());
        };

        let ApplyOutcome {
            reboot_required,
            message,
        } = self.ops.apply(&plan)?;

        self.state.reboot_required = reboot_required;
        self.state.result_message = message;
        self.state.screen = Screen::Result;
        self.state.status = String::from("Apply complete.");
        Ok(())
    }
}
