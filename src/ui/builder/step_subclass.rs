use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(_app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let title = Paragraph::new(" Character Builder: Step 3 - Subclass ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));

    // Placeholder rendering
    let msg = Paragraph::new("This step is a placeholder for subclass selection.").block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Subclass Selection "),
    );

    let outer = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

    frame.render_widget(title, outer[0]);
    frame.render_widget(msg, outer[1]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Class;
        }
        KeyCode::Enter => {
            app.builder.step = CharacterCreationStep::Abilities;
        }
        _ => {}
    }
}
