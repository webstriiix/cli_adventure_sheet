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
#[derive(Debug, Clone, Copy, PartialEq)]
enum BgFocus {
    Name,
    Personality,
    BackgroundSearch,
    BackgroundList,
}

fn get_focus(app: &App) -> BgFocus {
    match app.builder.focus_index {
        0 => BgFocus::Name,
        1 => BgFocus::Personality,
        2 => BgFocus::BackgroundSearch,
        _ => BgFocus::BackgroundList,
    }
}

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let body =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    // ── Left: Form fields ──
    let form = Layout::vertical([
        Constraint::Length(3), // Character Name
        Constraint::Length(5), // Personality
        Constraint::Min(0),    // Feat status
    ])
    .margin(1)
    .split(body[0]);

    let focus = get_focus(app);

    // Name field
    let name_focused = focus == BgFocus::Name;
    let name_display = if name_focused {
        format!("{}█", app.builder.name)
    } else if app.builder.name.is_empty() {
        "⚠ Required".to_string()
    } else {
        app.builder.name.clone()
    };
    let name_border_style = if name_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let name_p = Paragraph::new(name_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Character Name * ")
                .border_style(name_border_style),
        )
        .style(if name_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        });
    frame.render_widget(name_p, form[0]);

    // Personality field
    let per_focused = focus == BgFocus::Personality;
    let per_display = if per_focused {
        format!("{}█", app.builder.trait_text)
    } else if app.builder.trait_text.is_empty() {
        "(personality trait, ideal, bond or flaw)".to_string()
    } else {
        app.builder.trait_text.clone()
    };
    let per_border_style = if per_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let per_p = Paragraph::new(per_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Personality ")
                .border_style(per_border_style),
        )
        .style(if per_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .wrap(Wrap { trim: true });
    frame.render_widget(per_p, form[1]);

    // Feat chosen indicator
    let feat_text = if let Some(feat_id) = app.builder.background_feat_id {
        let feat_name = app
            .all_feats
            .iter()
            .find(|f| f.id == feat_id)
            .map(|f| f.name.as_str())
            .unwrap_or("Unknown");
        format!("Origin Feat: {} ✓\nPress 'F' to change", feat_name)
    } else {
        let bg_grants_feat = app
            .builder
            .bg_id
            .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
            .map(|b| b.grants_bonus_feat)
            .unwrap_or(false);
        if bg_grants_feat {
            "Background grants an Origin Feat!\nPress 'F' to choose it.".to_string()
        } else {
            "No Origin Feat from this background.".to_string()
        }
    };
    let feat_p = Paragraph::new(feat_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Origin Feat ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
    frame.render_widget(feat_p, form[2]);

    // ── Right: Search + Background list ──
    let right_layout = Layout::vertical([
        Constraint::Length(3), // Search Box
        Constraint::Min(0),    // List
    ])
    .split(body[1]);

    let bg_search_focused = focus == BgFocus::BackgroundSearch;
    let bg_list_focused = focus == BgFocus::BackgroundList;

    let query = app.builder.background_search.to_lowercase();
    let filtered: Vec<&crate::models::compendium::Background> = app
        .backgrounds
        .iter()
        .filter(|bg| query.is_empty() || bg.name.to_lowercase().contains(&query))
        .collect();

    // Clamp and check selected background selection
    let selected_idx = app
        .builder
        .list_state
        .selected()
        .map(|i| i.min(filtered.len().saturating_sub(1)));
    if selected_idx != app.builder.list_state.selected() {
        app.builder.list_state.select(selected_idx);
    }

    // Search bar
    let search_border_style = if bg_search_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let search_text = if bg_search_focused {
        format!("{}█", app.builder.background_search)
    } else if app.builder.background_search.is_empty() {
        "(type to search...)".to_string()
    } else {
        app.builder.background_search.clone()
    };
    let search_p = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search Background (Tab) ")
                .border_style(search_border_style),
        )
        .style(if bg_search_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        });
    frame.render_widget(search_p, right_layout[0]);

    // Background list
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|bg| {
            let is_selected = Some(bg.id) == app.builder.bg_id;
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let source_lbl = crate::models::compendium::source_id_label(bg.source_id);
            let name_with_source = format!("{} ({})", bg.name, source_lbl);
            let mut spans = vec![Span::styled(name_with_source, name_style)];
            if bg.grants_bonus_feat {
                spans.push(Span::styled(" [+Feat]", Style::default().fg(Color::Yellow)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let bg_border = if bg_list_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Background (Tab to focus) ")
                .border_style(bg_border),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, right_layout[1], &mut app.builder.list_state);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let focus = get_focus(app);

    match key.code {
        KeyCode::Esc => {
            if focus == BgFocus::BackgroundSearch && !app.builder.background_search.is_empty() {
                app.builder.background_search.clear();
                app.builder.list_state.select(Some(0));
                let filtered: Vec<&crate::models::compendium::Background> = app
                    .backgrounds
                    .iter()
                    .filter(|bg| {
                        bg.name
                            .to_lowercase()
                            .contains(&app.builder.background_search.to_lowercase())
                    })
                    .collect();
                if let Some(bg) = filtered.first() {
                    app.builder.bg_id = Some(bg.id);
                }
            } else {
                app.builder.step = CharacterCreationStep::Class;
                app.builder.list_state.select(Some(0));
                app.builder.focus_index = 0;
                app.status_msg.clear();
            }
        }
        KeyCode::Tab => {
            app.builder.focus_index = (app.builder.focus_index + 1) % 4;
        }
        KeyCode::BackTab => {
            if app.builder.focus_index == 0 {
                app.builder.focus_index = 3;
            } else {
                app.builder.focus_index -= 1;
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            let bg_grants_feat = app
                .builder
                .bg_id
                .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
                .map(|b| b.grants_bonus_feat)
                .unwrap_or(false);
            if bg_grants_feat {
                app.builder.show_feat_modal = true;
                app.builder.feat_list_state.select(Some(0));
                app.builder.feat_picker_search.clear();
            } else {
                app.status_msg =
                    "Select a background that grants an Origin Feat first.".to_string();
            }
        }
        KeyCode::Enter => {
            match focus {
                BgFocus::Name => {
                    app.builder.focus_index = 1; // Move to personality
                }
                BgFocus::Personality => {
                    app.builder.focus_index = 2; // Move to background search
                }
                BgFocus::BackgroundSearch => {
                    app.builder.focus_index = 3; // Move to background list
                }
                BgFocus::BackgroundList => {
                    // Validate
                    if app.builder.name.trim().is_empty() {
                        app.status_msg = "Character Name is required!".to_string();
                        app.builder.focus_index = 0;
                        return;
                    }

                    // Check if selected background grants feat and we haven't chosen one yet
                    let bg_grants_feat = app
                        .builder
                        .bg_id
                        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
                        .map(|b| b.grants_bonus_feat)
                        .unwrap_or(false);
                    if bg_grants_feat && app.builder.background_feat_id.is_none() {
                        app.status_msg =
                            "This background grants an Origin Feat — press 'F' to choose it!"
                                .to_string();
                        app.builder.show_feat_modal = true;
                        app.builder.feat_list_state.select(Some(0));
                        return;
                    }

                    // Save draft and advance
                    if !app.save_draft() {
                        return;
                    }
                    app.builder.step = CharacterCreationStep::Species;
                    app.builder.list_state.select(Some(0));
                    app.builder.focus_index = 0;
                    app.status_msg.clear();
                }
            }
        }
        // Text input for Name, Personality and BackgroundSearch
        KeyCode::Backspace => match focus {
            BgFocus::Name => {
                app.builder.name.pop();
            }
            BgFocus::Personality => {
                app.builder.trait_text.pop();
            }
            BgFocus::BackgroundSearch => {
                app.builder.background_search.pop();
                app.builder.list_state.select(Some(0));
                let query = app.builder.background_search.to_lowercase();
                let filtered: Vec<&crate::models::compendium::Background> = app
                    .backgrounds
                    .iter()
                    .filter(|bg| query.is_empty() || bg.name.to_lowercase().contains(&query))
                    .collect();
                if let Some(bg) = filtered.first() {
                    app.builder.bg_id = Some(bg.id);
                } else {
                    app.builder.bg_id = None;
                }
            }
            BgFocus::BackgroundList => {}
        },
        KeyCode::Char(c) => match focus {
            BgFocus::Name => {
                app.builder.name.push(c);
            }
            BgFocus::Personality => {
                app.builder.trait_text.push(c);
            }
            BgFocus::BackgroundSearch => {
                app.builder.background_search.push(c);
                app.builder.list_state.select(Some(0));
                let query = app.builder.background_search.to_lowercase();
                let filtered: Vec<&crate::models::compendium::Background> = app
                    .backgrounds
                    .iter()
                    .filter(|bg| query.is_empty() || bg.name.to_lowercase().contains(&query))
                    .collect();
                if let Some(bg) = filtered.first() {
                    app.builder.bg_id = Some(bg.id);
                } else {
                    app.builder.bg_id = None;
                }
            }
            BgFocus::BackgroundList => {}
        },
        // Background list navigation (only when focused on list)
        KeyCode::Up => {
            if focus == BgFocus::BackgroundList {
                let query = app.builder.background_search.to_lowercase();
                let filtered: Vec<&crate::models::compendium::Background> = app
                    .backgrounds
                    .iter()
                    .filter(|bg| query.is_empty() || bg.name.to_lowercase().contains(&query))
                    .collect();
                let len = filtered.len();
                if len > 0 {
                    let i = app
                        .builder
                        .list_state
                        .selected()
                        .map(|i| if i > 0 { i - 1 } else { len.saturating_sub(1) })
                        .unwrap_or(0);
                    app.builder.list_state.select(Some(i));
                    // Auto-select on hover
                    if let Some(bg) = filtered.get(i) {
                        app.builder.bg_id = Some(bg.id);
                    }
                }
            }
        }
        KeyCode::Down => {
            if focus == BgFocus::BackgroundList {
                let query = app.builder.background_search.to_lowercase();
                let filtered: Vec<&crate::models::compendium::Background> = app
                    .backgrounds
                    .iter()
                    .filter(|bg| query.is_empty() || bg.name.to_lowercase().contains(&query))
                    .collect();
                let len = filtered.len();
                if len > 0 {
                    let i = app
                        .builder
                        .list_state
                        .selected()
                        .map(|i| if i + 1 < len { i + 1 } else { 0 })
                        .unwrap_or(0);
                    app.builder.list_state.select(Some(i));
                    if let Some(bg) = filtered.get(i) {
                        app.builder.bg_id = Some(bg.id);
                        // Reset feat if background changed
                        app.builder.background_feat_id = None;
                    }
                }
            }
        }
        _ => {}
    }
}
