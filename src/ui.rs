use crate::state::{AppState, Screen};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui_form::Form;

const BLUE: Color = Color::Rgb(96, 165, 250);
const CYAN: Color = Color::Rgb(34, 211, 238);
const GREEN: Color = Color::Rgb(74, 222, 128);
const YELLOW: Color = Color::Rgb(250, 204, 21);
const RED: Color = Color::Rgb(248, 113, 113);
const MAGENTA: Color = Color::Rgb(232, 121, 249);
const TEXT: Color = Color::Rgb(226, 232, 240);

pub fn render(frame: &mut Frame<'_>, state: &AppState, form: &Form) {
    match state.screen {
        Screen::Edit => render_edit(frame, state, form),
        Screen::Confirm => render_confirm(frame, state),
        Screen::Blocked => render_blocked(frame, state),
        Screen::Result => render_result(frame, state),
    }
}

fn render_edit(frame: &mut Frame<'_>, state: &AppState, form: &Form) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_snapshot(frame, chunks[0], state);
    form.render(chunks[1], frame.buffer_mut());
    render_status(frame, chunks[2], &state.status);
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
        Paragraph::new(summary).block(Block::default().borders(Borders::ALL).title("Confirmar"));
    frame.render_widget(confirm, chunks[1]);

    let warnings = {
        let warnings = state.warnings();
        if warnings.is_empty() {
            String::from("Sem aviso de reinicialização.")
        } else {
            warnings.join(" • ")
        }
    };

    render_status(frame, chunks[2], &warnings);
    render_footer(frame, chunks[3], "enter aplicar  esc voltar");
}

fn render_blocked(frame: &mut Frame<'_>, state: &AppState) {
    let area = centered_rect(72, 36, frame.area());
    let block = Paragraph::new(vec![
        Line::from(Span::styled(
            "Privilégios de administrador necessários",
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            state.blocked_reason.as_str(),
            Style::default().fg(TEXT),
        )),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(RED))
            .title(Span::styled(
                "Bloqueado",
                Style::default().fg(RED).add_modifier(Modifier::BOLD),
            )),
    );
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
            "Reinicialização necessária.",
            Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
        )));
    }

    let result = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(GREEN))
            .title(Span::styled(
                "Resultado",
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(result, chunks[1]);
    render_footer(frame, chunks[2], "enter voltar à edição  q sair");
}

fn render_snapshot(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    use crate::state::DOMAIN_TARGET;
    let snapshot = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "Nome do computador: ",
                Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
            ),
            Span::styled(state.snapshot.hostname.as_str(), Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "Domínio atual: ",
                Style::default().fg(MAGENTA).add_modifier(Modifier::BOLD),
            ),
            Span::styled(state.snapshot.domain.as_str(), Style::default().fg(TEXT)),
        ]),
        Line::from(vec![
            Span::styled(
                "Domínio de destino: ",
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                DOMAIN_TARGET,
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BLUE))
            .title(Span::styled(
                "Estado atual",
                Style::default().fg(BLUE).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(snapshot, area);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let status = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(YELLOW))
            .title(Span::styled(
                "Status",
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(status, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, text: &str) {
    let footer = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BLUE)),
    );
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
