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

// ── Layout helpers ──

/// Splits area into [header, body, separator, footer] with 2-char padding on sides.
fn padded_body(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let inner = Layout::horizontal([
        Constraint::Length(2),
        Constraint::Min(10),
        Constraint::Length(2),
    ])
    .split(area)[1];

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner);

    (chunks[0], chunks[1], chunks[2], chunks[3])
}

/// Full-width separator area (fills padded width).
fn sep_area(outer: Rect) -> Rect {
    Layout::horizontal([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(outer)[1]
}

// ── Edit screen ──

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    let (hdr, body, sep, ftr) = padded_body(frame.area());
    render_header(frame, hdr);
    render_body(frame, body, state);
    render_sep(frame, sep);
    render_footer(frame, ftr);
}

fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let h = Line::from(vec![
        Span::styled("admin-toolkit", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("stage system changes", Style::default().fg(DIM)),
        Span::raw("  "),
        Span::styled("v1.3.3", Style::default().fg(DIM)),
    ]);
    frame.render_widget(Paragraph::new(h), area);
}

fn render_body(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let chunks = Layout::vertical([
        Constraint::Length(6),
        Constraint::Min(4),
        Constraint::Length(2),
    ])
    .split(area);

    render_system(frame, chunks[0], state);
    render_actions(frame, chunks[1], state);
    render_status(frame, chunks[2], &state.status);
}

fn render_sep(frame: &mut Frame<'_>, area: Rect) {
    let full = sep_area(area);
    let line = Line::from(Span::styled(
        "─".repeat(full.width as usize),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(line), full);
}

fn render_system(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let label = Line::from(Span::styled(
        "── SISTEMA ──",
        Style::default().fg(DIM).add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(label), area);

    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    let w = 10usize;

    // Atual (current)
    let left = vec![
        Line::from(Span::styled("  Atual", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled(format!("  {:<w$}", "hostname", w = w), Style::default().fg(DIM)),
            Span::styled(&state.snapshot.hostname, Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<w$}", "domínio", w = w), Style::default().fg(DIM)),
            Span::styled(&state.snapshot.domain, Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<w$}", "admin", w = w), Style::default().fg(DIM)),
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

    // Destino (target)
    let mut right_spans: Vec<Vec<Span>> = vec![
        vec![Span::styled("  Destino", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))],
        vec![
            Span::styled(format!("  {:<w$}", "domínio", w = w), Style::default().fg(DIM)),
            Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
        ],
    ];

    if state.hostname_enabled && !state.hostname_target.is_empty() {
        right_spans.push(vec![
            Span::styled(format!("  {:<w$}", "hostname", w = w), Style::default().fg(DIM)),
            Span::styled(&state.hostname_target, Style::default().fg(WHITE)),
        ]);
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        right_spans.push(vec![
            Span::styled(format!("  {:<w$}", "usuário", w = w), Style::default().fg(DIM)),
            Span::styled(&state.create_user_username, Style::default().fg(WHITE)),
        ]);
    }

    frame.render_widget(
        Paragraph::new(right_spans.into_iter().map(Line::from).collect::<Vec<_>>()),
        cols[1],
    );
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "── AÇÕES ──",
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
    let line = Line::from(vec![
        Span::styled("── STATUS ── ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(text, Style::default().fg(YELLOW)),
    ]);
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
    let (hdr, body, sep, ftr) = padded_body(frame.area());
    render_header(frame, hdr);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    lines.insert(0, Line::from(Span::styled(
        "── REVISÃO ──",
        Style::default().fg(DIM).add_modifier(Modifier::BOLD),
    )));
    for line in &mut lines {
        let spans = line.spans.clone();
        if !spans.is_empty() {
            line.spans.insert(0, Span::raw("  "));
        }
    }
    frame.render_widget(Paragraph::new(lines), body);

    render_sep(frame, sep);

    let warn = state.warnings();
    let warn_text = if warn.is_empty() {
        String::from("Nenhum aviso.")
    } else {
        warn.join(" • ")
    };
    let w = Line::from(vec![
        Span::styled("── STATUS ── ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(warn_text, Style::default().fg(YELLOW)),
    ]);
    frame.render_widget(Paragraph::new(w), ftr);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter aplicar  esc voltar",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, ftr);
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
    let (hdr, body, sep, ftr) = padded_body(frame.area());
    render_header(frame, hdr);

    let mut lines = vec![Line::from(vec![
        Span::styled("── RESULTADO ── ", Style::default().fg(DIM).add_modifier(Modifier::BOLD)),
        Span::styled(&state.result_message, Style::default().fg(WHITE)),
    ])];
    if state.reboot_required {
        lines.push(Line::from(Span::styled(
            "  Reinicialização necessária.",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), body);
    render_sep(frame, sep);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter voltar  q sair",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, ftr);
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
