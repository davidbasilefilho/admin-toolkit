use ratatui_form::form::Form;
use serde_json::Value;

use crate::masked_field::PasswordField;

pub const DOMAIN_TARGET: &str = "itu.local";
pub const PREFEITURA_USER: &str = "Prefeitura";

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub hostname: String,
    pub domain: String,
    pub elevated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Edit,
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
    pub hostname_enabled: bool,
    pub password_enabled: bool,
    pub domain_enabled: bool,
    pub hostname_target: String,
    pub password_value: String,
    pub domain_target: String,
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
            hostname_enabled: false,
            password_enabled: false,
            domain_enabled: false,
            hostname_target: String::new(),
            password_value: String::new(),
            domain_target: String::from(DOMAIN_TARGET),
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

    pub fn build_form(&self) -> Form {
        let mut builder = Form::builder()
            .title("Ações em Estágio");

        builder = builder
            .checkbox("hostname_enabled", "Alterar nome do computador")
                .checked(self.hostname_enabled)
                .done()
            .checkbox("password_enabled", "Alterar senha da Prefeitura")
                .checked(self.password_enabled)
                .done()
            .checkbox("domain_enabled", "Alterar domínio para itu.local")
                .checked(self.domain_enabled)
                .done();

        let mut hostname_field = builder
            .text("hostname_target", "Novo nome do computador")
                .placeholder("ex: PC-02")
                .initial_value(self.hostname_target.clone());
        if self.hostname_enabled {
            hostname_field = hostname_field.required();
        }
        builder = hostname_field.done();

        let masked = PasswordField::new("password_value", "Senha da Prefeitura")
            .placeholder("nova senha")
            .initial_value(self.password_value.clone());
        let masked: Box<dyn ratatui_form::field::Field> = if self.password_enabled {
            Box::new(masked.required())
        } else {
            Box::new(masked)
        };
        builder = builder.field(masked);

        let mut domain_field = builder
            .text("domain_target", "Domínio de destino")
                .placeholder("itu.local")
                .initial_value(self.domain_target.clone());
        if self.domain_enabled {
            domain_field = domain_field.required();
        }
        builder = domain_field.done();

        builder.build()
    }

    pub fn extract_form_values(&mut self, json: &Value) {
        self.hostname_enabled = json["hostname_enabled"]
            .as_bool()
            .unwrap_or(false);
        self.password_enabled = json["password_enabled"]
            .as_bool()
            .unwrap_or(false);
        self.domain_enabled = json["domain_enabled"]
            .as_bool()
            .unwrap_or(false);
        self.hostname_target = json["hostname_target"]
            .as_str()
            .unwrap_or("")
            .to_string();
        self.password_value = json["password_value"]
            .as_str()
            .unwrap_or("")
            .to_string();
        self.domain_target = json["domain_target"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Trim values
        self.hostname_target = self.hostname_target.trim().to_string();
        self.password_value = self.password_value.trim().to_string();
        self.domain_target = self.domain_target.trim().to_string();
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
    fn confirm_requires_required_values() {
        let mut state = AppState::new(snapshot());
        state.hostname_enabled = true;

        assert!(!state.can_confirm());

        state.hostname_target = String::from("PC-02");

        assert!(state.can_confirm());
    }

    #[test]
    fn build_and_extract_roundtrip() {
        let mut state = AppState::new(snapshot());
        state.hostname_enabled = true;
        state.hostname_target = String::from("PC-02");
        state.password_enabled = true;
        state.password_value = String::from("secret123");
        state.domain_enabled = true;
        state.domain_target = String::from("demo.local");

        let form = state.build_form();
        let json = form.to_json();

        let mut extracted = AppState::new(snapshot());
        extracted.extract_form_values(&json);

        assert!(extracted.hostname_enabled);
        assert!(extracted.password_enabled);
        assert!(extracted.domain_enabled);
        assert_eq!(extracted.hostname_target, "PC-02");
        assert_eq!(extracted.password_value, "secret123");
        assert_eq!(extracted.domain_target, "demo.local");
    }

    #[test]
    fn extract_from_empty_form() {
        let state = AppState::new(snapshot());
        let form = state.build_form();
        let json = form.to_json();

        let mut extracted = AppState::new(snapshot());
        extracted.extract_form_values(&json);

        assert!(!extracted.hostname_enabled);
        assert!(!extracted.password_enabled);
        assert!(!extracted.domain_enabled);
        assert_eq!(extracted.hostname_target, "");
        assert_eq!(extracted.password_value, "");
        assert_eq!(extracted.domain_target, DOMAIN_TARGET);
    }
}
