use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    if app.editing_notes {
        // Edit mode
        let chunks = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Length(1), // spacer
            Constraint::Min(3),    // text area
        ])
        .split(area);

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "  EDITING NOTES",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (Esc to save)", Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(header, chunks[0]);

        // Show buffer with cursor rendered at notes_cursor
        let cursor_pos = app.notes_cursor.min(app.notes_buffer.len());
        // Ensure we split at a valid char boundary to prevent panics
        let safe_pos = if app.notes_buffer.is_char_boundary(cursor_pos) {
            cursor_pos
        } else {
            let mut p = cursor_pos;
            while p > 0 && !app.notes_buffer.is_char_boundary(p) {
                p -= 1;
            }
            p
        };

        let left = &app.notes_buffer[..safe_pos];
        let right = &app.notes_buffer[safe_pos..];
        let display = format!("  {}▌{}", left, right);

        let text = Paragraph::new(display)
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((app.content_scroll as u16, 0));
        frame.render_widget(text, chunks[2]);
    } else {
        // View mode
        let chunks = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Length(1), // spacer
            Constraint::Min(3),    // content
        ])
        .split(area);

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "  NOTES",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            if !app.sidebar_focused {
                Span::styled("  (e to edit)", Style::default().fg(Color::DarkGray))
            } else {
                Span::raw("")
            },
        ]));
        frame.render_widget(header, chunks[0]);

        let text = match &character.notes {
            Some(notes) if !notes.is_empty() => {
                // Hide internal [SKILLS:...] tag from display
                let display_notes = if let Some(start) = notes.find("[SKILLS:") {
                    let end = notes[start..].find(']').map(|e| start + e + 1).unwrap_or(notes.len());
                    let cleaned = format!("{}{}", &notes[..start], &notes[end..]);
                    cleaned.trim().to_string()
                } else {
                    notes.clone()
                };
                Paragraph::new(format!("  {display_notes}"))
                    .style(Style::default().fg(Color::White))
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((app.content_scroll as u16, 0))
            }
            _ => Paragraph::new("  No notes.\n\n  Press 'e' to start writing notes.")
                .style(Style::default().fg(Color::DarkGray)),
        };
        frame.render_widget(text, chunks[2]);
    }
}
