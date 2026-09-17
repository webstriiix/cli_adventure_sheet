use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
    let steps = [
        "1. Class",
        "2. Background",
        "3. Species",
        "4. Abilities",
        "5. Equipment",
    ];
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
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
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
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
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
    if app.builder.show_skill_choice_modal {
        render_skill_choice_modal(app, frame, area);
    } else if app.builder.show_subclass_modal {
        render_subclass_modal(app, frame, area);
    } else if app.builder.show_feat_modal {
        render_feat_modal(app, frame, area);
    } else if app.builder.show_progression_asi_modal {
        render_progression_asi_modal(app, frame, area);
    } else if app.builder.show_progression_wm_modal {
        render_progression_wm_modal(app, frame, area);
    } else if app.builder.feature_detail_modal.is_some() {
        render_feature_detail_modal(app, frame, area);
    }

    // Footer
    let help_text = if app.builder.show_skill_choice_modal {
        "↑↓/JK navigate   Type to search   Enter pick   Backspace clear   Esc close"
    } else if app.builder.show_subclass_modal {
        "↑↓ select subclass   Enter confirm   Esc close"
    } else if app.builder.show_feat_modal {
        "↑↓ select feat   Enter confirm   Esc close"
    } else if app.builder.show_progression_asi_modal {
        use crate::models::app_state::AsiModalStage;
        match app.builder.asi_modal_stage {
            AsiModalStage::Mode => "↑↓ select mode   A/B or Enter choose   Esc close",
            AsiModalStage::Stats => {
                if app.builder.asi_stat_picker_open {
                    "↑↓ pick stat   Enter confirm   Esc close picker"
                } else {
                    "↑↓ switch slot   Space/Enter open picker   D clear   Enter submit   Esc back"
                }
            }
            AsiModalStage::Feat => "↑↓ navigate   Type to search   Enter confirm   Esc back",
        }
    } else if app.builder.show_progression_wm_modal {
        "↑↓ select weapon   Enter confirm   Esc close"
    } else if app.builder.feature_detail_modal.is_some() {
        "Esc / Enter close modal   ↑↓ scroll"
    } else {
        match app.builder.step {
            CharacterCreationStep::Class => {
                "←→ switch pane   ↑↓/JK navigate   +/- level   Enter select slot   Ctrl+K detail   Tab proceed"
            }
            CharacterCreationStep::Background => {
                "Tab navigate fields   ↑↓ select background   F feat   Enter proceed   Esc back"
            }
            CharacterCreationStep::Species => "↑↓ navigate   Enter select/expand   Esc back",
            CharacterCreationStep::Abilities => {
                "↑↓ navigate   +/- adjust scores   Enter proceed   Esc back"
            }
            CharacterCreationStep::Equipment => "←→ select option   Enter finish   Esc back",
        }
    };

    let mut footer_lines = vec![Line::from(Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray),
    ))];
    if !app.status_msg.is_empty() {
        footer_lines.push(Line::from(Span::styled(
            app.status_msg.as_str(),
            Style::default().fg(Color::LightYellow),
        )));
    }
    frame.render_widget(Paragraph::new(footer_lines), outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.builder.show_skill_choice_modal {
        handle_skill_choice_modal_key(app, key);
        return;
    }
    if app.builder.show_subclass_modal {
        handle_subclass_modal_key(app, key);
        return;
    }
    if app.builder.show_feat_modal {
        handle_feat_modal_key(app, key);
        return;
    }
    if app.builder.show_progression_asi_modal {
        handle_progression_asi_modal_key(app, key);
        return;
    }
    if app.builder.show_progression_wm_modal {
        handle_progression_wm_modal_key(app, key);
        return;
    }
    if app.builder.feature_detail_modal.is_some() {
        handle_feature_detail_modal_key(app, key);
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
        detail
            .subclasses
            .iter()
            .map(|sc| sc.subclass.clone())
            .collect()
    } else {
        Vec::new()
    }
}

fn render_subclass_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_popup(area, 65, 14);
    frame.render_widget(Clear, popup_area);

    let subclasses = get_available_subclasses(app);
    let class_name = app
        .builder
        .class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .and_then(|c| c.subclass_title.as_deref())
        .unwrap_or("Subclass");

    let items: Vec<ListItem> = subclasses
        .iter()
        .map(|sc| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    sc.name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  ({})", sc.short_name),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let title = format!(" Choose {} (Level {}) ", class_name, app.builder.level);
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
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
        KeyCode::Esc => {
            app.builder.show_subclass_modal = false;
        }
        KeyCode::Up => {
            let cur = app.builder.subclass_list_state.selected().unwrap_or(0);
            let next = if cur > 0 {
                cur - 1
            } else {
                subclasses.len() - 1
            };
            app.builder.subclass_list_state.select(Some(next));
        }
        KeyCode::Down => {
            let cur = app.builder.subclass_list_state.selected().unwrap_or(0);
            let next = if cur + 1 < subclasses.len() {
                cur + 1
            } else {
                0
            };
            app.builder.subclass_list_state.select(Some(next));
        }
        KeyCode::Enter => {
            let selected = app.builder.subclass_list_state.selected().unwrap_or(0);
            if let Some(sc) = subclasses.get(selected) {
                app.builder.subclass_id = Some(sc.id);
                app.status_msg = format!("Subclass '{}' selected.", sc.name);
                step_class::refresh_progression_manifest(app);
            }
            app.builder.show_subclass_modal = false;
        }
        _ => {}
    }
}

fn get_origin_feats(app: &App) -> Vec<crate::models::Feat> {
    app.all_feats
        .iter()
        .filter(|f| f.prerequisite.is_none())
        .cloned()
        .collect()
}

fn render_feat_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_popup(area, 65, 16);
    frame.render_widget(Clear, popup_area);

    let lvl = app.builder.progression_slot_level.unwrap_or(1);
    let filtered = get_valid_asi_feats(
        &app.all_feats,
        lvl,
        &app.builder.feat_picker_search,
    );

    // Clamp cursor to the filtered result set.
    let len = filtered.len();
    let cursor = app.builder.feat_list_state.selected().unwrap_or(0).min(len.saturating_sub(1));
    app.builder.feat_list_state.select(if len == 0 { None } else { Some(cursor) });

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let is_cur = i == cursor;
            let prefix = if is_cur { ">> " } else { "   " };
            let style = if is_cur {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let source_label = f.source_slug_str();
            let display_name = if source_label.is_empty() || source_label.eq_ignore_ascii_case("Other") {
                f.name.clone()
            } else {
                format!("{} [{}]", f.name, source_label.to_uppercase())
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(display_name, style),
            ]))
        })
        .collect();

    let title = format!(
        " Choose Origin Feat — {} feat(s) — Search: {}▌ ",
        len, app.builder.feat_picker_search
    );
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, popup_area, &mut app.builder.feat_list_state);
}

fn handle_feat_modal_key(app: &mut App, key: KeyEvent) {
    let lvl = app.builder.progression_slot_level.unwrap_or(1);
    let feats = get_origin_feats(app);
    let filtered = get_valid_asi_feats(&feats, lvl, &app.builder.feat_picker_search);
    let len = filtered.len();

    match key.code {
        KeyCode::Esc => {
            app.builder.show_feat_modal = false;
            app.builder.feat_picker_search.clear();
            app.builder.feat_list_state.select(None);
        }
        KeyCode::Up => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur > 0 { cur - 1 } else { len.saturating_sub(1) };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Down => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur + 1 < len { cur + 1 } else { 0 };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Enter => {
            let selected = app.builder.feat_list_state.selected().unwrap_or(0);
            if let Some(f) = filtered.get(selected.min(len.saturating_sub(1))) {
                app.builder.background_feat_id = Some(f.id);
                app.status_msg = format!("Origin Feat '{}' selected.", f.name);
            }
            app.builder.show_feat_modal = false;
            app.builder.feat_picker_search.clear();
            app.builder.feat_list_state.select(None);
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

// ── Skill Choice Modal ────────────────────────────────────────────────────────
//
// Shown when the user presses Enter on a "skill_proficiency:N" DecisionSlot in
// the Level 1 progression tree. Displays only the skills listed in the class's
// `skill_choices.from` array, filtered by a live search string. Skills already
// chosen in other slots are shown dimmed and cannot be selected again.

/// Collect the skill option list for the currently highlighted class.
/// Returns (choose_count, allowed_skill_names).
fn get_class_skill_options(app: &App) -> (usize, Vec<String>) {
    let q = app.builder.class_search.to_lowercase();
    let maybe_class: Option<crate::models::Class> = {
        let filtered: Vec<&crate::models::Class> = app
            .classes
            .iter()
            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
            .collect();
        app.builder
            .list_state
            .selected()
            .and_then(|i| filtered.get(i).map(|c| (*c).clone()))
    };
    match maybe_class {
        None => (0, Vec::new()),
        Some(class) => step_class::parse_skill_choices_pub(&class),
    }
}

fn render_skill_choice_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let slot = app.builder.skill_choice_slot;
    let (choose_count, all_options) = get_class_skill_options(app);

    // Skills already committed to *other* slots must be greyed out and
    // excluded from selection to prevent duplicates.
    let taken: std::collections::HashSet<String> = app
        .builder
        .skill_choices
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != slot)
        .filter_map(|(_, s)| if s.is_empty() { None } else { Some(s.clone()) })
        .collect();

    // Skills from higher priority sources (Species, Feat) that may trigger replacement
    let reserved_skills: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        // Species skill
        if let Some(ref s) = app.builder.race_skill_choice {
            set.insert(s.to_lowercase());
        }
        // Feat skills
        for s in &app.builder.feat_skill_choices {
            set.insert(s.to_lowercase());
        }
        set
    };

    let search = app.builder.skill_choice_search.to_lowercase();

    // Build the filtered+annotated list: (display_name, is_taken, is_reserved)
    let visible: Vec<(String, bool, bool)> = all_options
        .iter()
        .filter(|s| search.is_empty() || s.to_lowercase().contains(&search))
        .map(|s| {
            let is_taken = taken.contains(s);
            let is_reserved = reserved_skills.contains(&s.to_lowercase());
            (s.clone(), is_taken, is_reserved)
        })
        .collect();

    let len = visible.len();
    let cursor = app.builder.skill_choice_cursor.min(len.saturating_sub(1));
    app.builder.skill_choice_cursor = cursor;
    app.builder
        .skill_choice_list_state
        .select(if len == 0 { None } else { Some(cursor) });

    let popup_area = centered_popup(area, 60, 18);
    frame.render_widget(Clear, popup_area);

    let count_hint = if search.is_empty() {
        format!("{} skills", len)
    } else {
        format!("{}/{} skills", len, all_options.len())
    };

    let title = format!(
        " Skill Proficiency — Slot {}/{}   {}   Search: {}▌ ",
        slot + 1,
        choose_count,
        count_hint,
        app.builder.skill_choice_search,
    );

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, (name, is_taken, is_reserved))| {
            let is_cursor = i == cursor;
            let prefix = if is_cursor { ">> " } else { "   " };
            if *is_taken {
                // Already chosen in another slot — dim, unselectable visually
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{} (already chosen)", name),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            } else if *is_reserved {
                // Already from Species/BG — show replacement indicator
                let (pre, color, suffix) = if is_cursor {
                    (
                        "⚠ ",
                        Color::Yellow,
                        " (Sudah dari Species/BG — Replacement)",
                    )
                } else {
                    (
                        "⚠ ",
                        Color::Yellow,
                        " (Sudah dari Species/BG — Replacement)",
                    )
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(pre, Style::default().fg(color)),
                    Span::styled(name.clone(), Style::default().fg(color)),
                    Span::styled(suffix, Style::default().fg(Color::DarkGray)),
                ]))
            } else if is_cursor {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        name.clone(),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(name.clone(), Style::default().fg(Color::White)),
                ]))
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));

    frame.render_stateful_widget(list, popup_area, &mut app.builder.skill_choice_list_state);
}

fn handle_skill_choice_modal_key(app: &mut App, key: KeyEvent) {
    use crossterm::event::KeyModifiers;

    let (_, all_options) = get_class_skill_options(app);
    let slot = app.builder.skill_choice_slot;

    let taken: std::collections::HashSet<String> = app
        .builder
        .skill_choices
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != slot)
        .filter_map(|(_, s)| if s.is_empty() { None } else { Some(s.clone()) })
        .collect();

    // Skills from higher priority sources (Species, Feat) that may trigger replacement
    let reserved_skills: std::collections::HashSet<String> = {
        let mut set = std::collections::HashSet::new();
        // Species skill
        if let Some(ref s) = app.builder.race_skill_choice {
            set.insert(s.to_lowercase());
        }
        // Feat skills
        for s in &app.builder.feat_skill_choices {
            set.insert(s.to_lowercase());
        }
        set
    };

    let search = app.builder.skill_choice_search.to_lowercase();
    let visible: Vec<(String, bool, bool)> = all_options
        .iter()
        .filter(|s| search.is_empty() || s.to_lowercase().contains(&search))
        .map(|s| {
            let is_taken = taken.contains(s);
            let is_reserved = reserved_skills.contains(&s.to_lowercase());
            (s.clone(), is_taken, is_reserved)
        })
        .collect();

    let len = visible.len();

    match key.code {
        KeyCode::Esc => {
            app.builder.show_skill_choice_modal = false;
            app.builder.skill_choice_search.clear();
            app.builder.skill_choice_cursor = 0;
        }

        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            if len > 0 {
                let cur = app.builder.skill_choice_cursor;
                app.builder.skill_choice_cursor = if cur == 0 { len - 1 } else { cur - 1 };
            }
        }

        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            if len > 0 {
                app.builder.skill_choice_cursor = (app.builder.skill_choice_cursor + 1) % len;
            }
        }

        KeyCode::Backspace => {
            app.builder.skill_choice_search.pop();
            app.builder.skill_choice_cursor = 0;
        }

        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            app.builder.skill_choice_search.push(c);
            app.builder.skill_choice_cursor = 0;
        }

        KeyCode::Enter => {
            let cursor = app.builder.skill_choice_cursor.min(len.saturating_sub(1));
            if let Some((skill_name, is_taken, is_reserved)) = visible.get(cursor) {
                if *is_taken {
                    app.status_msg = format!("{} is already chosen in another slot.", skill_name);
                    return;
                }
                if *is_reserved {
                    // Warn user that this skill will be marked as replacement
                    app.status_msg = format!(
                        "Skill '{}' sudah dimiliki dari Species/BG. Ini akan dipilih sebagai replacement.",
                        skill_name
                    );
                }
                // Grow skill_choices vec to accommodate this slot index
                while app.builder.skill_choices.len() <= slot {
                    app.builder.skill_choices.push(String::new());
                }
                app.builder.skill_choices[slot] = skill_name.clone();
                app.status_msg = format!(
                    "Skill Proficiency '{}' selected for slot {}.",
                    skill_name,
                    slot + 1
                );
            }
            app.builder.show_skill_choice_modal = false;
            app.builder.skill_choice_search.clear();
            app.builder.skill_choice_cursor = 0;
        }

        _ => {}
    }
}

// ── Progression ASI Modal (multi-stage) ──────────────────────────────────────

const STAT_NAMES: [&str; 6] = [
    "Strength",
    "Dexterity",
    "Constitution",
    "Intelligence",
    "Wisdom",
    "Charisma",
];
const STAT_SHORT: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

fn render_progression_asi_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    use crate::models::app_state::AsiModalStage;
    let lvl = app.builder.progression_slot_level.unwrap_or(4);

    match app.builder.asi_modal_stage {
        AsiModalStage::Mode => render_asi_mode_stage(app, frame, area, lvl),
        AsiModalStage::Stats => render_asi_stats_stage(app, frame, area, lvl),
        AsiModalStage::Feat => render_asi_feat_stage(app, frame, area, lvl),
    }
}

fn render_asi_mode_stage(app: &App, frame: &mut Frame, area: Rect, lvl: i32) {
    let popup_area = centered_popup(area, 55, 10);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" ASI / Feat Choice — Level {} ", lvl))
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let rows = Layout::vertical([
        Constraint::Length(1), // hint
        Constraint::Length(1), // spacer
        Constraint::Length(3), // option A
        Constraint::Length(1), // spacer
        Constraint::Length(3), // option B
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            "  Choose one:",
            Style::default().fg(Color::DarkGray),
        )),
        rows[0],
    );

    let (a_style, b_style) = if app.builder.asi_mode_cursor == 0 {
        (
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(Color::White),
        )
    } else {
        (
            Style::default().fg(Color::White),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" [ A ]  Ability Score Improvement ", a_style)),
            Line::from(Span::styled(
                "        +2 to one stat, or +1 to two stats",
                Style::default().fg(Color::DarkGray),
            )),
        ]),
        rows[2],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(" [ B ]  Choose a General Feat ", b_style)),
            Line::from(Span::styled(
                "        Pick any feat you qualify for",
                Style::default().fg(Color::DarkGray),
            )),
        ]),
        rows[4],
    );
}

fn render_asi_stats_stage(app: &mut App, frame: &mut Frame, area: Rect, lvl: i32) {
    let popup_area = centered_popup(area, 60, 20);
    frame.render_widget(Clear, popup_area);

    // Gather current ability scores for reference
    let scores: [i32; 6] = if let Some(ref c) = app.active_character {
        [
            c.strength,
            c.dexterity,
            c.constitution,
            c.intelligence,
            c.wisdom,
            c.charisma,
        ]
    } else {
        let b = &app.builder;
        [
            b.abilities[0] + b.bg_ability_bonuses[0],
            b.abilities[1] + b.bg_ability_bonuses[1],
            b.abilities[2] + b.bg_ability_bonuses[2],
            b.abilities[3] + b.bg_ability_bonuses[3],
            b.abilities[4] + b.bg_ability_bonuses[4],
            b.abilities[5] + b.bg_ability_bonuses[5],
        ]
    };

    let block = Block::default()
        .title(format!(" Ability Score Improvement — Level {} ", lvl))
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Decide total bonus display
    let slot0 = app.builder.asi_stat_slots[0];
    let slot1 = app.builder.asi_stat_slots[1];
    let bonus_hint = match (slot0, slot1) {
        (Some(a), Some(b)) if a == b => format!("+2 {}", STAT_SHORT[a]),
        (Some(a), Some(b)) => format!("+1 {} / +1 {}", STAT_SHORT[a], STAT_SHORT[b]),
        (Some(a), None) => format!("+1 {} / +1 ?", STAT_SHORT[a]),
        _ => "+1 ? / +1 ?".to_string(),
    };

    let sections = Layout::vertical([
        Constraint::Length(1), // hint
        Constraint::Length(1), // bonus summary
        Constraint::Length(1), // spacer
        Constraint::Length(3), // slot 0
        Constraint::Length(1), // spacer
        Constraint::Length(3), // slot 1
        Constraint::Min(0),    // stat picker (shown inline when open)
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            "  Select two stat slots (same stat = +2, different = +1/+1):",
            Style::default().fg(Color::DarkGray),
        )),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("  Total bonus: {}", bonus_hint),
            Style::default().fg(Color::Cyan),
        )),
        sections[1],
    );

    for (slot_idx, section) in [(0usize, sections[3]), (1usize, sections[5])] {
        let is_active = app.builder.asi_active_slot == slot_idx;
        let label = match app.builder.asi_stat_slots[slot_idx] {
            Some(s) => format!(
                "  Slot {}: {} ({}) ",
                slot_idx + 1,
                STAT_NAMES[s],
                scores[s]
            ),
            None => format!("  Slot {}: [ SELECT STAT ] ", slot_idx + 1),
        };
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(label, style))).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if is_active {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
            ),
            section,
        );
    }

    // Inline stat picker when open
    if app.builder.asi_stat_picker_open {
        let picker_area = sections[6];
        let items: Vec<ListItem> = STAT_NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let cursor = if i == app.builder.asi_stat_picker_cursor {
                    ">> "
                } else {
                    "   "
                };
                ListItem::new(Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:<14} ({})", name, scores[i]),
                        Style::default().fg(Color::White),
                    ),
                ]))
            })
            .collect();
        let picker = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Pick Stat ")
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));
        frame.render_widget(picker, picker_area);
    }
}

pub fn get_valid_asi_feats<'a>(
    feats: &'a [crate::models::Feat],
    lvl: i32,
    search: &str,
) -> Vec<&'a crate::models::Feat> {
    let search = search.to_lowercase();
    let mut valid_feats: std::collections::HashMap<String, &'a crate::models::Feat> =
        std::collections::HashMap::new();

    for f in feats.iter() {
        // Step 1: Constraint minimum level (hanya bisa diambil jika max requirement <= lvl karakter)
        if f.min_level() > lvl {
            continue;
        }

        // Step 2: Pencarian
        if !search.is_empty() && !f.name.to_lowercase().contains(&search) {
            continue;
        }

        // Step 3: De-duplikasi berdasarkan Nama Feat
        // Prioritaskan sumber "xphb" (2024 rules) atau edisi terbaru.
        let lower_name = f.name.to_lowercase();
        let current_source = f.source_slug_str().to_lowercase();

        if let Some(existing) = valid_feats.get(&lower_name) {
            let existing_source = existing.source_slug_str().to_lowercase();
            // Timpa/replace entry lama jika yang baru adalah XPHB.
            if current_source == "xphb" && existing_source != "xphb" {
                valid_feats.insert(lower_name, f);
            }
        } else {
            valid_feats.insert(lower_name, f);
        }
    }

    let mut filtered: Vec<&'a crate::models::Feat> = valid_feats.into_values().collect();
    filtered.sort_by(|a, b| a.name.cmp(&b.name));
    filtered
}

fn render_asi_feat_stage(app: &mut App, frame: &mut Frame, area: Rect, lvl: i32) {
    let popup_area = centered_popup(area, 65, 22);
    frame.render_widget(Clear, popup_area);

    let search = &app.builder.asi_feat_search;
    let filtered = get_valid_asi_feats(&app.all_feats, lvl, search);

    let len = filtered.len();
    // Clamp cursor and keep list_state in sync so Ratatui scrolls the viewport.
    let cursor = app.builder.asi_feat_cursor.min(len.saturating_sub(1));
    app.builder.asi_feat_cursor = cursor;
    app.builder
        .feat_list_state
        .select(if len == 0 { None } else { Some(cursor) });

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let is_cur = i == cursor;
            let prefix = if is_cur { ">> " } else { "   " };
            let style = if is_cur {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let source_label = f.source_slug_str();
            let display_name =
                if source_label.is_empty() || source_label.eq_ignore_ascii_case("Other") {
                    f.name.clone()
                } else {
                    format!("{} [{}]", f.name, source_label.to_uppercase())
                };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(display_name, style),
            ]))
        })
        .collect();

    let count_hint = if search.is_empty() {
        format!("{} feats", len)
    } else {
        format!("{}/{} feats", len, app.all_feats.len())
    };

    let title = format!(
        " Choose Feat — Level {}   {}   Search: {}▌ ",
        lvl, count_hint, app.builder.asi_feat_search,
    );

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        // highlight_style applies to the selected item's background via ListState;
        // the baked prefix gives a clear visual indicator on top of that.
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));

    frame.render_stateful_widget(list, popup_area, &mut app.builder.feat_list_state);
}

fn handle_progression_asi_modal_key(app: &mut App, key: KeyEvent) {
    use crate::models::app_state::AsiModalStage;

    match app.builder.asi_modal_stage {
        AsiModalStage::Mode => handle_asi_mode_key(app, key),
        AsiModalStage::Stats => handle_asi_stats_key(app, key),
        AsiModalStage::Feat => handle_asi_feat_key(app, key),
    }
}

fn handle_asi_mode_key(app: &mut App, key: KeyEvent) {
    use crate::models::app_state::AsiModalStage;
    match key.code {
        KeyCode::Esc => {
            app.builder.show_progression_asi_modal = false;
            app.builder.asi_modal_stage = AsiModalStage::Mode;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.builder.asi_mode_cursor = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.builder.asi_mode_cursor = 1;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.builder.asi_modal_stage = AsiModalStage::Stats;
            app.builder.asi_stat_slots = [None, None];
            app.builder.asi_active_slot = 0;
            app.builder.asi_stat_picker_open = false;
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.builder.asi_modal_stage = AsiModalStage::Feat;
            app.builder.asi_feat_cursor = 0;
            app.builder.asi_feat_search.clear();
        }
        KeyCode::Enter => {
            if app.builder.asi_mode_cursor == 0 {
                app.builder.asi_modal_stage = AsiModalStage::Stats;
                app.builder.asi_stat_slots = [None, None];
                app.builder.asi_active_slot = 0;
                app.builder.asi_stat_picker_open = false;
            } else {
                app.builder.asi_modal_stage = AsiModalStage::Feat;
                app.builder.asi_feat_cursor = 0;
                app.builder.asi_feat_search.clear();
            }
        }
        _ => {}
    }
}

fn handle_asi_stats_key(app: &mut App, key: KeyEvent) {
    use crate::models::app_state::AsiModalStage;

    if app.builder.asi_stat_picker_open {
        // ── Stat picker is open: navigate within the 6-stat list ────────────
        match key.code {
            KeyCode::Esc => {
                app.builder.asi_stat_picker_open = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.builder.asi_stat_picker_cursor > 0 {
                    app.builder.asi_stat_picker_cursor -= 1;
                } else {
                    app.builder.asi_stat_picker_cursor = 5;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.builder.asi_stat_picker_cursor = (app.builder.asi_stat_picker_cursor + 1) % 6;
            }
            KeyCode::Enter => {
                let chosen = app.builder.asi_stat_picker_cursor;
                app.builder.asi_stat_slots[app.builder.asi_active_slot] = Some(chosen);
                app.builder.asi_stat_picker_open = false;
                // Auto-advance to the other slot if it is still empty
                let other = 1 - app.builder.asi_active_slot;
                if app.builder.asi_stat_slots[other].is_none() {
                    app.builder.asi_active_slot = other;
                }
            }
            _ => {}
        }
        return;
    }

    // ── Picker closed: navigate the two slots ────────────────────────────────
    match key.code {
        KeyCode::Esc => {
            // Step back to mode selection
            app.builder.asi_modal_stage = AsiModalStage::Mode;
            app.builder.asi_stat_slots = [None, None];
            app.builder.asi_stat_picker_open = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.builder.asi_active_slot = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.builder.asi_active_slot = 1;
        }
        KeyCode::Enter => {
            // If both slots are filled → submit
            if app.builder.asi_stat_slots[0].is_some() && app.builder.asi_stat_slots[1].is_some() {
                submit_asi_stats(app);
            } else {
                // Open the picker for the active slot
                app.builder.asi_stat_picker_open = true;
                app.builder.asi_stat_picker_cursor =
                    app.builder.asi_stat_slots[app.builder.asi_active_slot].unwrap_or(0);
            }
        }
        KeyCode::Char(' ') => {
            // Space opens picker for active slot
            app.builder.asi_stat_picker_open = true;
            app.builder.asi_stat_picker_cursor =
                app.builder.asi_stat_slots[app.builder.asi_active_slot].unwrap_or(0);
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            // Clear active slot
            app.builder.asi_stat_slots[app.builder.asi_active_slot] = None;
        }
        _ => {}
    }
}

fn handle_asi_feat_key(app: &mut App, key: KeyEvent) {
    use crate::models::app_state::AsiModalStage;
    use crossterm::event::KeyModifiers;

    let lvl = app.builder.progression_slot_level.unwrap_or(4);
    let search = &app.builder.asi_feat_search;
    let filtered_len = get_valid_asi_feats(&app.all_feats, lvl, search).len();

    match key.code {
        KeyCode::Esc => {
            // Step back to mode selection
            app.builder.asi_modal_stage = AsiModalStage::Mode;
            app.builder.asi_feat_search.clear();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.builder.asi_feat_cursor > 0 {
                app.builder.asi_feat_cursor -= 1;
            } else {
                app.builder.asi_feat_cursor = filtered_len.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if filtered_len > 0 {
                app.builder.asi_feat_cursor = (app.builder.asi_feat_cursor + 1) % filtered_len;
            }
        }
        KeyCode::Backspace => {
            app.builder.asi_feat_search.pop();
            app.builder.asi_feat_cursor = 0;
        }
        KeyCode::Char(c)
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            app.builder.asi_feat_search.push(c);
            app.builder.asi_feat_cursor = 0;
        }
        KeyCode::Enter => {
            submit_asi_feat(app);
        }
        _ => {}
    }
}

// ── Submit helpers ─────────────────────────────────────────────────────────────

/// Build bumps array from the two stat slots and post to the API.
fn submit_asi_stats(app: &mut App) {
    use crate::models::app_state::AsiModalStage;

    let slot0 = match app.builder.asi_stat_slots[0] {
        Some(s) => s,
        None => {
            app.status_msg = "Please fill both stat slots.".to_string();
            return;
        }
    };
    let slot1 = match app.builder.asi_stat_slots[1] {
        Some(s) => s,
        None => {
            app.status_msg = "Please fill both stat slots.".to_string();
            return;
        }
    };

    // Build bumps: same stat twice → +2; different → +1/+1
    let mut bumps = [0i32; 6];
    bumps[slot0] += 1;
    bumps[slot1] += 1;

    let label = if slot0 == slot1 {
        format!("+2 {}", STAT_NAMES[slot0])
    } else {
        format!("+1 {} / +1 {}", STAT_NAMES[slot0], STAT_NAMES[slot1])
    };

    let lvl = app.builder.progression_slot_level.unwrap_or(4);

    // Save choice to local builder state
    app.builder.asi_choices.insert(lvl, label.clone());

    // Update local progression manifest if present
    if let Some(ref mut manifest) = app.builder.progression_manifest {
        if let Some(dp) = manifest
            .decision_points
            .iter_mut()
            .find(|dp| dp.level == lvl && dp.choice_type == "asi")
        {
            dp.status = crate::models::DecisionStatus::Complete;
            dp.current_choices = vec![crate::models::DecisionPointChoice {
                id: "asi".to_string(),
                description: label.clone(),
            }];
        }
    }

    let char_id = app
        .builder
        .draft_id
        .or_else(|| app.active_character.as_ref().map(|c| c.id));

    if let Some(id) = char_id {
        let req = crate::models::AsiChoiceRequest {
            bump_str: Some(bumps[0]),
            bump_dex: Some(bumps[1]),
            bump_con: Some(bumps[2]),
            bump_int: Some(bumps[3]),
            bump_wis: Some(bumps[4]),
            bump_cha: Some(bumps[5]),
            feat_id: None,
            source_type: Some("level".to_string()),
            gained_at_level: Some(lvl),
        };
        let rt = app.rt.clone();
        let client = app.client.clone();
        match rt.block_on(client.post_asi_choice(id, &req)) {
            Ok(_) => {
                app.status_msg = format!("ASI '{}' saved!", label);
                step_class::refresh_progression_manifest(app);
            }
            Err(e) => {
                app.status_msg = format!("Failed to save ASI: {}", e);
                return;
            }
        }
    } else {
        app.status_msg = format!("ASI '{}' chosen.", label);
    }

    app.builder.show_progression_asi_modal = false;
    app.builder.asi_modal_stage = AsiModalStage::Mode;
    app.builder.asi_stat_slots = [None, None];
}

/// Pick the currently highlighted feat and post to the API.
fn submit_asi_feat(app: &mut App) {
    use crate::models::app_state::AsiModalStage;

    let lvl = app.builder.progression_slot_level.unwrap_or(4);
    let search = &app.builder.asi_feat_search;
    let filtered = get_valid_asi_feats(&app.all_feats, lvl, search);

    let cursor = app
        .builder
        .asi_feat_cursor
        .min(filtered.len().saturating_sub(1));
    let feat = match filtered.get(cursor) {
        Some(f) => (*f).clone(),
        None => {
            app.status_msg = "No feat selected.".to_string();
            return;
        }
    };

    let lvl = app.builder.progression_slot_level.unwrap_or(4);

    // Save choice to local builder state
    app.builder.asi_choices.insert(lvl, feat.name.clone());

    // Update local progression manifest if present
    if let Some(ref mut manifest) = app.builder.progression_manifest {
        if let Some(dp) = manifest
            .decision_points
            .iter_mut()
            .find(|dp| dp.level == lvl && dp.choice_type == "asi")
        {
            dp.status = crate::models::DecisionStatus::Complete;
            dp.current_choices = vec![crate::models::DecisionPointChoice {
                id: feat.id.to_string(),
                description: feat.name.clone(),
            }];
        }
    }

    let char_id = app
        .builder
        .draft_id
        .or_else(|| app.active_character.as_ref().map(|c| c.id));

    if let Some(id) = char_id {
        let req = crate::models::AsiChoiceRequest {
            bump_str: None,
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: Some(feat.id),
            source_type: Some("level".to_string()),
            gained_at_level: Some(lvl),
        };
        let rt = app.rt.clone();
        let client = app.client.clone();
        match rt.block_on(client.post_asi_choice(id, &req)) {
            Ok(_) => {
                app.status_msg = format!("Feat '{}' saved!", feat.name);
                step_class::refresh_progression_manifest(app);
            }
            Err(e) => {
                app.status_msg = format!("Failed to save feat: {}", e);
                return;
            }
        }
    } else {
        app.status_msg = format!("Feat '{}' chosen.", feat.name);
    }

    app.builder.show_progression_asi_modal = false;
    app.builder.asi_modal_stage = AsiModalStage::Mode;
    app.builder.asi_feat_search.clear();
    app.builder.asi_feat_cursor = 0;
}

// ── Weapon Mastery Modal ──

fn render_progression_wm_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_popup(area, 65, 16);
    frame.render_widget(Clear, popup_area);

    let weapons = app.filtered_mastery_weapons();
    let lvl = app.builder.progression_slot_level.unwrap_or(1);

    let items: Vec<ListItem> = weapons
        .iter()
        .map(|w| {
            let mastery_prop = crate::utils::weapon_mastery::get_mastery_property(&w.name);
            ListItem::new(Line::from(vec![
                Span::styled(
                    w.name.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  [{}]", mastery_prop),
                    Style::default().fg(Color::Cyan),
                ),
            ]))
        })
        .collect();

    let title = format!(" Choose Weapon Mastery (Level {}) ", lvl);
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, popup_area, &mut app.builder.feat_list_state);
}

fn handle_progression_wm_modal_key(app: &mut App, key: KeyEvent) {
    let weapons = app.filtered_mastery_weapons();
    if weapons.is_empty() {
        app.builder.show_progression_wm_modal = false;
        return;
    }

    match key.code {
        KeyCode::Esc => {
            app.builder.show_progression_wm_modal = false;
        }
        KeyCode::Up => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur > 0 { cur - 1 } else { weapons.len() - 1 };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Down => {
            let cur = app.builder.feat_list_state.selected().unwrap_or(0);
            let next = if cur + 1 < weapons.len() { cur + 1 } else { 0 };
            app.builder.feat_list_state.select(Some(next));
        }
        KeyCode::Enter => {
            let selected = app.builder.feat_list_state.selected().unwrap_or(0);
            let weapon_name = app
                .filtered_mastery_weapons()
                .get(selected)
                .map(|w| w.name.clone());
            if let Some(wname) = weapon_name {
                if !app.builder.weapon_mastery_choices.contains(&wname) {
                    app.builder.weapon_mastery_choices.push(wname.clone());
                }
                let lvl = app.builder.progression_slot_level.unwrap_or(1);
                if let Some(ref mut manifest) = app.builder.progression_manifest {
                    if let Some(dp) = manifest
                        .decision_points
                        .iter_mut()
                        .find(|dp| dp.level == lvl && dp.choice_type == "weapon_mastery")
                    {
                        dp.status = crate::models::DecisionStatus::Complete;
                        if !dp.current_choices.iter().any(|c| c.description == wname) {
                            dp.current_choices.push(crate::models::DecisionPointChoice {
                                id: wname.clone(),
                                description: wname.clone(),
                            });
                        }
                    }
                }
                app.status_msg = format!("Weapon Mastery '{}' selected.", wname);
                step_class::refresh_progression_manifest(app);
            }
            app.builder.show_progression_wm_modal = false;
        }
        _ => {}
    }
}

// ── Feature Detail Modal (Ctrl+K) ──

fn render_feature_detail_modal(app: &mut App, frame: &mut Frame, area: Rect) {
    if let Some((title, body)) = &app.builder.feature_detail_modal {
        let popup_area = centered_popup(area, 75, 18);
        frame.render_widget(Clear, popup_area);

        let p = Paragraph::new(body.as_str())
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .wrap(Wrap { trim: true })
            .scroll((app.builder.feature_modal_scroll, 0));

        frame.render_widget(p, popup_area);
    }
}

fn handle_feature_detail_modal_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            app.builder.feature_detail_modal = None;
            app.builder.feature_modal_scroll = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.builder.feature_modal_scroll = app.builder.feature_modal_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.builder.feature_modal_scroll = app.builder.feature_modal_scroll.saturating_add(1);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_test_feat(
        id: i32,
        name: &str,
        source_slug: Option<&str>,
        prereq: Option<serde_json::Value>,
    ) -> crate::models::Feat {
        crate::models::Feat {
            id,
            name: name.to_string(),
            source_id: 1,
            source_slug: source_slug.map(|s| s.to_string()),
            prerequisite: prereq,
            ability: None,
            entries: json!([]),
            has_uses: false,
        }
    }

    #[test]
    fn test_get_valid_asi_feats_level_filtering() {
        let feats = vec![
            make_test_feat(1, "Alert", Some("xphb"), None), // min_level = 1
            make_test_feat(2, "Actor", Some("xphb"), Some(json!([{"level": 4}]))), // min_level = 4
            make_test_feat(
                3,
                "Boon of Combat Prowess",
                Some("xphb"),
                Some(json!([{"level": {"level": 19}}])),
            ), // min_level = 19
        ];

        let filtered_lvl4 = get_valid_asi_feats(&feats, 4, "");
        assert_eq!(filtered_lvl4.len(), 2);
        assert!(filtered_lvl4.iter().any(|f| f.name == "Alert"));
        assert!(filtered_lvl4.iter().any(|f| f.name == "Actor"));
        assert!(
            !filtered_lvl4
                .iter()
                .any(|f| f.name == "Boon of Combat Prowess")
        );

        let filtered_lvl19 = get_valid_asi_feats(&feats, 19, "");
        assert_eq!(filtered_lvl19.len(), 3);
    }

    #[test]
    fn test_get_valid_asi_feats_deduplication_prefers_xphb() {
        let feats = vec![
            make_test_feat(1, "Alert", Some("phb"), None),
            make_test_feat(2, "Alert", Some("xphb"), None),
            make_test_feat(3, "Actor", Some("phb"), None),
        ];

        let filtered = get_valid_asi_feats(&feats, 4, "");
        assert_eq!(filtered.len(), 2);

        let alert = filtered.iter().find(|f| f.name == "Alert").unwrap();
        assert_eq!(alert.id, 2);
        assert_eq!(alert.source_slug.as_deref(), Some("xphb"));
    }

    #[test]
    fn test_get_valid_asi_feats_search_filter() {
        let feats = vec![
            make_test_feat(1, "Alert", Some("xphb"), None),
            make_test_feat(2, "Actor", Some("xphb"), None),
            make_test_feat(3, "Tough", Some("xphb"), None),
        ];

        let filtered = get_valid_asi_feats(&feats, 4, "act");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Actor");
    }
}
