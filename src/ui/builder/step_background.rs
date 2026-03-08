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

    // 1) Layout
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 4 - Background ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // 2) Body
    let body = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[1]);

    // 3) List
    let items: Vec<ListItem> = app
        .backgrounds
        .iter()
        .map(|b| {
            ListItem::new(Line::from(vec![
                Span::raw(b.name.clone()),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(b.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Background "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // 4) Details
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(bg) = app.backgrounds.get(idx) {
            let mut lines = Vec::new();

            lines.push(Line::from(vec![
                Span::styled(
                    bg.name.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(bg.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(""));

            // Skills
            if let Some(skills) = &bg.skill_proficiencies {
                let mut skill_names = Vec::new();
                for sk in skills {
                    if let Some(obj) = sk.as_object() {
                        for k in obj.keys() {
                            skill_names.push(k.clone());
                        }
                    } else if let Some(s) = sk.as_str() {
                        skill_names.push(s.to_string());
                    }
                }
                if !skill_names.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Skill Proficiencies: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(skill_names.join(", ")),
                    ]));
                }
            }

            // Tools
            if let Some(tools) = &bg.tool_proficiencies {
                let mut tool_names = Vec::new();
                for t in tools {
                    if let Some(obj) = t.as_object() {
                        for k in obj.keys() {
                            tool_names.push(k.clone());
                        }
                    } else if let Some(s) = t.as_str() {
                        tool_names.push(s.to_string());
                    }
                }
                if !tool_names.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Tool Proficiencies: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(tool_names.join(", ")),
                    ]));
                }
            }

            // Languages
            if let Some(lang_count) = bg.language_count {
                if lang_count > 0 {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Bonus Languages: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("Choose {} additional language(s)", lang_count)),
                    ]));
                }
            }
            lines.push(Line::from(""));

            // Note
            lines.push(Line::from(Span::styled(
                "Background Feature:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(
                "Provides a unique roleplay feature and starting gear.",
            ));

            let detail_p = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Background Details "),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(detail_p, body[1]);
        }
    }

    // 5) Help
    let help = Paragraph::new("↑↓ select   Enter confirm   Esc back to abilities")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Abilities;
            app.builder.list_state.select(Some(0));
        }
        KeyCode::Up => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i > 0 {
                        i - 1
                    } else {
                        app.backgrounds.len().saturating_sub(1)
                    }
                }
                None => 0,
            };
            app.builder.list_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i + 1 < app.backgrounds.len() {
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
                if let Some(bg) = app.backgrounds.get(idx) {
                    app.builder.bg_id = Some(bg.id);
                    app.builder.background_feat_id = None;
                    app.builder.feat_picker_search.clear();
                    app.builder.feat_picker_index = 0;
                    app.builder.language_count = bg.language_count.unwrap_or(0);

                    // Reset bg ability state for new selection
                    app.builder.bg_ability_bonuses = [0; 6];
                    app.builder.bg_ability_step = 0;
                    app.builder.bg_ability_focus = 0;

                    let has_ability_choose = bg
                        .ability_bonuses
                        .as_ref()
                        .and_then(|v| v.first())
                        .and_then(|v| v.pointer("/choose/weighted/from"))
                        .is_some();

                    if has_ability_choose {
                        app.builder.step = CharacterCreationStep::BackgroundAbilities;
                    } else if bg.grants_bonus_feat {
                        // XPHB backgrounds all grant a bonus Origin feat — pick it first
                        app.builder.step = CharacterCreationStep::BackgroundFeat;
                    } else if app.builder.language_count > 0 {
                        app.builder.step = CharacterCreationStep::Languages;
                    } else {
                        app.builder.step = CharacterCreationStep::Proficiencies;
                    }

                    app.builder.list_state.select(Some(0));
                    app.status_msg.clear();
                }
            }
        }
        _ => {}
    }
}
