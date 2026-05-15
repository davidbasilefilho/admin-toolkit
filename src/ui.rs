use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, Screen};
use ratatui_form::Form;

const BG_BAR: Color = Color::DarkGray;
const FG_INVERT: Color = Color::Black;
const BG_INVERT: Color = Color::White;
const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const GREEN: Color = Color::Green;
const WHITE: Color = Color::White;
const YELLOW: Color = Color::Yellow;
const RED: Color = Color::Red;

pub fn render(frame: &mut Frame<'_>, state: &AppState, form: Option<&Form>) {
    match state.screen {
        Screen::Edit => render_edit(frame, state),
        Screen::Input(kind) => render_input(frame, state, kind, form),
        Screen::Confirm => render_confirm(frame, state),
        Screen::Blocked => render_blocked(frame, state),
        Screen::Result => render_result(frame, state),
    }
}

// ── Helpers ──

fn pad(area: Rect) -> Rect {
    Layout::horizontal([Constraint::Length(2), Constraint::Min(1), Constraint::Length(2)])
        .split(area)[1]
}

fn fill_bar(frame: &mut Frame<'_>, area: Rect, bg: Color) {
    let p = Paragraph::new(Line::from(Span::styled(
        " ".repeat(area.width as usize),
        Style::default().fg(bg).bg(bg),
    )));
    frame.render_widget(p, area);
}

// ── Edit screen ──

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();
    let has_status = !state.status.is_empty();

    let outer = Layout::vertical([
        Constraint::Length(1),   // header bar
        Constraint::Length(1),   // separator
        Constraint::Min(6),      // body
        Constraint::Length(1),   // footer separator
        Constraint::Length(1),   // footer
        Constraint::Length(1),   // bottom spacing
    ])
    .split(area);

    render_header_bar(frame, outer[0]);
    fill_bar(frame, outer[1], BG_BAR);

    let body = outer[2];
    let mut body_parts = vec![
        Constraint::Length(4),   // system
        Constraint::Length(1),   // separator
        Constraint::Min(3),      // actions
    ];
    if has_status {
        body_parts.push(Constraint::Length(1));
        body_parts.push(Constraint::Length(1));
    }
    let body_chunks = Layout::vertical(body_parts).split(body);

    render_system(frame, pad(body_chunks[0]), state);
    fill_bar(frame, body_chunks[1], BG_BAR);
    render_actions(frame, pad(body_chunks[2]), state);

    if has_status {
        fill_bar(frame, body_chunks[3], BG_BAR);
        render_status(frame, pad(body_chunks[4]), &state.status);
    }

    fill_bar(frame, outer[3], BG_BAR);
    render_footer(frame, pad(outer[4]));
}

fn render_header_bar(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new(Line::from(Span::styled(
        " admin-toolkit  alterações em lote do sistema",
        Style::default()
            .fg(FG_INVERT)
            .bg(BG_INVERT)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(p, area);
}

fn render_system(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let w_col = 12usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "Atual", w_col = w_col),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("Destino", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    lines.push(Line::from(vec![
        Span::styled(format!("{:<w_col$}", "hostname", w_col = w_col), Style::default().fg(DIM)),
        Span::styled(&state.snapshot.hostname, Style::default().fg(WHITE)),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("{:<w_col$}", "domínio", w_col = w_col), Style::default().fg(DIM)),
        Span::styled(&state.snapshot.domain, Style::default().fg(WHITE)),
        Span::raw("  "),
        Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
    ]));

    lines.push(Line::from(vec![
        Span::styled(format!("{:<w_col$}", "admin", w_col = w_col), Style::default().fg(DIM)),
        Span::styled(
            if state.snapshot.elevated { "Sim" } else { "Não" },
            if state.snapshot.elevated {
                Style::default().fg(GREEN)
            } else {
                Style::default().fg(RED)
            },
        ),
    ]));

    if state.hostname_enabled && !state.hostname_target.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "novo hostname", w_col = w_col),
                Style::default().fg(DIM),
            ),
            Span::styled(&state.hostname_target, Style::default().fg(WHITE)),
        ]));
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "novo usuário", w_col = w_col),
                Style::default().fg(DIM),
            ),
            Span::styled(&state.create_user_username, Style::default().fg(WHITE)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "AÇÕES",
        Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
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

        let base_style = if *focused {
            Style::default().fg(WHITE).bg(BG_BAR).add_modifier(Modifier::BOLD)
        } else if *enabled {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(DIM)
        };

        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::raw(" "),
            Span::styled(*label, base_style),
        ];

        if *enabled && !note.is_empty() {
            spans.push(Span::styled(format!("  [{}]", note), Style::default().fg(DIM)));
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let p = Paragraph::new(Line::from(Span::styled(text, Style::default().fg(YELLOW))));
    frame.render_widget(p, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new(Line::from(Span::styled(
        "↑↓ navegar  espaço alternar  e editar  enter confirmar  esc voltar  q sair",
        Style::default().fg(DIM),
    )));
    frame.render_widget(p, area);
}

// ── Input overlay ──

fn render_input(frame: &mut Frame<'_>, _state: &AppState, _kind: InputKind, form: Option<&Form>) {
    let backdrop = Paragraph::new(Line::from(Span::styled(
        "esc cancelar  enter confirmar",
        Style::default().fg(DIM),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(backdrop, frame.area());

    if let Some(form) = form {
        let area = centered_rect(70, 20, frame.area());
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

    render_header_bar(frame, chunks[0]);
    fill_bar(frame, chunks[1], BG_BAR);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    for line in &mut lines {
        let spans = line.spans.clone();
        if !spans.is_empty() {
            line.spans.insert(0, Span::raw("  "));
        }
    }
    frame.render_widget(Paragraph::new(lines), pad(chunks[2]));

    let warn = state.warnings();
    if !warn.is_empty() {
        let warn_lines: Vec<Line> = warn
            .iter()
            .map(|w| Line::from(Span::styled(format!("  {}", w), Style::default().fg(YELLOW))))
            .collect();
        frame.render_widget(Paragraph::new(warn_lines), pad(chunks[3]));
    }

    fill_bar(frame, chunks[4], BG_BAR);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter aplicar  esc voltar",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, pad(chunks[5]));
}

// ── Blocked screen ──

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(60, 25, frame.area());
    let lines = vec![
        Line::from(Span::styled(
            "Privilégios de administrador necessários",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
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
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header_bar(frame, chunks[0]);
    fill_bar(frame, chunks[1], BG_BAR);

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

    fill_bar(frame, chunks[3], BG_BAR);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter voltar  q sair",
        Style::default().fg(DIM),
    )));
    frame.render_widget(help, pad(chunks[4]));
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
