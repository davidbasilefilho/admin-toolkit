pub const DOMAIN_TARGET: &str = "itu.local";
pub const PREFEITURA_USER: &str = "Prefeitura";

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub hostname: String,
    pub domain: String,
    pub elevated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Hostname,
    Password,
    Domain,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Self::Hostname => Self::Password,
            Self::Password => Self::Domain,
            Self::Domain => Self::Hostname,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Hostname => Self::Domain,
            Self::Password => Self::Hostname,
            Self::Domain => Self::Password,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Hostname,
    Password,
    Domain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Edit,
    Input(InputKind),
    Confirm,
    Blocked,
    Result,
}

#[derive(Debug, Clone)]
pub struct ApplyPlan {
    pub hostname: Option<String>,
    pub password: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub snapshot: SystemSnapshot,
    pub screen: Screen,
    pub focus: Focus,
    pub hostname_enabled: bool,
    pub password_enabled: bool,
    pub domain_enabled: bool,
    pub hostname_target: String,
    pub password_value: String,
    pub domain_target: String,
    pub input_buffer: String,
    pub status: String,
    pub blocked_reason: String,
    pub result_message: String,
    pub reboot_required: bool,
}

impl AppState {
    pub fn new(snapshot: SystemSnapshot) -> Self {
        let blocked_reason = if snapshot.elevated {
            String::new()
        } else {
            String::from("Privilégios de administrador são necessários.")
        };

        Self {
            screen: if snapshot.elevated {
                Screen::Edit
            } else {
                Screen::Blocked
            },
            focus: Focus::Hostname,
            hostname_enabled: false,
            password_enabled: false,
            domain_enabled: false,
            hostname_target: String::new(),
            password_value: String::new(),
            domain_target: String::from(DOMAIN_TARGET),
            input_buffer: String::new(),
            status: String::from("Pronto."),
            blocked_reason,
            result_message: String::new(),
            reboot_required: false,
            snapshot,
        }
    }

    pub fn any_selected(&self) -> bool {
        self.hostname_enabled || self.password_enabled || self.domain_enabled
    }

    pub fn can_confirm(&self) -> bool {
        self.any_selected()
            && (!self.hostname_enabled || !self.hostname_target.is_empty())
            && (!self.password_enabled || !self.password_value.is_empty())
    }

    pub fn begin_input(&mut self, kind: InputKind) {
        self.input_buffer = match kind {
            InputKind::Hostname => self.hostname_target.clone(),
            InputKind::Password => self.password_value.clone(),
            InputKind::Domain => self.domain_target.clone(),
        };
        self.screen = Screen::Input(kind);
        self.status = match kind {
            InputKind::Hostname => String::from("Informe o novo nome do computador."),
            InputKind::Password => String::from("Informe a nova senha da Prefeitura."),
            InputKind::Domain => String::from("Informe o novo domínio."),
        };
    }

    pub fn cancel_input(&mut self) {
        self.input_buffer.clear();
        self.screen = Screen::Edit;
        self.status = String::from("Modo de edição.");
    }

    pub fn commit_input(&mut self, kind: InputKind) {
        let value = self.input_buffer.trim().to_string();
        match kind {
            InputKind::Hostname => self.hostname_target = value,
            InputKind::Password => self.password_value = value,
            InputKind::Domain => self.domain_target = value,
        }
        self.input_buffer.clear();
        self.screen = Screen::Edit;
        self.status = String::from("Destino atualizado.");
    }

    pub fn move_focus_next(&mut self) {
        self.focus = self.focus.next();
        self.status = String::from("Modo de edição.");
    }

    pub fn move_focus_previous(&mut self) {
        self.focus = self.focus.previous();
        self.status = String::from("Modo de edição.");
    }

    pub fn toggle_focused(&mut self) {
        match self.focus {
            Focus::Hostname => self.hostname_enabled = !self.hostname_enabled,
            Focus::Password => self.password_enabled = !self.password_enabled,
            Focus::Domain => self.domain_enabled = !self.domain_enabled,
        }
        self.status = String::from("Seleção atualizada.");
    }

    pub fn selected_plan(&self) -> Option<ApplyPlan> {
        if !self.any_selected() {
            return None;
        }

        Some(ApplyPlan {
            hostname: self.hostname_enabled.then(|| self.hostname_target.clone()),
            password: self.password_enabled.then(|| self.password_value.clone()),
            domain: self.domain_enabled.then(|| self.domain_target.clone()),
        })
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Nome do computador: {}", self.snapshot.hostname),
            format!("Domínio: {}", self.snapshot.domain),
        ];

        if self.hostname_enabled {
            lines.push(format!("Novo nome: {}", self.hostname_target));
        }

        if self.password_enabled {
            lines.push(format!(
                "Senha da Prefeitura: {}",
                mask_text(&self.password_value)
            ));
        }

        if self.domain_enabled {
            lines.push(format!("Domínio de destino: {}", self.domain_target));
        }

        lines
    }

    pub fn warnings(&self) -> Vec<&'static str> {
        let mut warnings = Vec::new();

        if self.hostname_enabled {
            warnings.push("Alteração do nome pode exigir reinicialização.");
        }

        if self.domain_enabled {
            warnings.push("Alteração de domínio pode exigir reinicialização ou novo fluxo.");
        }

        warnings
    }
}

pub fn mask_text(value: &str) -> String {
    if value.is_empty() {
        String::from("<empty>")
    } else {
        "•".repeat(value.chars().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> SystemSnapshot {
        SystemSnapshot {
            hostname: String::from("PC-01"),
            domain: String::from("WORKGROUP"),
            elevated: true,
        }
    }

    #[test]
    fn toggles_actions_independently() {
        let mut state = AppState::new(snapshot());

        state.toggle_focused();
        state.move_focus_next();
        state.toggle_focused();
        state.move_focus_previous();
        state.toggle_focused();

        assert!(state.password_enabled);
        assert!(!state.hostname_enabled);
        assert!(!state.domain_enabled);
    }

    #[test]
    fn summary_reflects_staged_actions() {
        let mut state = AppState::new(snapshot());
        state.hostname_enabled = true;
        state.hostname_target = String::from("PC-02");
        state.password_enabled = true;
        state.password_value = String::from("secret123");
        state.domain_enabled = true;
        state.domain_target = String::from("demo.local");

        let lines = state.summary_lines();

        assert!(lines.iter().any(|line| line.contains("PC-02")));
        assert!(lines.iter().any(|line| line.contains("Prefeitura")));
        assert!(lines.iter().any(|line| line.contains("demo.local")));
    }

    #[test]
    fn domain_defaults_but_can_change() {
        let mut state = AppState::new(snapshot());

        assert_eq!(state.domain_target, DOMAIN_TARGET);

        state.domain_enabled = true;
        state.domain_target = String::from("demo.local");

        let plan = state.selected_plan().unwrap();
        assert_eq!(plan.domain.as_deref(), Some("demo.local"));
    }

    #[test]
    fn statuses_are_localized_to_portuguese() {
        let mut state = AppState::new(snapshot());

        assert_eq!(state.status, "Pronto.");

        state.begin_input(InputKind::Hostname);
        assert_eq!(state.status, "Informe o novo nome do computador.");

        state.cancel_input();
        assert_eq!(state.status, "Modo de edição.");

        state.toggle_focused();
        assert_eq!(state.status, "Seleção atualizada.");

        state.input_buffer = String::from("PC-02");
        state.commit_input(InputKind::Hostname);
        assert_eq!(state.status, "Destino atualizado.");
    }

    #[test]
    fn confirm_requires_required_values() {
        let mut state = AppState::new(snapshot());
        state.hostname_enabled = true;

        assert!(!state.can_confirm());

        state.hostname_target = String::from("PC-02");

        assert!(state.can_confirm());
    }
}
