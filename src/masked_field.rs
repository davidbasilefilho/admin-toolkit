use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui_form::field::Field;
use ratatui_form::style::FormStyle;
use ratatui_form::validation::{ValidationError, Validator};
use serde_json::Value;

pub struct PasswordField {
    id: String,
    label: String,
    value: String,
    placeholder: Option<String>,
    validators: Vec<Box<dyn Validator>>,
    required: bool,
}

impl PasswordField {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: String::new(),
            placeholder: None,
            validators: Vec::new(),
            required: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn initial_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

}

impl Field for PasswordField {
    fn id(&self) -> &str {
        &self.id
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn height(&self) -> u16 {
        3
    }

    fn is_required(&self) -> bool {
        self.required
    }

    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle) {
        if area.width < 4 {
            return;
        }

        let label_style = if focused {
            style.label_focused
        } else {
            style.label
        };

        let label = Line::from(Span::styled(&self.label, label_style));
        let label_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        label.render(label_area, buf);

        let input_style = if focused {
            style.input_focused
        } else {
            style.input
        };

        let masked: String = self.value.chars().map(|_| '•').collect();
        let display_text = if masked.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                Line::from(Span::styled(
                    format!(" {} ", placeholder),
                    style.placeholder,
                ))
            } else {
                Line::from(Span::raw(""))
            }
        } else {
            Line::from(Span::styled(masked, input_style))
        };

        let input_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 1,
        };
        display_text.render(input_area, buf);
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(ch) if !ch.is_control() => {
                self.value.push(ch);
                true
            }
            KeyCode::Backspace => {
                self.value.pop();
                true
            }
            _ => false,
        }
    }

    fn value(&self) -> Value {
        Value::String(self.value.clone())
    }

    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.required && self.value.trim().is_empty() {
            errors.push(ValidationError {
                field_id: self.id.clone(),
                message: format!("{} é obrigatório.", self.label),
            });
        }

        for validator in &self.validators {
            if let Err(msg) = validator.validate(&self.value) {
                errors.push(ValidationError {
                    field_id: self.id.clone(),
                    message: msg,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
