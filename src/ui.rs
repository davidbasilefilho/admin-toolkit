use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, Screen};
use ratatui_form::Form;

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;

pub fn render(frame: &mut Frame<'_>, state: &AppState, form: Option<&Form>) {
    match state.screen {
        Screen::Edit => render_edit(frame, state),
        Screen::Input(kind) => render_input(frame, state, kind, form),
        Screen::Confirm => render_confirm(frame, state),
        Screen::Blocked => render_blocked(frame, state),
        Screen::Result => render_result(frame, state),
    }
}

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(6),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_snapshot(frame, chunks[0], state);
    render_actions(frame, chunks[1], state);
    render_status(frame, chunks[2], &state.status);
}

fn render_snapshot(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let text = vec![
        Line::from(vec![
            Span::styled("hostname ", Style::default().fg(DIM)),
            Span::styled(&state.snapshot.hostname, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("domain   ", Style::default().fg(DIM)),
            Span::styled(&state.snapshot.domain, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("target   ", Style::default().fg(DIM)),
            Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
        ]),
    ];
    frame.render_widget(Paragraph::new(text), area);
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let rows = vec![
        action_row("Alterar nome do computador", state.hostname_enabled, matches!(state.focus, Focus::Hostname), state.hostname_target.as_str()),
        action_row("Alterar senha da Prefeitura", state.password_enabled, matches!(state.focus, Focus::Password), if state.password_value.is_empty() { "precisa de senha" } else { "senha definida" }),
        action_row("Alterar domínio para itu.local", state.domain_enabled, matches!(state.focus, Focus::Domain), state.domain_target.as_str()),
        action_row("Criar usuário", state.create_user_enabled, matches!(state.focus, Focus::CreateUser), if state.create_user_username.is_empty() { "precisa de nome" } else { state.create_user_username.as_str() }),
    ];

    let mut lines = Vec::new();
    // Add a small header
    lines.push(Line::from(Span::styled("═ AÇÕES", Style::default().fg(DIM))));
    lines.push(Line::from(""));
    for row in rows {
        lines.push(row);
        lines.push(Line::from(""));
    }
    // Shortcuts inline
    lines.push(Line::from(Span::styled(
        "  ↑↓ navegar  espaço alternar  e editar  enter confirmar  q sair",
        Style::default().fg(DIM),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), area);
}

fn action_row<'a>(label: &'a str, enabled: bool, focused: bool, note: &'a str) -> Line<'a> {
    let marker = if enabled { "[x]" } else { "[ ]" };
    let marker_style = if enabled {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(DIM)
    };

    let label_style = if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else if enabled {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(DIM)
    };

    Line::from(vec![
        Span::styled(marker, marker_style),
        Span::raw(" "),
        Span::styled(label, label_style),
        Span::styled(format!("  [{}]", note), Style::default().fg(DIM)),
    ])
}

fn render_input(frame: &mut Frame<'_>, _state: &AppState, _kind: InputKind, form: Option<&Form>) {
    let area = centered_rect(60, 40, frame.area());

    // Backdrop fill hint
    let backdrop = Paragraph::new(Line::from(Span::styled(
        "esc cancelar  enter confirmar",
        Style::default().fg(DIM),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(backdrop, frame.area());

    if let Some(form) = form {
        form.render(area, frame.buffer_mut());
    }
}

fn render_confirm(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(6),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_snapshot(frame, chunks[0], state);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    lines.insert(0, Line::from(Span::styled("═ RESUMO", Style::default().fg(DIM))));
    lines.insert(1, Line::from(""));

    frame.render_widget(Paragraph::new(lines), chunks[1]);

    let warn = state.warnings();
    let warn_text = if warn.is_empty() {
        String::from("Sem aviso de reinicialização.")
    } else {
        warn.join(" • ")
    };
    render_status(frame, chunks[2], &warn_text);
    render_footer(frame, chunks[3], "enter aplicar  esc voltar");
}

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(60, 30, frame.area());
    let lines = vec![
        Line::from(Span::styled("Privilégios de administrador necessários", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(&state.blocked_reason, Style::default().fg(Color::White))),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn render_result(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .split(frame.area());

    render_snapshot(frame, chunks[0], state);

    let mut lines = vec![Line::from(state.result_message.as_str())];
    if state.reboot_required {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Reinicialização necessária.",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), chunks[1]);
    render_footer(frame, chunks[2], "enter voltar  q sair");
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let p = Paragraph::new(Line::from(Span::styled(text, Style::default().fg(Color::Yellow))))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let p = Paragraph::new(Line::from(Span::styled(text, Style::default().fg(DIM))))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vert[1])[1]
}
