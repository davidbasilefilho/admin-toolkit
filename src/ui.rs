use crate::state::{AppState, DOMAIN_TARGET, Focus, InputKind, PREFEITURA_USER, Screen, mask_text};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render(frame: &mut Frame<'_>, state: &AppState) {
    match state.screen {
        Screen::Edit => render_edit(frame, state),
        Screen::Input(kind) => render_input(frame, state, kind),
        Screen::Confirm => render_confirm(frame, state),
        Screen::Blocked => render_blocked(frame, state),
        Screen::Result => render_result(frame, state),
    }
}

fn render_edit(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_snapshot(frame, chunks[0], state);
    render_actions(frame, chunks[1], state);
    render_status(frame, chunks[2], &state.status);
    render_footer(
        frame,
        chunks[3],
        "↑↓ move  space toggle  e edit target  enter confirm  q quit",
    );
}

fn render_input(frame: &mut Frame<'_>, state: &AppState, kind: InputKind) {
    let title = match kind {
        InputKind::Hostname => String::from("Hostname target"),
        InputKind::Password => format!("Password target for {}", PREFEITURA_USER),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_snapshot(frame, chunks[0], state);

    let content = match kind {
        InputKind::Hostname => state.input_buffer.clone(),
        InputKind::Password => mask_text(&state.input_buffer),
    };

    let input = Paragraph::new(vec![
        Line::from(title),
        Line::from(Span::styled(content, Style::default().fg(Color::Yellow))),
    ])
    .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input, chunks[1]);

    render_status(frame, chunks[2], "Enter save  esc cancel  backspace delete");
}

fn render_confirm(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_snapshot(frame, chunks[0], state);

    let summary = state
        .summary_lines()
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();

    let confirm =
        Paragraph::new(summary).block(Block::default().borders(Borders::ALL).title("Confirm"));
    frame.render_widget(confirm, chunks[1]);

    let warnings = if state.warnings().is_empty() {
        String::from("No reboot warning.")
    } else {
        state.warnings().join(" ")
    };

    render_status(frame, chunks[2], &warnings);
    render_footer(frame, chunks[3], "enter apply  esc back");
}

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(72, 36, frame.area());
    let block = Paragraph::new(vec![
        Line::from("Elevation required"),
        Line::from(Span::styled(
            state.blocked_reason.as_str(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Blocked"));
    frame.render_widget(block, area);
}

fn render_result(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_snapshot(frame, chunks[0], state);

    let mut lines = vec![Line::from(state.result_message.as_str())];
    if state.reboot_required {
        lines.push(Line::from(Span::styled(
            "Reboot required.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let result =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Result"));
    frame.render_widget(result, chunks[1]);
    render_footer(frame, chunks[2], "enter return to edit  q quit");
}

fn render_snapshot(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let snapshot = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(state.snapshot.hostname.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Domain: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(state.snapshot.domain.as_str()),
        ]),
        Line::from(vec![
            Span::styled(
                "Domain target: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(DOMAIN_TARGET),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current state"),
    );
    frame.render_widget(snapshot, area);
}

fn render_actions(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let items = vec![
        action_item(
            "Change hostname",
            state.hostname_enabled,
            matches!(state.focus, Focus::Hostname),
            if state.hostname_target.is_empty() {
                Some("needs target")
            } else {
                Some(state.hostname_target.as_str())
            },
        ),
        action_item(
            "Change password for Prefeitura",
            state.password_enabled,
            matches!(state.focus, Focus::Password),
            if state.password_value.is_empty() {
                Some("needs password")
            } else {
                Some("password set")
            },
        ),
        action_item(
            "Change domain to itu.local",
            state.domain_enabled,
            matches!(state.focus, Focus::Domain),
            Some("fixed target"),
        ),
    ];

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Actions"));
    frame.render_widget(list, area);
}

fn action_item(label: &str, enabled: bool, focused: bool, note: Option<&str>) -> ListItem<'static> {
    let marker = if enabled { "[x]" } else { "[ ]" };
    let mut spans = vec![
        Span::raw(marker),
        Span::raw(" "),
        Span::raw(label.to_string()),
    ];

    if let Some(note) = note {
        spans.push(Span::raw(" - "));
        spans.push(Span::styled(
            note.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    let mut item = ListItem::new(Line::from(spans));
    if focused {
        item = item.style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    }
    item
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let status = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let footer = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
