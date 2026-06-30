use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::models::app_state::CharacterCreationStep;

pub mod step_abilities;
pub mod step_background;
pub mod step_class;
pub mod step_equipment;
pub mod step_race;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    // Stepper header
    let steps = ["1. Class", "2. Background", "3. Species", "4. Abilities", "5. Equipment"];
    let cur_idx = match app.builder.step {
        CharacterCreationStep::Class => 0,
        CharacterCreationStep::Background => 1,
        CharacterCreationStep::Species => 2,
        CharacterCreationStep::Abilities => 3,
        CharacterCreationStep::Equipment => 4,
    };

    let mut spans = Vec::new();
    for (i, label) in steps.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ── ", Style::default().fg(Color::DarkGray)));
        }
        if i == cur_idx {
            spans.push(Span::styled(
                format!("[{}]", label),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        } else if i < cur_idx {
            spans.push(Span::styled(
                format!("{}✓", label),
                Style::default().fg(Color::Green),
            ));
        } else {
            spans.push(Span::styled(
                label.to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    let stepper_p = Paragraph::new(Line::from(spans))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(stepper_p, outer[0]);

    // Step content
    match app.builder.step {
        CharacterCreationStep::Class => step_class::render(app, frame, outer[1]),
        CharacterCreationStep::Background => step_background::render(app, frame, outer[1]),
        CharacterCreationStep::Species => step_race::render(app, frame, outer[1]),
        CharacterCreationStep::Abilities => step_abilities::render(app, frame, outer[1]),
        CharacterCreationStep::Equipment => step_equipment::render(app, frame, outer[1]),
    }

    // Modals (rendered on top)
    if app.builder.show_subclass_modal {
        render_subclass_modal(app, frame, area);
    } else if app.builder.show_feat_modal {
        render_feat_modal(app, frame, area);
    }

    // Footer
    let help_text = if app.builder.show_subclass_modal {
        "↑↓ select subclass   Enter confirm   Esc close"
    } else if app.builder.show_feat_modal {
        "↑↓ select feat   Enter confirm   Esc close"
    } else {
        match app.builder.step {
            CharacterCreationStep::Class =>
                "↑↓ select class   +/- change level   Enter proceed   Esc back",
            CharacterCreationStep::Background =>
                "Tab navigate fields   ↑↓ select background   F feat   Enter proceed   Esc back",
            CharacterCreationStep::Species =>
                "↑↓ navigate   Enter select/expand   Esc back",
            CharacterCreationStep::Abilities =>
                "↑↓ navigate   +/- adjust scores   Enter proceed   Esc back",
            CharacterCreationStep::Equipment =>
                "←→ select option   Enter finish   Esc back",
        }
    };

    let mut footer_lines = vec![Line::from(Span::styled(help_text, Style::default().fg(Color::DarkGray)))];
    if !app.status_msg.is_empty() {
        footer_lines.push(Line::from(Span::styled(
            app.status_msg.as_str(),
            Style::default().fg(Color::LightYellow),
        )));
    }
    frame.render_widget(Paragraph::new(footer_lines), outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.builder.show_subclass_modal {
        handle_subclass_modal_key(app, key);
        return;
    }
    if app.builder.show_feat_modal {
        handle_feat_modal_key(app, key);
        return;
    }

    match app.builder.step {
        CharacterCreationStep::Class => step_class::handle_key(app, key),
        CharacterCreationStep::Background => step_background::handle_key(app, key),
        CharacterCreationStep::Species => step_race::handle_key(app, key),
        CharacterCreationStep::Abilities => step_abilities::handle_key(app, key),
        CharacterCreationStep::Equipment => step_equipment::handle_key(app, key),
    }
}

fn centered_popup(area: Rect, width_pct: u16, height: u16) -> Rect {
    let popup_w = area.width * width_pct / 100;
    let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(popup_x, popup_y, popup_w, height)
}

fn get_available_subclasses(app: &App) -> Vec<crate::models::compendium::Subclass> {
    if let Some(ref detail) = app.class_detail {
        detail.subclasses.iter().map(|sc| sc.subclass.clone()).collect()
    } else {
        Vec::new()
    }
}

fn render_subclass_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_popup(area, 65, 14);
    frame.render_widget(Clear, popup_area);

    let subclasses = get_available_subclasses(app);
    let class_name = app.builder.class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .and_then(|c| c.subclass_title.as_deref())
        .unwrap_or("Subclass");

    let items: Vec<ListItem> = subclasses.iter().map(|sc| {
        ListItem::new(Line::from(vec![
            Span::styled(sc.name.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({})", sc.short_name), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let title = format!(" Choose {} (Level {}) ", class_name, app.builder.level);
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, popup_area, &mut app.builder.subclass_list_state);
}

fn handle_subclass_modal_key(app: &mut App, key: KeyEvent) {
    let subclasses = get_available_subclasses(app);
    if subclasses.is_empty() {
        app.builder.show_subclass_modal = false;
        return;
    }
    match key.code {
        KeyCode::Esc => { app.builder.show_subclass_modal = false; }
        KeyCode::Up => {
            let cur = app.builder.subclass_list_state.selected().unwrap_or(0);
            let next = if cur > 0 { cur - 1 } else { subclasses.len() - 1 };
            app.builder.subclass_list_state.select(Some(next));
        }
        KeyCode::Down => {
            let cur = app.builder.subclass_list_state.selected().unwrap_or(0);
            let next = if cur + 1 < subclasses.len() { cur + 1 } else { 0 };
            app.builder.subclass_list_state.select(Some(next));
        }
        KeyCode::Enter => {
            let selected = app.builder.subclass_list_state.selected().unwrap_or(0);
            if let Some(sc) = subclasses.get(selected) {
                app.builder.subclass_id = Some(sc.id);
                app.status_msg = format!("Subclass '{}' selected.", sc.name);
            }
            app.builder.show_subclass_modal = false;
        }
        _ => {}
    }
}

fn get_origin_feats(app: &App) -> Vec<crate::models::Feat> {
    app.all_feats.iter().filter(|f| f.prerequisite.is_none()).cloned().collect()
}

fn render_feat_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_popup(area, 65, 16);
    frame.render_widget(Clear, popup_area);

    let feats = get_origin_feats(app);
    let search = app.builder.feat_picker_search.to_lowercase();
    let filtered: Vec<_> = feats.iter()
        .filter(|f| search.is_empty() || f.name.to_lowercase().contains(&search))
        .collect();

    let items: Vec<ListItem> = filtered.iter().map(|f| {
        ListItem::new(Line::from(Span::raw(format!("  {}", f.name))))
    }).collect();

    let title = format!(" Choose Origin Feat (Search: {}▌) ", app.builder.feat_picker_search);
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, popup_area, &mut app.builder.feat_list_state);
}

fn handle_feat_modal_key(app: &mut App, key: KeyEvent) {
    let feats = get_origin_feats(app);
    let search = app.builder.feat_picker_search.to_lowercase();
    let filtered: Vec<_> = feats.iter()
        .filter(|f| search.is_empty() || f.name.to_lowercase().contains(&search))
        .collect();

    match key.code {
        KeyCode::Esc => {
            app.builder.show_feat_modal = false;
            app.builder.feat_picker_search.clear();
        }
        KeyCode::Up => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur > 0 { cur - 1 } else { filtered.len().saturating_sub(1) };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Down => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur + 1 < filtered.len() { cur + 1 } else { 0 };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Enter => {
            let selected = app.builder.feat_list_state.selected().unwrap_or(0);
            if let Some(f) = filtered.get(selected) {
                app.builder.background_feat_id = Some(f.id);
                app.status_msg = format!("Origin Feat '{}' selected.", f.name);
            }
            app.builder.show_feat_modal = false;
            app.builder.feat_picker_search.clear();
        }
        KeyCode::Backspace => {
            app.builder.feat_picker_search.pop();
            app.builder.feat_list_state.select(Some(0));
        }
        KeyCode::Char(c) => {
            app.builder.feat_picker_search.push(c);
            app.builder.feat_list_state.select(Some(0));
        }
        _ => {}
    }
}
