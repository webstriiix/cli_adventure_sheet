use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crate::models::compendium::source_id_label;
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

    // 1) Layout: Title, Body, Help
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 2 - Race ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // 2) Body Layout: List (Left), Details (Right)
    let body = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[1]);

    // 3) Render List
    let items: Vec<ListItem> = app
        .races
        .iter()
        .map(|r| {
            ListItem::new(Line::from(vec![
                Span::raw(r.name.clone()),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(r.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Race "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // 4) Render Selected Race Details
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(race) = app.races.get(idx) {
            let mut lines = Vec::new();

            lines.push(Line::from(vec![
                Span::styled(
                    race.name.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(race.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(""));

            // Size & Speed
            lines.push(Line::from(vec![
                Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(race.size.join(", ")),
            ]));

            let speed_str = if let Some(spd) = race.speed.as_object() {
                spd.get("walk")
                    .and_then(|v| v.as_i64())
                    .map(|v| format!("{} ft.", v))
                    .unwrap_or_else(|| "Unknown".to_string())
            } else if let Some(spd) = race.speed.as_i64() {
                format!("{} ft.", spd)
            } else {
                "Unknown".to_string()
            };
            lines.push(Line::from(vec![
                Span::styled("Speed: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(speed_str),
            ]));
            lines.push(Line::from(""));

            // Bonuses
            lines.push(Line::from(Span::styled(
                "Ability Bonuses:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for bonus in &race.ability_bonuses {
                if let Some(obj) = bonus.as_object() {
                    for (k, v) in obj {
                        if let Some(val) = v.as_i64() {
                            lines.push(Line::from(format!("  • +{} {}", val, k.to_uppercase())));
                        }
                    }
                }
            }
            lines.push(Line::from(""));

            // Traits
            if !race.trait_tags.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Traits:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                let traits = race.trait_tags.join(", ");
                lines.push(Line::from(format!("  {}", traits)));
            }

            let detail_p = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(" Traits "))
                .wrap(Wrap { trim: true });
            frame.render_widget(detail_p, body[1]);
        }
    }

    // 5) Help Footer
    let help = Paragraph::new("↑↓ select   Enter confirm   Esc back to method")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.screen = crate::models::app_state::Screen::CharacterList;
            app.builder = crate::models::app_state::BuilderState::default();
        }
        KeyCode::Up => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i > 0 {
                        i - 1
                    } else {
                        app.races.len().saturating_sub(1)
                    }
                }
                None => 0,
            };
            app.builder.list_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i + 1 < app.races.len() {
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
                if let Some(race) = app.races.get(idx) {
                    app.builder.race_id = Some(race.id);
                    app.builder.bonus_feat_id = None;
                    app.builder.race_skill_choice = None;
                    app.builder.feat_picker_index = 0;
                    if race.grants_bonus_feat {
                        // Human XPHB: go to Skillful step first, then Versatile feat
                        app.builder.step = CharacterCreationStep::RaceSkill;
                    } else {
                        app.builder.step = CharacterCreationStep::Class;
                        app.builder.list_state.select(Some(0));
                    }
                    app.status_msg.clear();
                }
            }
        }
        _ => {}
    }
}
