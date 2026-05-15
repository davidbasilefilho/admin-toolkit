use std::io;
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
#[cfg(test)]
use crossterm::event::KeyModifiers;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui_form::Form;
use ratatui_form::form::FormResult;

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
    form: Option<Form>,
    ops: Ops,
}

impl<Ops: WindowsOps> App<Ops> {
    fn new(snapshot: crate::state::SystemSnapshot, ops: Ops) -> Self {
        Self { state: AppState::new(snapshot), form: None, ops }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, &self.state, self.form.as_ref()))?;
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
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
            Event::Resize(_, _) => Ok(false),
            _ => Ok(false),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        match self.state.screen {
            Screen::Blocked => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(true),
                _ => Ok(false),
            },
            Screen::Edit => self.handle_edit_key(key),
            Screen::Input(kind) => self.handle_input_key(key, kind),
            Screen::Confirm => self.handle_confirm_key(key.code),
            Screen::Result => self.handle_result_key(key.code),
        }
    }

    fn handle_edit_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Up | KeyCode::BackTab => {
                self.state.move_focus_previous();
                Ok(false)
            }
            KeyCode::Down | KeyCode::Tab => {
                self.state.move_focus_next();
                Ok(false)
            }
            KeyCode::Char(' ') => {
                self.state.toggle_focused();
                Ok(false)
            }
            KeyCode::Char('e') => {
                let enabled = match self.state.focus {
                    Focus::Hostname => self.state.hostname_enabled,
                    Focus::Password => self.state.password_enabled,
                    Focus::Domain => self.state.domain_enabled,
                    Focus::CreateUser => self.state.create_user_enabled,
                };
                if enabled {
                    self.begin_edit();
                } else {
                    self.state.status = String::from(
                        "Selecione a ação com espaço antes de editar.",
                    );
                }
                Ok(false)
            }
            KeyCode::Enter => {
                if self.state.can_confirm() {
                    self.state.screen = Screen::Confirm;
                    self.state.status = String::from("Revise as alterações em estágio.");
                } else if matches!(self.state.focus, Focus::Hostname) && self.state.hostname_enabled {
                    self.begin_edit();
                } else if matches!(self.state.focus, Focus::Password) && self.state.password_enabled {
                    self.begin_edit();
                } else if matches!(self.state.focus, Focus::Domain) && self.state.domain_enabled {
                    self.begin_edit();
                } else if matches!(self.state.focus, Focus::CreateUser) && self.state.create_user_enabled {
                    self.begin_edit();
                } else {
                    self.state.status = String::from("Ative as ações e preencha os campos obrigatórios primeiro.");
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn begin_edit(&mut self) {
        let kind = match self.state.focus {
            Focus::Hostname => InputKind::Hostname,
            Focus::Password => InputKind::Password,
            Focus::Domain => InputKind::Domain,
            Focus::CreateUser => InputKind::CreateUser,
        };
        self.form = Some(self.state.build_input_form(kind));
        self.state.begin_input(kind);
    }

    fn handle_input_key(&mut self, key: KeyEvent, kind: InputKind) -> io::Result<bool> {
        // Intercept q and Esc for cancel
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.form = None;
            self.state.cancel_input();
            return Ok(false);
        }

        if let Some(ref mut form) = self.form {
            form.handle_input(key);

            match form.result() {
                FormResult::Submitted => {
                    let json = form.to_json();
                    let value = AppState::extract_input_value(kind, &json);
                    self.form = None;
                    self.state.commit_input(kind, value);
                }
                FormResult::Cancelled => {
                    self.form = None;
                    self.state.cancel_input();
                }
                FormResult::Active => {}
            }
        }

        Ok(false)
    }

    fn handle_confirm_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => {
                self.state.screen = Screen::Edit;
                self.state.status = String::from("Modo de edição.");
                Ok(false)
            }
            KeyCode::Enter => self.apply_staged_changes(),
            _ => Ok(false),
        }
    }

    fn handle_result_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => Ok(true),
            KeyCode::Enter => {
                self.state.screen = Screen::Edit;
                self.state.status = String::from("Modo de edição.");
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn apply_staged_changes(&mut self) -> io::Result<bool> {
        let Some(plan) = self.state.selected_plan() else {
            self.state.status = String::from("Nenhuma ação selecionada.");
            return Ok(false);
        };

        let ApplyOutcome { reboot_required, message } = self.ops.apply(&plan)?;
        self.state.reboot_required = reboot_required;
        self.state.result_message = message;
        self.state.screen = Screen::Result;
        self.state.status = String::from("Aplicação concluída.");
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    struct FakeWindowsOps { applied: Cell<usize> }

    impl WindowsOps for FakeWindowsOps {
        fn snapshot(&self) -> io::Result<crate::state::SystemSnapshot> {
            Ok(crate::state::SystemSnapshot { hostname: String::from("PC-01"), domain: String::from("WORKGROUP"), elevated: true })
        }
        fn apply(&self, plan: &crate::state::ApplyPlan) -> io::Result<ApplyOutcome> {
            self.applied.set(self.applied.get() + 1);
            Ok(ApplyOutcome { reboot_required: plan.hostname.is_some() || plan.domain.is_some(), message: String::from("Aplicado com sucesso.") })
        }
    }

    fn app() -> App<FakeWindowsOps> {
        App::new(crate::state::SystemSnapshot { hostname: String::from("PC-01"), domain: String::from("WORKGROUP"), elevated: true }, FakeWindowsOps::default())
    }

    #[test]
    fn q_quits_from_edit() {
        let mut a = app();
        assert!(a.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())).unwrap());
    }

    #[test]
    fn esc_quits_from_edit() {
        let mut a = app();
        assert!(a.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())).unwrap());
    }

    #[test]
    fn tab_and_backtab_cycle_focus() {
        let mut a = app();
        a.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.focus, Focus::Password));
        a.handle_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.focus, Focus::Hostname));
    }

    #[test]
    fn space_toggles_focused_action() {
        let mut a = app();
        a.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty())).unwrap();
        assert!(a.state.hostname_enabled);
    }

    #[test]
    fn e_opens_input_form() {
        let mut a = app();
        a.state.hostname_enabled = true;
        a.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.screen, Screen::Input(InputKind::Hostname)));
        assert!(a.form.is_some());
    }

    #[test]
    fn esc_from_input_cancels() {
        let mut a = app();
        a.state.hostname_enabled = true;
        a.state.screen = Screen::Input(InputKind::Hostname);
        a.form = Some(a.state.build_input_form(InputKind::Hostname));
        a.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.screen, Screen::Edit));
        assert!(a.form.is_none());
    }

    #[test]
    fn enter_flow_confirm_apply_result() {
        let mut a = app();
        a.state.hostname_enabled = true;
        a.state.hostname_target = String::from("PC-02");
        a.state.screen = Screen::Confirm;
        a.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.screen, Screen::Result));
        assert_eq!(a.ops.applied.get(), 1);
        a.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.screen, Screen::Edit));
    }

    #[test]
    fn confirm_esc_returns_to_edit() {
        let mut a = app();
        a.state.screen = Screen::Confirm;
        a.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())).unwrap();
        assert!(matches!(a.state.screen, Screen::Edit));
    }
}
