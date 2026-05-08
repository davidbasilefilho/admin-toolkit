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
            String::from("Administrator privileges are required.")
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
            input_buffer: String::new(),
            status: String::from("Ready."),
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
        };
        self.screen = Screen::Input(kind);
        self.status = match kind {
            InputKind::Hostname => String::from("Enter new hostname."),
            InputKind::Password => String::from("Enter new password for Prefeitura."),
        };
    }

    pub fn cancel_input(&mut self) {
        self.input_buffer.clear();
        self.screen = Screen::Edit;
        self.status = String::from("Edit mode.");
    }

    pub fn commit_input(&mut self, kind: InputKind) {
        let value = self.input_buffer.trim().to_string();
        match kind {
            InputKind::Hostname => self.hostname_target = value,
            InputKind::Password => self.password_value = value,
        }
        self.input_buffer.clear();
        self.screen = Screen::Edit;
        self.status = String::from("Target updated.");
    }

    pub fn move_focus_next(&mut self) {
        self.focus = self.focus.next();
        self.status = String::from("Edit mode.");
    }

    pub fn move_focus_previous(&mut self) {
        self.focus = self.focus.previous();
        self.status = String::from("Edit mode.");
    }

    pub fn toggle_focused(&mut self) {
        match self.focus {
            Focus::Hostname => self.hostname_enabled = !self.hostname_enabled,
            Focus::Password => self.password_enabled = !self.password_enabled,
            Focus::Domain => self.domain_enabled = !self.domain_enabled,
        }
        self.status = String::from("Selection updated.");
    }

    pub fn selected_plan(&self) -> Option<ApplyPlan> {
        if !self.any_selected() {
            return None;
        }

        Some(ApplyPlan {
            hostname: self.hostname_enabled.then(|| self.hostname_target.clone()),
            password: self.password_enabled.then(|| self.password_value.clone()),
            domain: self.domain_enabled.then(|| String::from(DOMAIN_TARGET)),
        })
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Hostname: {}", self.snapshot.hostname),
            format!("Domain: {}", self.snapshot.domain),
        ];

        if self.hostname_enabled {
            lines.push(format!("Change hostname to: {}", self.hostname_target));
        }

        if self.password_enabled {
            lines.push(format!(
                "Set password for {}: {}",
                PREFEITURA_USER,
                mask_text(&self.password_value)
            ));
        }

        if self.domain_enabled {
            lines.push(format!("Join domain: {}", DOMAIN_TARGET));
        }

        lines
    }

    pub fn warnings(&self) -> Vec<&'static str> {
        let mut warnings = Vec::new();

        if self.hostname_enabled {
            warnings.push("Hostname change may require a reboot.");
        }

        if self.domain_enabled {
            warnings.push("Domain join may require a reboot or rejoin flow.");
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

        let lines = state.summary_lines();

        assert!(lines.iter().any(|line| line.contains("PC-02")));
        assert!(lines.iter().any(|line| line.contains("Prefeitura")));
        assert!(lines.iter().any(|line| line.contains(DOMAIN_TARGET)));
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
