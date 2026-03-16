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

    // 1) Layout: Title, Body, Help
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 3 - Class ")
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
        .classes
        .iter()
        .map(|c| {
            ListItem::new(Line::from(vec![
                Span::raw(c.name.clone()),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", c.source_slug),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Class "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // 4) Render Selected Class Details
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx) {
            let mut lines = Vec::new();

            lines.push(Line::from(vec![
                Span::styled(
                    class.name.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", class.source_slug),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(""));

            // Core features
            lines.push(Line::from(vec![
                Span::styled("Hit Die: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("d{}", class.hit_die)),
            ]));

            if let Some(saves) = &class.proficiency_saves {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Saving Throws: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(saves.join(", ")),
                ]));
            }

            if let Some(armor) = &class.armor_proficiencies {
                lines.push(Line::from(vec![
                    Span::styled("Armor: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(armor.join(", ")),
                ]));
            }

            if let Some(weapons) = &class.weapon_proficiencies {
                lines.push(Line::from(vec![
                    Span::styled("Weapons: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(weapons.join(", ")),
                ]));
            }

            lines.push(Line::from(""));

            // Spellcasting snippet
            if let Some(spell_ability) = &class.spellcasting_ability {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Spellcasting Ability: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(spell_ability),
                ]));
                if let Some(progression) = &class.caster_progression {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Progression: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(progression),
                    ]));
                }
                lines.push(Line::from(
                    "*(You will select spells later in the character sheet!)*",
                ));
                lines.push(Line::from(""));
            }

            // Subclass naming flavor
            if let Some(sub_title) = &class.subclass_title {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Subclass Title: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(sub_title),
                ]));
            }

            let detail_p = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Class Features "),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(detail_p, body[1]);
        }
    }

    // 5) Help Footer
    let help = Paragraph::new("↑↓ select   Enter confirm   Esc back to race")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Race;
            // Best effort to find their previous race selection
            let idx = app
                .builder
                .race_id
                .and_then(|id| app.races.iter().position(|r| r.id == id))
                .unwrap_or(0);
            app.builder.list_state.select(Some(idx));
        }
        KeyCode::Up => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i > 0 {
                        i - 1
                    } else {
                        app.classes.len().saturating_sub(1)
                    }
                }
                None => 0,
            };
            app.builder.list_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.builder.list_state.selected() {
                Some(i) => {
                    if i + 1 < app.classes.len() {
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
                if let Some(_class) = app.classes.get(idx) {
                    // Class selected, figure out routing and state
                    let selected_class = &app.classes[app.builder.list_state.selected().unwrap()];
                    app.builder.class_id = Some(selected_class.id);

                    // Default to none unless we see a caster progression
                    app.builder.spellcasting_type = "none".to_string();
                    if let Some(progression) = selected_class.caster_progression.as_ref() {
                        if !progression.is_empty() && progression != "none" {
                            app.builder.spellcasting_type = progression.clone();
                        }
                    }
                    
                    // Extra check: if backend has spellcasting_ability, it's a caster!
                    if app.builder.spellcasting_type == "none" && selected_class.spellcasting_ability.is_some() {
                        app.builder.spellcasting_type = "half".to_string(); // Assume half for Paladin/Ranger if unspecified
                    }

                    // Look up subclasses to evaluate lock level.
                    // This data comes from the API when we need it, but for wizard routing
                    // we can fetch it now.
                    app.builder.skip_subclass = true; // Default
                    let class_name = selected_class.name.clone();
                    let source_slug = selected_class.source_slug.clone();

                    let rt = app.rt.clone();
                    let client = app.client.clone();
                    let res = rt.block_on(async {
                        client.get_class_detail(&class_name, &source_slug).await
                    });

                    let mut details_opt = None;
                    if let Ok(details) = res {
                        details_opt = Some(details.clone());
                        if !details.subclasses.is_empty() {
                            if details.subclasses[0].subclass.unlock_level == 1 {
                                app.builder.skip_subclass = false;
                            }
                        }
                    }

                    // Check for Weapon Mastery feature at level 1 (2024 Paladin/Fighter etc.)
                    let mut weapon_mastery_feat = None;
                    
                    if let Some(details) = details_opt {
                        for feat in details.features {
                            if feat.level == 1 {
                                let interpreted = feat.interpret();
                                if let crate::models::features::Feature::WeaponMastery { choose } = interpreted {
                                    weapon_mastery_feat = Some(choose);
                                    break;
                                }
                            }
                        }
                    }

                    if let Some(choose) = weapon_mastery_feat {
                        app.builder.builder_pending_feature = Some(crate::models::features::Feature::WeaponMastery { choose });
                        app.builder.step = CharacterCreationStep::FeatWeaponMastery;
                        app.builder.feat_picker_index = 0;
                        app.builder.feat_picker_search.clear();
                    } else if app.builder.skip_subclass {
                        app.builder.step = CharacterCreationStep::Abilities;
                    } else {
                        app.builder.step = CharacterCreationStep::Subclass;
                    }

                    app.builder.list_state.select(Some(0));
                    app.builder.subclass_id = None; // reset on changing class
                    app.status_msg.clear();
                }
            }
        }
        _ => {}
    }
}
