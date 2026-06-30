use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    // ── Left: Species + Subrace nested list ──
    let items: Vec<ListItem> = app.races.iter().flat_map(|race| {
        let is_cur = Some(race.id) == app.builder.race_id;
        let name_style = if is_cur {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let src = crate::models::compendium::source_id_label(race.source_id);
        let race_item = ListItem::new(Line::from(vec![
            Span::styled(race.name.clone(), name_style),
            Span::styled(format!(" [{}]", src), Style::default().fg(Color::DarkGray)),
        ]));

        // If this race is expanded, show subraces underneath
        let sub_items: Vec<ListItem> = if is_cur {
            app.subraces.iter()
                .filter(|sr| sr.race_id == race.id)
                .map(|sr| {
                    let is_sel = Some(sr.id) == app.builder.subrace_id;
                    let sub_style = if is_sel {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled("  ├─ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(sr.name.clone(), sub_style),
                    ]))
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut all = vec![race_item];
        all.extend(sub_items);
        all
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Species ").border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // ── Right: Info panel ──
    if let Some(race_id) = app.builder.race_id {
        if let Some(race) = app.races.iter().find(|r| r.id == race_id).cloned() {
            let right = Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).split(body[1]);

            let mut info_lines = vec![
                Line::from(Span::styled(race.name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];

            if !race.size.is_empty() {
                info_lines.push(Line::from(vec![
                    Span::styled("Size:    ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(race.size.join(", ")),
                ]));
            }

            // Speed: could be integer or object
            let speed_str = match &race.speed {
                serde_json::Value::Number(n) => format!("{} ft.", n),
                serde_json::Value::Object(map) => {
                    map.iter().map(|(k, v)| format!("{}: {} ft.", k, v)).collect::<Vec<_>>().join(", ")
                }
                _ => String::from("—"),
            };
            info_lines.push(Line::from(vec![
                Span::styled("Speed:   ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(speed_str),
            ]));

            // Subrace detail
            if let Some(subrace_id) = app.builder.subrace_id {
                if let Some(sr) = app.subraces.iter().find(|s| s.id == subrace_id) {
                    info_lines.push(Line::from(""));
                    info_lines.push(Line::from(vec![
                        Span::styled("Lineage: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(sr.name.clone(), Style::default().fg(Color::Yellow)),
                    ]));
                }
            }

            let info_p = Paragraph::new(info_lines)
                .block(Block::default().borders(Borders::ALL).title(" Species Info ").border_style(Style::default().fg(Color::DarkGray)))
                .wrap(Wrap { trim: true });
            frame.render_widget(info_p, right[0]);

            // Trait tags panel
            let tags: Vec<Line> = race.trait_tags.iter().map(|t| {
                Line::from(vec![Span::styled(format!("  • {}", t), Style::default().fg(Color::Gray))])
            }).collect();

            let traits_panel = Paragraph::new(if tags.is_empty() {
                vec![Line::from(Span::styled("No trait tags listed.", Style::default().fg(Color::DarkGray)))]
            } else { tags })
            .block(Block::default().borders(Borders::ALL).title(" Racial Traits ").border_style(Style::default().fg(Color::DarkGray)))
            .wrap(Wrap { trim: true });
            frame.render_widget(traits_panel, right[1]);
        }
    } else {
        let placeholder = Paragraph::new("← Select a species to preview traits")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(placeholder, body[1]);
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if app.builder.subrace_id.is_some() {
                app.builder.subrace_id = None;
            } else {
                app.builder.step = CharacterCreationStep::Background;
                app.builder.list_state.select(Some(0));
                app.status_msg.clear();
            }
        }
        KeyCode::Up => {
            let cur = app.builder.list_state.selected().unwrap_or(0);
            app.builder.list_state.select(Some(cur.saturating_sub(1)));
            update_selection_from_list(app);
        }
        KeyCode::Down => {
            let total = count_list_items(app);
            let cur = app.builder.list_state.selected().unwrap_or(0);
            let next = if cur + 1 < total { cur + 1 } else { total.saturating_sub(1) };
            app.builder.list_state.select(Some(next));
            update_selection_from_list(app);
        }
        KeyCode::Enter => {
            if app.builder.race_id.is_some() {
                let has_subraces = app.builder.race_id
                    .map(|id| app.subraces.iter().any(|sr| sr.race_id == id))
                    .unwrap_or(false);

                if has_subraces && app.builder.subrace_id.is_none() {
                    app.status_msg = "Please select a Lineage (use ↑↓ to expand, then ↓ to sub-options).".to_string();
                    return;
                }

                if !app.save_draft() { return; }
                app.builder.step = CharacterCreationStep::Abilities;
                app.builder.list_state.select(Some(0));
                app.status_msg.clear();
            }
        }
        _ => {}
    }
}

fn count_list_items(app: &App) -> usize {
    app.races.iter().fold(0usize, |acc, race| {
        let sub_count = if Some(race.id) == app.builder.race_id {
            app.subraces.iter().filter(|sr| sr.race_id == race.id).count()
        } else { 0 };
        acc + 1 + sub_count
    })
}

fn update_selection_from_list(app: &mut App) {
    let target = app.builder.list_state.selected().unwrap_or(0);
    let mut idx = 0usize;
    let races = app.races.clone();
    for race in &races {
        if idx == target {
            if app.builder.race_id != Some(race.id) {
                app.builder.race_id = Some(race.id);
                app.builder.subrace_id = None;
            }
            return;
        }
        idx += 1;

        if Some(race.id) == app.builder.race_id {
            let subraces: Vec<_> = app.subraces.iter()
                .filter(|sr| sr.race_id == race.id)
                .cloned()
                .collect();
            for sr in &subraces {
                if idx == target {
                    app.builder.subrace_id = Some(sr.id);
                    return;
                }
                idx += 1;
            }
        }
    }
}
