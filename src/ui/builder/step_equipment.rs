use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // 1) Layout
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 6 - Equipment Options ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // 2) Body
    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    // Left List
    let items = vec![
        ListItem::new(" Option A: Class & Background Packages "),
        ListItem::new(" Option B: Starting Gold (Buy your own) "),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Starting Gear Flow "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // Right Details
    let mut lines = Vec::new();

    if let Some(idx) = app.builder.list_state.selected() {
        if idx == 0 {
            lines.push(Line::from(vec![Span::styled(
                "Standard Equipment Packages",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from("Receive the standard starting gear provided by your selected Class and Background."));
            lines.push(Line::from("Your inventory will be pre-populated with common adventuring items like armor, sidearms, and dungeoneer's packs."));
            lines.push(Line::from(""));

            // Display some snippets from chosen class/bg
            lines.push(Line::from(Span::styled(
                "From Class:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            if let Some(c_id) = app.builder.class_id {
                if let Some(_class) = app.classes.iter().find(|c| c.id == c_id) {
                    // Check class starting_equipment json
                    lines.push(Line::from("  Includes core weapons, armor, and packs."));
                }
            } else {
                lines.push(Line::from("  (No class selected)"));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "From Background:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            if let Some(b_id) = app.builder.bg_id {
                if let Some(_bg) = app.backgrounds.iter().find(|b| b.id == b_id) {
                    lines.push(Line::from(
                        "  Includes flavor items, tools, and a pouch of gold.",
                    ));
                }
            } else {
                lines.push(Line::from("  (No background selected)"));
            }
        } else {
            lines.push(Line::from(vec![Span::styled(
                "Starting Gold",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Forgo the standard equipment packages and start with a lump sum of gold.",
            ));
            lines.push(Line::from("You can buy your own custom weapons, armor, and gear in the character sheet later."));
            lines.push(Line::from(""));
            lines.push(Line::from(
                "Amount ranges from 2d4 x 10gp to 5d4 x 10gp depending on class.",
            ));
        }
    }

    let detail_p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Option Details "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(detail_p, body[1]);

    // 3) Footer
    let help = Paragraph::new("↑↓ select   Enter confirm   Esc back to proficiencies")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Proficiencies;
            app.builder.ability_focus = 0;
            app.builder.list_state.select(Some(0)); // Reset inner state if needed
        }
        KeyCode::Up => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i > 0 {
                        i - 1
                    } else {
                        1
                    }
                }
                None => 0,
            };
            app.builder.list_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i < 1 {
                        i + 1
                    } else {
                        0
                    }
                }
                None => 0,
            };
            app.builder.list_state.select(Some(i));
        }
        KeyCode::Enter => {
            if let Some(idx) = app.builder.list_state.selected() {
                app.builder.equipment_option = Some(idx);
                
                // Move to Spells if caster, otherwise Details
                let mut is_caster = app.builder.spellcasting_type != "none";
                
                // Backup check: if spellcasting_type is none, check the actual class data
                if !is_caster {
                    if let Some(c_id) = app.builder.class_id {
                        if let Some(class) = app.classes.iter().find(|c| c.id == c_id) {
                            if class.spellcasting_ability.is_some() {
                                is_caster = true;
                            }
                        }
                    }
                }

                if is_caster {
                    app.builder.step = CharacterCreationStep::Spells;
                } else {
                    app.builder.step = CharacterCreationStep::Details;
                }
                app.builder.focus_index = 0;
            }
        }
        _ => {}
    }
}
