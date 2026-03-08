use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::models::app_state::AuthMode;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Center a box
    let popup = centered_rect(50, 60, area);
    frame.render_widget(Clear, popup);

    let title = match app.auth_mode {
        AuthMode::Login => " Login ",
        AuthMode::Signup => " Sign Up ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let field_count = if app.auth_mode == AuthMode::Signup {
        3
    } else {
        2
    };

    // Layout: fields + spacing + help + status
    let mut constraints = Vec::new();
    for _ in 0..field_count {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Length(1)); // spacer
    constraints.push(Constraint::Length(2)); // help text
    constraints.push(Constraint::Min(1)); // status

    let chunks = Layout::vertical(constraints).split(inner);

    let fields: Vec<(&str, usize)> = if app.auth_mode == AuthMode::Signup {
        vec![("Username", 2), ("Email", 0), ("Password", 1)]
    } else {
        vec![("Email", 0), ("Password", 1)]
    };

    for (i, (label, field_idx)) in fields.iter().enumerate() {
        let is_focused = app.auth_focus == i;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::Gray
        };

        let display_text = if *label == "Password" {
            "*".repeat(app.auth_fields[*field_idx].len())
        } else {
            app.auth_fields[*field_idx].clone()
        };

        let cursor_text = if is_focused {
            format!("{display_text}_")
        } else {
            display_text
        };

        let input = Paragraph::new(cursor_text).block(
            Block::default()
                .title(*label)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
        frame.render_widget(input, chunks[i]);
    }

    // Help text
    let help_idx = field_count + 1;
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" next field  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" submit  "),
            Span::styled("F2", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" toggle login/signup"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ]),
    ])
    .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[help_idx]);

    // Status message
    let status_idx = field_count + 2;
    if !app.status_msg.is_empty() {
        let status = Paragraph::new(app.status_msg.as_str())
            .style(Style::default().fg(Color::Red));
        frame.render_widget(status, chunks[status_idx]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
