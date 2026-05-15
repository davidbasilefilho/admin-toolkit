use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;
use ratatui_form::field::Field;
use ratatui_form::style::FormStyle;
use ratatui_form::validation::{ValidationError, Validator};
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

pub struct PasswordField {
    id: String,
    label: String,
    value: String,
    cursor_position: usize,
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
            cursor_position: 0,
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
        self.cursor_position = self.value.len();
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
        1
    }

    fn is_required(&self) -> bool {
        self.required
    }

    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle) {
        if area.height < 1 || area.width < 1 {
            return;
        }

        // Render label (same style as TextInput)
        let label_style = if focused {
            style.label_focused
        } else {
            style.label
        };

        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!("{}{}: ", self.label, required_marker);
        let label_width = label_text.width().min(area.width as usize);

        let label_span = Span::styled(&label_text, label_style);
        let label_line = Line::from(label_span);
        let label_area = Rect {
            x: area.x,
            y: area.y,
            width: label_width as u16,
            height: 1,
        };
        label_line.render(label_area, buf);

        // Calculate input area
        let input_x = area.x + label_width as u16;
        let input_width = area.width.saturating_sub(label_width as u16);

        if input_width == 0 {
            return;
        }

        // Determine display text (masked)
        if self.value.is_empty() {
            let display_text = if let Some(ref placeholder) = self.placeholder {
                placeholder.as_str()
            } else {
                ""
            };
            let input_bg_style = if focused {
                style.input_focused
            } else {
                style.input
            };

            for x in input_x..input_x + input_width {
                buf[(x, area.y)].set_style(input_bg_style);
                buf[(x, area.y)].set_char(' ');
            }

            let visible_text: String = display_text.chars().take(input_width as usize).collect();
            let display_style = style.placeholder;
            for (i, c) in visible_text.chars().enumerate() {
                if input_x + i as u16 >= area.x + area.width {
                    break;
                }
                buf[(input_x + i as u16, area.y)].set_char(c);
                buf[(input_x + i as u16, area.y)].set_style(display_style);
            }
        } else {
            // Mask: show bullet per char
            let masked: String = self.value.chars().map(|_| '•').collect();
            self.render_masked(area, buf, focused, style, masked);
        };
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(ch) if !ch.is_control() => {
                self.value.insert(self.cursor_position, ch);
                self.cursor_position += ch.len_utf8();
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    let prev = self.value[..self.cursor_position]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.value.remove(prev);
                    self.cursor_position = prev;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position = self.value[..self.cursor_position]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position = self.value[self.cursor_position..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.cursor_position + i)
                        .unwrap_or(self.value.len());
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                true
            }
            KeyCode::End => {
                self.cursor_position = self.value.len();
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

impl PasswordField {
    fn render_masked(
        &self,
        area: Rect,
        buf: &mut Buffer,
        focused: bool,
        style: &FormStyle,
        masked: String,
    ) {
        let input_x = area.x;
        let input_width = area.width;
        let input_bg_style = if focused {
            style.input_focused
        } else {
            style.input
        };

        // Fill background
        for x in input_x..input_x + input_width {
            buf[(x, area.y)].set_style(input_bg_style);
            buf[(x, area.y)].set_char(' ');
        }

        // Render masked text
        let visible: String = masked.chars().take(input_width as usize).collect();
        for (i, c) in visible.chars().enumerate() {
            let pos = input_x + i as u16;
            if pos >= area.x + area.width {
                break;
            }
            buf[(pos, area.y)].set_char(c);
            buf[(pos, area.y)].set_style(input_bg_style);
        }

        // Cursor
        if focused {
            let cursor_x = input_x + self.value[..self.cursor_position].width() as u16;
            if cursor_x < area.x + area.width {
                buf[(cursor_x, area.y)].set_style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::SLOW_BLINK),
                );
            }
        }
    }
}
