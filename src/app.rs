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
use ratatui_form::form::FormResult;

use crate::state::{AppState, Screen};
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
    form: ratatui_form::Form,
    ops: Ops,
}

impl<Ops: WindowsOps> App<Ops> {
    fn new(snapshot: crate::state::SystemSnapshot, ops: Ops) -> Self {
        let state = AppState::new(snapshot);
        let form = state.build_form();
        Self { state, form, ops }
    }

    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, &self.state, &self.form))?;

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
            Screen::Edit => self.handle_form_key(key),
            Screen::Confirm => self.handle_confirm_key(key.code),
            Screen::Result => self.handle_result_key(key.code),
        }
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> io::Result<bool> {
        // Intercept q for quit
        if matches!(key.code, KeyCode::Char('q')) {
            return Ok(true);
        }

        self.form.handle_input(key);

        match self.form.result() {
            FormResult::Submitted => {
                let json = self.form.to_json();
                self.state.extract_form_values(&json);

                if self.state.can_confirm() {
                    self.state.screen = Screen::Confirm;
                    self.state.status = String::from("Revise as alterações em estágio.");
                } else {
                    self.state.status =
                        String::from("Ative as ações e preencha os campos obrigatórios primeiro.");
                }
            }
            FormResult::Cancelled => {
                // Esc from form: quit
                return Ok(true);
            }
            FormResult::Active => {}
        }

        Ok(false)
    }

    fn handle_confirm_key(&mut self, code: KeyCode) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => {
                self.state.screen = Screen::Edit;
                self.form = self.state.build_form();
                self.state.status = String::from("Modo de edição.");
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
                self.form = self.state.build_form();
                self.state.status = String::from("Modo de edição.");
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn apply_staged_changes(&mut self) -> io::Result<()> {
        let Some(plan) = self.state.selected_plan() else {
            self.state.status = String::from("Nenhuma ação selecionada.");
            return Ok(());
        };

        let ApplyOutcome {
            reboot_required,
            message,
        } = self.ops.apply(&plan)?;

        self.state.reboot_required = reboot_required;
        self.state.result_message = message;
        self.state.screen = Screen::Result;
        self.state.status = String::from("Aplicação concluída.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Default)]
    struct FakeWindowsOps {
        applied: Cell<usize>,
    }

    impl WindowsOps for FakeWindowsOps {
        fn snapshot(&self) -> io::Result<crate::state::SystemSnapshot> {
            Ok(crate::state::SystemSnapshot {
                hostname: String::from("PC-01"),
                domain: String::from("WORKGROUP"),
                elevated: true,
            })
        }

        fn apply(&self, plan: &crate::state::ApplyPlan) -> io::Result<ApplyOutcome> {
            self.applied.set(self.applied.get() + 1);
            Ok(ApplyOutcome {
                reboot_required: plan.hostname.is_some() || plan.domain.is_some(),
                message: String::from("Aplicado com sucesso."),
            })
        }
    }

    fn app() -> App<FakeWindowsOps> {
        App::new(
            crate::state::SystemSnapshot {
                hostname: String::from("PC-01"),
                domain: String::from("WORKGROUP"),
                elevated: true,
            },
            FakeWindowsOps::default(),
        )
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
    fn confirm_and_result_flow() {
        let mut a = app();

        // Set up state as if form was submitted
        a.state.hostname_enabled = true;
        a.state.hostname_target = String::from("PC-02");
        a.state.screen = Screen::Confirm;
        a.state.status = String::from("Revise as alterações em estágio.");

        // Enter on Confirm -> apply + go to Result
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        a.handle_key(enter).unwrap();
        assert!(matches!(a.state.screen, Screen::Result));
        assert_eq!(a.ops.applied.get(), 1);

        // Enter on Result -> back to Edit (form rebuilt)
        a.handle_key(enter).unwrap();
        assert!(matches!(a.state.screen, Screen::Edit));
    }

    #[test]
    fn confirm_esc_returns_to_edit() {
        let mut a = app();
        a.state.hostname_enabled = true;
        a.state.hostname_target = String::from("PC-02");
        a.state.screen = Screen::Confirm;

        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        a.handle_key(esc).unwrap();
        assert!(matches!(a.state.screen, Screen::Edit));
    }

    #[test]
    fn plan_from_enabled_actions() {
        let mut a = app();
        a.state.hostname_enabled = true;
        a.state.hostname_target = String::from("PC-02");
        a.state.domain_enabled = true;
        a.state.domain_target = String::from("demo.local");

        let plan = a.state.selected_plan().unwrap();
        assert_eq!(plan.hostname.as_deref(), Some("PC-02"));
        assert_eq!(plan.domain.as_deref(), Some("demo.local"));
        assert!(plan.password.is_none());
    }

    #[test]
    fn extract_and_confirm_validation() {
        let mut a = app();

        // Simulate form submission: set values, extract, check can_confirm
        a.state.hostname_enabled = true;
        a.state.hostname_target = String::from("PC-02");

        assert!(a.state.can_confirm());

        // Now disable and check again
        a.state.hostname_enabled = false;
        assert!(!a.state.can_confirm());
    }
}
