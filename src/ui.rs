use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, Screen};
use ratatui_form::Form;

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const GREEN: Color = Color::Green;
const WHITE: Color = Color::White;
const YELLOW: Color = Color::Yellow;

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
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);
    render_body(frame, chunks[1], state);
    render_separator(frame, chunks[2]);
    render_footer(frame, chunks[3]);
}

fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let header = Line::from(vec![
        Span::styled(" admin-toolkit", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("stage system changes", Style::default().fg(DIM)),
        Span::raw("  "),
        Span::styled("v1.3.2", Style::default().fg(DIM)),
    ]);
    frame.render_widget(Paragraph::new(header), area);
}

fn render_body(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    // Split body into info panel + actions + status
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    render_system_info(frame, chunks[0], state);
    render_actions(frame, chunks[1], state);
    render_status(frame, chunks[2], &state.status);
}

fn render_separator(frame: &mut Frame<'_>, area: Rect) {
    let sep = Line::from(Span::styled(
        "──".repeat(area.width.saturating_sub(1) as usize / 2),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(sep), area);
}

fn render_system_info(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "  SYSTEM",
        Style::default().fg(DIM).add_modifier(Modifier::BOLD),
    )));

    let label_w = 10usize;
    lines.push(Line::from(vec![
        Span::styled(format!("  {:<width$}", "hostname", width = label_w), Style::default().fg(DIM)),
        Span::styled(&state.snapshot.hostname, Style::default().fg(WHITE)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("  {:<width$}", "domain", width = label_w), Style::default().fg(DIM)),
        Span::styled(&state.snapshot.domain, Style::default().fg(WHITE)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("  {:<width$}", "target", width = label_w), Style::default().fg(DIM)),
        Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
    ]));

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "  ACTIONS",
        Style::default().fg(DIM).add_modifier(Modifier::BOLD),
    )));

    let rows = [
        ("Alterar nome do computador", state.hostname_enabled, matches!(state.focus, Focus::Hostname), state.hostname_target.as_str()),
        ("Alterar senha da Prefeitura", state.password_enabled, matches!(state.focus, Focus::Password), if state.password_value.is_empty() { "precisa de senha" } else { "senha definida" }),
        ("Alterar domínio para itu.local", state.domain_enabled, matches!(state.focus, Focus::Domain), state.domain_target.as_str()),
        ("Criar usuário", state.create_user_enabled, matches!(state.focus, Focus::CreateUser), if state.create_user_username.is_empty() { "precisa de nome" } else { state.create_user_username.as_str() }),
    ];

    for (label, enabled, focused, note) in &rows {
        let marker = if *enabled { "[x]" } else { "[ ]" };
        let marker_style = if *enabled {
            Style::default().fg(GREEN)
        } else {
            Style::default().fg(DIM)
        };

        let label_style = if *focused {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else if *enabled {
            Style::default().fg(WHITE)
        } else {
            Style::default().fg(DIM)
        };

        let mut row_spans = vec![
            Span::raw("  "),
            Span::styled(marker, marker_style),
            Span::raw(" "),
            Span::styled(*label, label_style),
        ];

        if *enabled && !note.is_empty() {
            row_spans.push(Span::styled(format!("  [{}]", note), Style::default().fg(DIM)));
        }

        lines.push(Line::from(row_spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let status = Line::from(vec![
        Span::styled("  STATUS  ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(text, Style::default().fg(YELLOW)),
    ]);
    frame.render_widget(Paragraph::new(status), area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let shortcuts = "  ↑↓ navegar  espaço alternar  e editar  enter confirmar  esc voltar  q sair";
    let p = Paragraph::new(Line::from(Span::styled(shortcuts, Style::default().fg(DIM))));
    frame.render_widget(p, area);
}

// ── Input overlay ──

fn render_input(frame: &mut Frame<'_>, _state: &AppState, _kind: InputKind, form: Option<&Form>) {
    let area = centered_rect(70, 30, frame.area());

    // Dimmmed backdrop
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

// ── Confirm screen ──

fn render_confirm(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    lines.insert(0, Line::from(Span::styled("  REVIEW", Style::default().fg(DIM).add_modifier(Modifier::BOLD))));
    for line in &mut lines {
        let spans = line.spans.clone();
        if !spans.is_empty() {
            line.spans.insert(0, Span::raw("  "));
        }
    }

    frame.render_widget(Paragraph::new(lines), chunks[1]);
    render_separator(frame, chunks[2]);

    let warn = state.warnings();
    let warn_text = if warn.is_empty() {
        String::from("Nenhum aviso.")
    } else {
        warn.join(" • ")
    };
    let w = Line::from(vec![
        Span::styled("  STATUS  ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(warn_text, Style::default().fg(YELLOW)),
    ]);
    frame.render_widget(Paragraph::new(w), chunks[3]);

    let f = Line::from(Span::styled("  enter aplicar  esc voltar", Style::default().fg(DIM)));
    frame.render_widget(Paragraph::new(f), chunks[4]);
}

// ── Blocked screen ──

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(60, 25, frame.area());
    let lines = vec![
        Line::from(Span::styled("Privilégios de administrador necessários", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(&state.blocked_reason, Style::default().fg(WHITE))),
        Line::from(""),
        Line::from(Span::styled("q sair", Style::default().fg(DIM))),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

// ── Result screen ──

fn render_result(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);

    let mut lines = vec![Line::from(vec![
        Span::styled("  RESULT  ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(&state.result_message, Style::default().fg(WHITE)),
    ])];
    if state.reboot_required {
        lines.push(Line::from(Span::styled(
            "  Reinicialização necessária.",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), chunks[1]);
    render_separator(frame, chunks[2]);

    let f = Line::from(Span::styled("  enter voltar  q sair", Style::default().fg(DIM)));
    frame.render_widget(Paragraph::new(f), chunks[3]);
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
