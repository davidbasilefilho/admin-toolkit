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

// ── Layout ──

fn full_sep(area: Rect) -> Rect {
    // Separator spans full terminal width
    let sep = Layout::horizontal([Constraint::Min(1)]).split(area);
    sep[0]
}

fn pad(area: Rect) -> Rect {
    Layout::horizontal([Constraint::Length(2), Constraint::Min(1), Constraint::Length(2)])
        .split(area)[1]
}

// ── Edit screen ──

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    // Full-width layout: header | sep | body | sep | footer
    let chunks = Layout::vertical([
        Constraint::Length(1),   // header
        Constraint::Length(1),   // separator
        Constraint::Min(6),      // body (system + actions + status)
        Constraint::Length(1),   // separator
        Constraint::Length(1),   // footer
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);

    // Separator between header and body
    let sep1 = full_sep(chunks[1]);
    let line = Line::from(Span::styled(
        "─".repeat(sep1.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line), sep1);

    // Body: system | sep | actions | sep | status
    let body = pad(chunks[2]);
    let body_chunks = Layout::vertical([
        Constraint::Length(4),   // system
        Constraint::Length(1),   // separator
        Constraint::Min(3),      // actions
        Constraint::Length(1),   // separator
        Constraint::Length(1),   // status
    ])
    .split(body);

    render_system(frame, body_chunks[0], state);

    // Separator between system and actions (full width)
    let sep_a = full_sep(body_chunks[1]);
    let line_a = Line::from(Span::styled(
        "─".repeat(sep_a.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line_a), sep_a);

    render_actions(frame, body_chunks[2], state);

    // Separator between actions and status (full width)
    let sep_b = full_sep(body_chunks[3]);
    let line_b = Line::from(Span::styled(
        "─".repeat(sep_b.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line_b), sep_b);

    render_status(frame, body_chunks[4], &state.status);

    // Footer separator (full width)
    let sep2 = full_sep(chunks[3]);
    let line2 = Line::from(Span::styled(
        "─".repeat(sep2.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line2), sep2);

    render_footer(frame, chunks[4]);
}

fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let h = Line::from(vec![
        Span::styled("admin-toolkit", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("stage system changes", Style::default().fg(DIM)),
        Span::raw("  "),
        Span::styled("v1.3.4", Style::default().fg(DIM)),
    ]);
    frame.render_widget(Paragraph::new(h), area);
}

fn render_system(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    let w = 10usize;

    // Atual
    let left = vec![
        Line::from(Span::styled(
            "Atual",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{:<w$}", "hostname", w = w), Style::default().fg(DIM)),
            Span::styled(&state.snapshot.hostname, Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<w$}", "domínio", w = w), Style::default().fg(DIM)),
            Span::styled(&state.snapshot.domain, Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<w$}", "admin", w = w), Style::default().fg(DIM)),
            Span::styled(
                if state.snapshot.elevated { "Sim" } else { "Não" },
                if state.snapshot.elevated {
                    Style::default().fg(GREEN)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(left), cols[0]);

    // Destino
    let mut right = vec![
        Line::from(Span::styled(
            "Destino",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{:<w$}", "domínio", w = w), Style::default().fg(DIM)),
            Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
        ]),
    ];

    if state.hostname_enabled && !state.hostname_target.is_empty() {
        right.push(Line::from(vec![
            Span::styled(format!("{:<w$}", "hostname", w = w), Style::default().fg(DIM)),
            Span::styled(&state.hostname_target, Style::default().fg(WHITE)),
        ]));
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        right.push(Line::from(vec![
            Span::styled(format!("{:<w$}", "usuário", w = w), Style::default().fg(DIM)),
            Span::styled(&state.create_user_username, Style::default().fg(WHITE)),
        ]));
    }

    frame.render_widget(Paragraph::new(right), cols[1]);
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let rows = [
        ("Alterar nome do computador", state.hostname_enabled, matches!(state.focus, Focus::Hostname), state.hostname_target.as_str()),
        ("Alterar senha da Prefeitura", state.password_enabled, matches!(state.focus, Focus::Password), if state.password_value.is_empty() { "precisa de senha" } else { "senha definida" }),
        ("Alterar domínio para itu.local", state.domain_enabled, matches!(state.focus, Focus::Domain), state.domain_target.as_str()),
        ("Criar usuário", state.create_user_enabled, matches!(state.focus, Focus::CreateUser), if state.create_user_username.is_empty() { "precisa de nome" } else { state.create_user_username.as_str() }),
    ];

    let mut lines = Vec::new();

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

        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::raw(" "),
            Span::styled(*label, label_style),
        ];

        if *enabled && !note.is_empty() {
            spans.push(Span::styled(format!("  [{}]", note), Style::default().fg(DIM)));
        }

        lines.push(Line::from(spans));
    }

    // Add blank line after actions for spacing from edge
    lines.push(Line::from(""));

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let line = Line::from(Span::styled(text, Style::default().fg(YELLOW)));
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let shortcuts = "↑↓ navegar  espaço alternar  e editar  enter confirmar  esc voltar  q sair";
    let p = Paragraph::new(Line::from(Span::styled(shortcuts, Style::default().fg(DIM))));
    frame.render_widget(p, area);
}

// ── Input overlay ──

fn render_input(frame: &mut Frame<'_>, _state: &AppState, _kind: InputKind, form: Option<&Form>) {
    let area = centered_rect(70, 30, frame.area());

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
        Constraint::Length(1),
        Constraint::Min(2),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);

    let sep = full_sep(chunks[1]);
    let line = Line::from(Span::styled(
        "─".repeat(sep.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line), sep);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    for line in &mut lines {
        let spans = line.spans.clone();
        if !spans.is_empty() {
            line.spans.insert(0, Span::raw("  "));
        }
    }
    frame.render_widget(Paragraph::new(lines), pad(chunks[2]));

    // Warnings
    let warn = state.warnings();
    if !warn.is_empty() {
        let warn_lines: Vec<Line> = warn
            .iter()
            .map(|w| {
                Line::from(Span::styled(
                    format!("  {}", w),
                    Style::default().fg(YELLOW),
                ))
            })
            .collect();
        frame.render_widget(Paragraph::new(warn_lines), pad(chunks[3]));
    }

    let sep2 = full_sep(chunks[4]);
    let line2 = Line::from(Span::styled(
        "─".repeat(sep2.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line2), sep2);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter aplicar  esc voltar",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, chunks[5]);
}

// ── Blocked screen ──

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(60, 25, frame.area());
    let lines = vec![
        Line::from(Span::styled(
            "Privilégios de administrador necessários",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
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
        Constraint::Length(1),
        Constraint::Min(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, chunks[0]);

    let sep = full_sep(chunks[1]);
    let line = Line::from(Span::styled(
        "─".repeat(sep.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line), sep);

    let mut lines = vec![Line::from(Span::styled(
        &state.result_message,
        Style::default().fg(WHITE),
    ))];
    if state.reboot_required {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Reinicialização necessária.",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), pad(chunks[2]));

    let sep2 = full_sep(chunks[3]);
    let line2 = Line::from(Span::styled(
        "─".repeat(sep2.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line2), sep2);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter voltar  q sair",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, chunks[4]);
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
