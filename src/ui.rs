use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, Screen};
use crate::theme::Theme;
use ratatui_form::Form;

// ── Block builders ──

fn outer_frame<'a>(theme: &Theme) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.border_frame))
}

fn panel<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(theme.border_panel))
        .style(Style::default().bg(theme.bg))
        .title(title)
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
}

// ── Separators ──

fn heavy_sep(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let p = Paragraph::new(Line::from(Span::styled(
        "━".repeat(area.width.saturating_sub(1) as usize),
        Style::default().fg(theme.sep),
    )));
    frame.render_widget(p, area);
}

fn thin_sep(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let p = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width.saturating_sub(1) as usize),
        Style::default().fg(theme.sep),
    )));
    frame.render_widget(p, area);
}

// ── Public entry ──

pub fn render(frame: &mut Frame<'_>, state: &AppState, form: Option<&Form>) {
    let theme = Theme::cohesive_dark();

    match state.screen {
        Screen::Edit => render_edit(frame, state, &theme),
        Screen::Input(kind) => render_input(frame, kind, form, &theme),
        Screen::Confirm => render_confirm(frame, state, &theme),
        Screen::Blocked => render_blocked(frame, state, &theme),
        Screen::Result => render_result(frame, state, &theme),
    }
}

// ── Edit screen ──

fn render_edit(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let outer = outer_frame(theme);
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let has_status = !state.status.is_empty();
    let sys_extra = sys_extra_lines(state);
    let sys_content_height = 5 + sys_extra;
    let sys_panel_height = sys_content_height + 2;

    let mut constraints: Vec<Constraint> = vec![
        Constraint::Length(1),                       // 0: title
        Constraint::Length(1),                       // 1: subtitle
        Constraint::Length(sys_panel_height as u16), // 2: sys panel
        Constraint::Length(1),                       // 3: spacing
        Constraint::Min(3),                          // 4: actions panel
    ];

    if has_status {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(1));
    }

    let sep_idx = constraints.len();
    constraints.push(Constraint::Length(1));
    let footer_idx = constraints.len();
    constraints.push(Constraint::Length(1));

    let chunks = Layout::vertical(constraints).split(inner);

    // Title line (centered, no border)
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "admin-toolkit",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v1.4.0", Style::default().fg(theme.fg_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "alterações em lote do sistema",
        Style::default().fg(theme.fg_dim),
    )));
    frame.render_widget(subtitle, chunks[1]);

    // System info panel
    let sys_panel = panel("Sistema", theme);
    frame.render_widget(&sys_panel, chunks[2]);
    let sys_inner = sys_panel.inner(chunks[2]);
    render_system_content(frame, sys_inner, state, theme);

    // Actions panel
    let act_panel = panel("Ações", theme);
    frame.render_widget(&act_panel, chunks[4]);
    let act_inner = act_panel.inner(chunks[4]);
    render_actions_content(frame, act_inner, state, theme);

    // Status row
    if has_status {
        let status_sep = sep_idx - 2;
        let status_area = sep_idx - 1;
        thin_sep(frame, chunks[status_sep], theme);
        let status_line = Paragraph::new(Line::from(Span::styled(
            &state.status,
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(status_line, chunks[status_area]);
    }

    heavy_sep(frame, chunks[sep_idx], theme);

    let footer = Paragraph::new(Line::from(Span::styled(
        "↑↓ navegar  espaço alternar  e editar  enter confirmar  esc voltar  q sair",
        Style::default().fg(theme.fg_dim),
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

fn render_system_content(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let w_col = 12usize;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "Atual", w_col = w_col),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Destino",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "─".repeat(area.width.saturating_sub(1) as usize),
            Style::default().fg(theme.border_panel),
        )),
    ];

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "hostname", w_col = w_col),
            Style::default().fg(theme.fg_dim),
        ),
        Span::styled(
            &state.snapshot.hostname,
            Style::default().fg(theme.fg_main),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "domínio", w_col = w_col),
            Style::default().fg(theme.fg_dim),
        ),
        Span::styled(&state.snapshot.domain, Style::default().fg(theme.fg_main)),
        Span::raw("  "),
        Span::styled(DOMAIN_TARGET, Style::default().fg(theme.accent)),
    ]));

    let (admin_label, admin_style) = if state.snapshot.elevated {
        ("Sim", Style::default().fg(theme.status_success))
    } else {
        ("Não", Style::default().fg(theme.status_error))
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<w_col$}", "admin", w_col = w_col),
            Style::default().fg(theme.fg_dim),
        ),
        Span::styled(admin_label, admin_style.add_modifier(Modifier::BOLD)),
    ]));

    if state.hostname_enabled && !state.hostname_target.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "novo hostname", w_col = w_col),
                Style::default().fg(theme.fg_dim),
            ),
            Span::styled(
                &state.hostname_target,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if state.create_user_enabled && !state.create_user_username.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<w_col$}", "novo usuário", w_col = w_col),
                Style::default().fg(theme.fg_dim),
            ),
            Span::styled(
                &state.create_user_username,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_actions_content(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let mut lines = Vec::new();

    let rows = [
        (
            "Alterar nome do computador",
            state.hostname_enabled,
            matches!(state.focus, Focus::Hostname),
            state.hostname_target.as_str(),
            !state.hostname_target.is_empty(),
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
            !state.password_value.is_empty(),
        ),
        (
            "Alterar domínio para itu.local",
            state.domain_enabled,
            matches!(state.focus, Focus::Domain),
            state.domain_target.as_str(),
            !state.domain_target.is_empty(),
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
            !state.create_user_username.is_empty(),
        ),
    ];

    for (label, enabled, focused, note, has_value) in &rows {
        let marker = if *enabled { "[x]" } else { "[ ]" };
        let marker_style = if *enabled {
            Style::default().fg(theme.status_success)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        let base_style = if *focused {
            Style::default()
                .fg(theme.invert_fg)
                .bg(theme.invert_bg)
                .add_modifier(Modifier::BOLD)
        } else if *enabled {
            Style::default().fg(theme.fg_main)
        } else {
            Style::default().fg(theme.fg_dim)
        };

        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::raw(" "),
            Span::styled(*label, base_style),
        ];

        if *enabled && !note.is_empty() {
            let note_style = if *focused {
                Style::default().fg(theme.fg_dim)
            } else if *has_value {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.status_warning)
            };
            spans.push(Span::styled(
                format!("  [{}]", note),
                note_style.add_modifier(Modifier::ITALIC),
            ));
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

// ── Input overlay ──

fn render_input(frame: &mut Frame<'_>, _kind: InputKind, form: Option<&Form>, theme: &Theme) {
    let area = frame.area();

    let outer = outer_frame(theme);
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner);
    let form_area = centered_rect(70, 20, chunks[0]);

    if let Some(form) = form {
        form.render(form_area, frame.buffer_mut());
    }

    let help = Paragraph::new(Line::from(Span::styled(
        "esc cancelar  enter confirmar",
        Style::default().fg(theme.fg_dim),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}

// ── Confirm screen ──

fn styled_summary_line(line: &str, theme: &Theme) -> Line<'static> {
    if let Some((label, value)) = line.split_once(": ") {
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{}:", label),
                Style::default().fg(theme.fg_dim),
            ),
            Span::raw(" "),
            Span::styled(
                value.to_string(),
                Style::default()
                    .fg(theme.fg_main)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![Span::styled(
            format!("  {}", line),
            Style::default().fg(theme.fg_main),
        )])
    }
}

fn render_confirm(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let outer = outer_frame(theme);
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(1), // subtitle
        Constraint::Min(2),    // summary
        Constraint::Length(1), // sep
        Constraint::Length(2), // warnings
        Constraint::Length(1), // footer sep
        Constraint::Length(1), // footer
    ])
    .split(inner);

    // Title line (centered, no border)
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "admin-toolkit",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v1.4.0", Style::default().fg(theme.fg_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Subtitle
    let subtitle = Paragraph::new(Line::from(Span::styled(
        "alterações em lote do sistema",
        Style::default().fg(theme.fg_dim),
    )));
    frame.render_widget(subtitle, chunks[1]);

    let lines: Vec<Line> = state
        .summary_lines()
        .into_iter()
        .map(|s| styled_summary_line(&s, theme))
        .collect();
    frame.render_widget(Paragraph::new(lines), chunks[2]);

    let warn = state.warnings();
    if !warn.is_empty() {
        let warn_lines: Vec<Line> = warn
            .iter()
            .map(|w| {
                Line::from(Span::styled(
                    format!("  ⚠ {}", w),
                    Style::default()
                        .fg(theme.status_warning)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect();
        frame.render_widget(Paragraph::new(warn_lines), chunks[4]);
    }

    heavy_sep(frame, chunks[4], theme);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter aplicar  esc voltar",
        Style::default().fg(theme.fg_dim),
    )));
    frame.render_widget(help, chunks[6]);
}

// ── Blocked screen ──

fn render_blocked(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let outer = outer_frame(theme);
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let lines = vec![
        Line::from(Span::styled(
            "Privilégios de administrador necessários",
            Style::default()
                .fg(theme.status_error)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &state.blocked_reason,
            Style::default().fg(theme.fg_main),
        )),
        Line::from(""),
        Line::from(Span::styled("q sair", Style::default().fg(theme.fg_dim))),
    ];

    let p = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(p, inner);
}

// ── Result screen ──

fn render_result(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let outer = outer_frame(theme);
    frame.render_widget(&outer, area);
    let inner = outer.inner(area);

    let chunks = Layout::vertical([
        Constraint::Min(2),
        Constraint::Length(1), // sep
        Constraint::Length(1), // footer
    ])
    .split(inner);

    let success = state.reboot_required
        || state.result_message.contains("sucesso")
        || state.result_message.contains("Applied");

    let mut lines = vec![Line::from(Span::styled(
        &state.result_message,
        Style::default().fg(if success {
            theme.status_success
        } else {
            theme.status_error
        }),
    ))];
    if state.reboot_required {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "⚠ Reinicialização necessária.",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines), chunks[0]);

    heavy_sep(frame, chunks[1], theme);

    let help = Paragraph::new(Line::from(Span::styled(
        "enter voltar  q sair",
        Style::default().fg(theme.fg_dim),
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
