use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // title
        Constraint::Min(5),    // list
        Constraint::Length(2), // help + status
    ])
    .split(area);

    let title = Paragraph::new(" Your Characters ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, chunks[0]);

    if app.characters.is_empty() {
        let empty = Paragraph::new("  No characters yet. Press N to create one.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            );
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .characters
            .iter()
            .map(|ch| {
                let level = crate::utils::level_from_xp(ch.experience_pts);
                let hp_color = if ch.current_hp <= ch.max_hp / 4 {
                    Color::Red
                } else if ch.current_hp <= ch.max_hp / 2 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let line = Line::from(vec![
                    Span::styled(ch.name.clone(), Style::default().fg(Color::White)),
                    Span::styled(
                        format!("  Lvl {level}"),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("  HP {}/{}", ch.current_hp, ch.max_hp),
                        Style::default().fg(hp_color),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        app.char_list_state.select(Some(app.selected_char));
        frame.render_stateful_widget(list, chunks[1], &mut app.char_list_state);
    }

    let mut help_lines = vec![Line::from(vec![
        Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" navigate  "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" view  "),
        Span::styled("E", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" edit  "),
        Span::styled("D", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" delete  "),
        Span::styled("N", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" new  "),
        Span::styled("R", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" refresh  "),
        Span::styled("L", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" logout  "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" quit"),
    ])];

    if !app.status_msg.is_empty() {
        help_lines.push(Line::from(Span::styled(
            app.status_msg.as_str(),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let help = Paragraph::new(help_lines).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);

    // Delete confirmation popup
    if app.delete_confirm {
        let char_name = app
            .characters
            .get(app.selected_char)
            .map(|c| c.name.as_str())
            .unwrap_or("this character");

        let popup_area = centered_popup(area, 50, 5);
        let msg = format!(" Delete \"{}\"? (Y to confirm, any other key cancels) ", char_name);
        let popup = Paragraph::new(msg)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(" Confirm Delete ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            );
        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
    }
}

fn centered_popup(area: ratatui::layout::Rect, width_pct: u16, height: u16) -> ratatui::layout::Rect {
    let popup_w = area.width * width_pct / 100;
    let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(height)) / 2;
    ratatui::layout::Rect::new(popup_x, popup_y, popup_w, height)
}
