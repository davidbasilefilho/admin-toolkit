use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, Screen};
use ratatui_form::Form;

// ── Brutalist monochrome palette ──

const FG_MAIN: Color = Color::White;
const FG_DIM: Color = Color::DarkGray;
const BORDER_FRAME: Color = Color::White;
const BORDER_PANEL: Color = Color::DarkGray;
const SEP: Color = Color::DarkGray;
const INVERT_BG: Color = Color::White;
const INVERT_FG: Color = Color::Black;
const ACCENT: Color = Color::Cyan;
const GREEN: Color = Color::Green;
const YELLOW: Color = Color::Yellow;
const RED: Color = Color::Red;

// ── Block builders ──

fn outer_frame<'a>() -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(BORDER_FRAME))
        .title(Line::from(vec![
            Span::styled(
                " admin-toolkit ",
                Style::default().fg(FG_MAIN).add_modifier(Modifier::BOLD),
            ),
            Span::styled("v1.4.0", Style::default().fg(FG_DIM)),
        ]))
        .title_alignment(Alignment::Left)
}

fn panel<'a>(title: &'a str) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(BORDER_PANEL))
        .title(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Left)
}

// ── Separators ──

fn heavy_sep(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new(Line::from(Span::styled(
        "━".repeat(area.width.saturating_sub(1) as usize),
        Style::default().fg(SEP),
    )));
    frame.render_widget(p, area);
}

fn thin_sep(frame: &mut Frame<'_>, area: Rect) {
    let p = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width.saturating_sub(1) as usize),
        Style::default().fg(SEP),
    )));
    frame.render_widget(p, area);
}

// ── Public entry ──

pub fn render(frame: &mut Frame<'_>, state: &AppState, form: Option<&Form>) {
    match state.screen {
        Screen::Edit => render_edit(frame, state),
        Screen::Input(kind) => render_input(frame, kind, form),
        Screen::Confirm => render_confirm(frame, state),
        Screen::Blocked => render_blocked(frame, state),
        Screen::Result => render_result(frame, state),
    }
}

// ── Edit screen ──

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();

    let outer = outer_frame();
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let has_status = !state.status.is_empty();
    let sys_extra = sys_extra_lines(state);
    let sys_content_height = 5 + sys_extra;   // header + blank + 3 data + extras
    let sys_panel_height = sys_content_height + 2; // +2 for panel borders

    let mut constraints: Vec<Constraint> = vec![
        Constraint::Length(1),                        // 0: subtitle
        Constraint::Length(sys_panel_height as u16),  // 1: sys panel
        Constraint::Length(1),                        // 2: spacing
        Constraint::Min(3),                           // 3: actions panel
    ];

    // status sep + status text (conditional)
    let status_idx = constraints.len(); // 4
    if has_status {
        constraints.push(Constraint::Length(1)); // thin sep
        constraints.push(Constraint::Length(1)); // status row
    }

    // footer sep + footer
    let sep_idx = constraints.len(); // 5 or 6
    constraints.push(Constraint::Length(1)); // heavy sep

    let footer_idx = constraints.len(); // 6 or 7
    constraints.push(Constraint::Length(1)); // footer

    let chunks = Layout::vertical(constraints).split(inner);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "alterações em lote do sistema",
        Style::default().fg(FG_DIM),
    )));
    frame.render_widget(subtitle, chunks[0]);

    // System info panel
    let sys_panel = panel("Sistema");
    frame.render_widget(&sys_panel, chunks[1]);
    let sys_inner = sys_panel.inner(chunks[1]);
    render_system_content(frame, sys_inner, state);

    // Actions panel
    let act_panel = panel("Ações");
    frame.render_widget(&act_panel, chunks[3]);
    let act_inner = act_panel.inner(chunks[3]);
    render_actions_content(frame, act_inner, state);

    // Status row
    if has_status {
        thin_sep(frame, chunks[status_idx]);
        let status_line = Paragraph::new(Line::from(Span::styled(
            &state.status,
            Style::default().fg(YELLOW),
        )));
        frame.render_widget(status_line, chunks[status_idx + 1]);
    }

    // Footer separator
    heavy_sep(frame, chunks[sep_idx]);

    // Footer
    let footer = Paragraph::new(Line::from(Span::styled(
        "↑↓ navegar  espaço alternar  e editar  enter confirmar  esc voltar  q sair",
        Style::default().fg(FG_DIM),
    )));
    frame.render_widget(footer, chunks[footer_idx]);
}

fn sys_extra_lines(state: &AppState) -> usize {
    let mut extra = 0;
    if state.hostname_enabled && !state.hostname_target.is_empty() {
        extra += 1;
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        extra += 1;
    }
    extra
}

fn render_system_content(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let w_col = 12usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "Atual", w_col = w_col),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Destino",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "hostname", w_col = w_col),
            Style::default().fg(FG_DIM),
        ),
        Span::styled(&state.snapshot.hostname, Style::default().fg(FG_MAIN)),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "domínio", w_col = w_col),
            Style::default().fg(FG_DIM),
        ),
        Span::styled(&state.snapshot.domain, Style::default().fg(FG_MAIN)),
        Span::raw("  "),
        Span::styled(DOMAIN_TARGET, Style::default().fg(ACCENT)),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "admin", w_col = w_col),
            Style::default().fg(FG_DIM),
        ),
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
                Style::default().fg(FG_DIM),
            ),
            Span::styled(&state.hostname_target, Style::default().fg(FG_MAIN)),
        ]));
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "novo usuário", w_col = w_col),
                Style::default().fg(FG_DIM),
            ),
            Span::styled(
                &state.create_user_username,
                Style::default().fg(FG_MAIN),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_actions_content(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    let rows = [
        (
            "Alterar nome do computador",
            state.hostname_enabled,
            matches!(state.focus, Focus::Hostname),
            state.hostname_target.as_str(),
        ),
        (
            "Alterar senha da Prefeitura",
            state.password_enabled,
            matches!(state.focus, Focus::Password),
            if state.password_value.is_empty() {
                "precisa de senha"
            } else {
                "senha definida"
            },
        ),
        (
            "Alterar domínio para itu.local",
            state.domain_enabled,
            matches!(state.focus, Focus::Domain),
            state.domain_target.as_str(),
        ),
        (
            "Criar usuário",
            state.create_user_enabled,
            matches!(state.focus, Focus::CreateUser),
            if state.create_user_username.is_empty() {
                "precisa de nome"
            } else {
                state.create_user_username.as_str()
            },
        ),
    ];

    for (label, enabled, focused, note) in &rows {
        let marker = if *enabled { "[x]" } else { "[ ]" };
        let marker_style = if *enabled {
            Style::default().fg(GREEN)
        } else {
            Style::default().fg(FG_DIM)
        };

        let base_style = if *focused {
            Style::default()
                .fg(INVERT_FG)
                .bg(INVERT_BG)
                .add_modifier(Modifier::BOLD)
        } else if *enabled {
            Style::default().fg(FG_MAIN)
        } else {
            Style::default().fg(FG_DIM)
        };

        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::raw(" "),
            Span::styled(*label, base_style),
        ];

        if *enabled && !note.is_empty() {
            spans.push(Span::styled(
                format!("  [{}]", note),
                Style::default().fg(FG_DIM),
            ));
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

// ── Input overlay ──

fn render_input(frame: &mut Frame<'_>, _kind: InputKind, form: Option<&Form>) {
    let area = frame.area();

    let outer = outer_frame();
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    let form_area = centered_rect(70, 20, chunks[0]);

    if let Some(form) = form {
        form.render(form_area, frame.buffer_mut());
    }

    let help = Paragraph::new(Line::from(Span::styled(
        "esc cancelar  enter confirmar",
        Style::default().fg(FG_DIM),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}

// ── Confirm screen ──

fn render_confirm(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();

    let outer = outer_frame();
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([
        Constraint::Min(2),
        Constraint::Length(1), // sep
        Constraint::Length(2), // warnings
        Constraint::Length(1), // footer sep
        Constraint::Length(1), // footer
    ])
    .split(inner);

    let mut lines = state.summary_lines().into_iter().map(Line::from).collect::<Vec<_>>();
    for line in &mut lines {
        let spans = line.spans.clone();
        if !spans.is_empty() {
            line.spans.insert(0, Span::styled("  ", Style::default()));
        }
    }
    frame.render_widget(Paragraph::new(lines), chunks[0]);

    let warn = state.warnings();
    if !warn.is_empty() {
        let warn_lines: Vec<Line> = warn
            .iter()
            .map(|w| {
                Line::from(Span::styled(
                    format!("  ⚠ {}", w),
                    Style::default().fg(YELLOW),
                ))
            })
            .collect();
        frame.render_widget(Paragraph::new(warn_lines), chunks[2]);
    }

    heavy_sep(frame, chunks[3]);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter aplicar  esc voltar",
        Style::default().fg(FG_DIM),
    )));
    frame.render_widget(help, chunks[4]);
}

// ── Blocked screen ──

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();

    let outer = outer_frame();
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let lines = vec![
        Line::from(Span::styled(
            "Privilégios de administrador necessários",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &state.blocked_reason,
            Style::default().fg(FG_MAIN),
        )),
        Line::from(""),
        Line::from(Span::styled("q sair", Style::default().fg(FG_DIM))),
    ];

    let p = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(p, inner);
}

// ── Result screen ──

fn render_result(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();

    let outer = outer_frame();
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([
        Constraint::Min(2),
        Constraint::Length(1), // sep
        Constraint::Length(1), // footer
    ])
    .split(inner);

    let mut lines = vec![Line::from(Span::styled(
        &state.result_message,
        Style::default().fg(FG_MAIN),
    ))];
    if state.reboot_required {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "⚠ Reinicialização necessária.",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), chunks[0]);

    heavy_sep(frame, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter voltar  q sair",
        Style::default().fg(FG_DIM),
    )));
    frame.render_widget(help, chunks[2]);
}

// ── Utility ──

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
