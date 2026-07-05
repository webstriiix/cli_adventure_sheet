use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let body = Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).split(area);

    // ── Left: Class list ──
    let items: Vec<ListItem> = app.classes.iter().map(|c| {
        let is_selected = Some(c.id) == app.builder.class_id;
        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        ListItem::new(Line::from(vec![
            Span::styled(c.name.clone(), style),
            Span::styled(format!(" [{}]", c.source_slug), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Class ").border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // ── Right: Level table + class info ──
    let right = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(body[1]);

    // Level selector header
    let subclass_indicator = if app.builder.subclass_id.is_some() {
        let sc_name = app.class_detail.as_ref()
            .and_then(|d| d.subclasses.iter().find(|s| Some(s.subclass.id) == app.builder.subclass_id))
            .map(|s| s.subclass.name.clone())
            .unwrap_or_default();
        format!(" | Subclass: {}", sc_name)
    } else if app.builder.level >= 3 {
        " | [Enter 'S' to pick subclass]".to_string()
    } else {
        String::new()
    };

    let level_header = Paragraph::new(Line::from(vec![
        Span::styled("Level: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", app.builder.level), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("  (-/+ to change)", Style::default().fg(Color::DarkGray)),
        Span::styled(subclass_indicator, Style::default().fg(Color::Cyan)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(level_header, right[0]);

    // Level progression table
    if let Some(idx) = app.builder.list_state.selected() {
        // clone class to avoid holding an immutable borrow while rendering (rendering needs &mut App)
        if let Some(class) = app.classes.get(idx).cloned() {
            render_class_detail(app, frame, right[1], &class);
        }
    }
}

fn render_class_detail(app: &mut App, frame: &mut Frame, area: Rect, class: &crate::models::Class) {
    let panel = Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);

    // Progression table
    let prof_bonus = |lvl: i32| -> i32 { ((lvl - 1) / 4) + 2 };

    // Use class_detail only if it matches the class being rendered; otherwise treat as empty to avoid showing stale data
    let features = if app.class_detail.as_ref().map(|d| d.class.id) == Some(class.id) {
        app.class_detail.as_ref().map(|d| d.features.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };

    let header = Row::new(vec![
        Cell::from("Lvl").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Prof").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Cantrips").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Prepared").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Slots(1..9)").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Features").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    // If class_detail doesn't match this class, show loading placeholder instead of stale features
    let detail_loaded = app.class_detail.as_ref().map(|d| d.class.id) == Some(class.id);

    // Attempt to extract cantrips/prepared from class_table if available
    let mut table_cantrips: Vec<Option<String>> = vec![None; 20];
    let mut table_prepared: Vec<Option<String>> = vec![None; 20];
    if let Some(tbl) = &class.class_table {
        for (i, entry) in tbl.iter().enumerate().take(20) {
            if let Some(arr) = entry.as_array() {
                // Heuristic: if array length >= 2, assume [cantrips, prepared, ...]
                if arr.len() >= 1 {
                    table_cantrips[i] = Some(arr[0].to_string().trim_matches('"').to_string());
                }
                if arr.len() >= 2 {
                    table_prepared[i] = Some(arr[1].to_string().trim_matches('"').to_string());
                }
            }
        }
    }

    // Helper to render spell slots for a level (abbreviated to 1..5)
    let slot_str_for_level = |lvl: usize| -> String {
        if let Some(slots) = &class.spell_slots {
            if let Some(row) = slots.get(lvl) {
                // show up to 5 slot levels (1..5) for compactness
                let mut parts = Vec::new();
                for (idx, &v) in row.iter().enumerate().take(5) {
                    let display = if v <= 0 { "—".to_string() } else { v.to_string() };
                    parts.push(format!("{}:{}", idx + 1, display));
                }
                return parts.join(" ");
            }
        }
        "—".to_string()
    };

    // truncate long strings for cantrips/prepared/features to keep table readable
    let truncate = |s: &str, n: usize| {
        if s.len() <= n { s.to_string() } else { format!("{}…", &s[..n.saturating_sub(1)]) }
    };

    if !detail_loaded {
        let loading = Paragraph::new(Span::styled("Loading class details...", Style::default().fg(Color::DarkGray)))
            .block(Block::default().borders(Borders::ALL).title(" Level Progression (1-20) ").border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(loading, panel[0]);
    } else {
        let rows: Vec<Row> = (1i32..=20).map(|lvl| {
            let li = (lvl - 1) as usize;
            let is_current = lvl == app.builder.level;
            let feats_at_level: Vec<&str> = features.iter()
                .filter(|f| f.level == lvl && !f.is_subclass_gate)
                .map(|f| f.name.as_str())
                .collect();
            let feat_str_raw = if feats_at_level.is_empty() { "—".to_string() } else { feats_at_level.join(", ") };
            let feat_str = truncate(&feat_str_raw, 40);

            let cantrips_raw = table_cantrips.get(li).and_then(|o| o.clone()).unwrap_or_else(|| "—".to_string());
            let cantrips = truncate(&cantrips_raw, 8);
            let prepared_raw = table_prepared.get(li).and_then(|o| o.clone()).unwrap_or_else(|| "—".to_string());
            let prepared = truncate(&prepared_raw, 8);
            let slots = slot_str_for_level(li);

            let style = if is_current {
                Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            Row::new(vec![
                Cell::from(format!("{:>2}", lvl)),
                Cell::from(format!("+{}", prof_bonus(lvl))),
                Cell::from(cantrips),
                Cell::from(prepared),
                Cell::from(slots),
                Cell::from(feat_str),
            ]).style(style)
        }).collect();

        let table = Table::new(rows, [Constraint::Length(4), Constraint::Length(5), Constraint::Length(9), Constraint::Length(9), Constraint::Length(18), Constraint::Min(0)])
            .header(header)
            .block(Block::default().borders(Borders::ALL).title(" Level Progression (1-20) ").border_style(Style::default().fg(Color::DarkGray)));
        frame.render_stateful_widget(table, panel[0], &mut app.builder.progression_table_state);
    }

    // Class info summary
    let mut info_lines = Vec::new();
    info_lines.push(Line::from(vec![
        Span::styled(class.name.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  d{} Hit Die", class.hit_die), Style::default().fg(Color::DarkGray)),
    ]));
    if let Some(saves) = &class.proficiency_saves {
        info_lines.push(Line::from(vec![
            Span::styled("Saves: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(saves.join(", ").to_uppercase()),
        ]));
    }
    if let Some(ability) = &class.spellcasting_ability {
        info_lines.push(Line::from(vec![
            Span::styled("Spellcasting: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(ability.to_uppercase()),
        ]));
    }
    if let Some(sub_title) = &class.subclass_title {
        info_lines.push(Line::from(vec![
            Span::styled("Subclass Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(sub_title.clone()),
        ]));
    }
    if app.builder.level >= 3 {
        info_lines.push(Line::from(""));
        info_lines.push(Line::from(Span::styled(
            "  Level 3+: Press 'S' to choose a Subclass",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
        )));
    }

    let info_p = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Class Info ").border_style(Style::default().fg(Color::DarkGray)))
        .wrap(Wrap { trim: true });
    frame.render_widget(info_p, panel[1]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.screen = crate::models::app_state::Screen::CharacterList;
            app.builder = crate::models::app_state::BuilderState::default();
        }
        KeyCode::Up => {
            let len = app.classes.len();
            let i = match app.builder.list_state.selected() {
                Some(0) | None => len.saturating_sub(1),
                Some(i) => i - 1,
            };
            app.builder.list_state.select(Some(i));
            // Load class details for the new selection
            load_class_detail_for_selected(app);
            // ensure progression table selection follows current level
            app.builder.progression_table_state.select(Some((app.builder.level - 1) as usize));
        }
        KeyCode::Down => {
            let len = app.classes.len();
            let i = match app.builder.list_state.selected() {
                Some(i) if i + 1 < len => i + 1,
                _ => 0,
            };
            app.builder.list_state.select(Some(i));
            load_class_detail_for_selected(app);
            app.builder.progression_table_state.select(Some((app.builder.level - 1) as usize));
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.builder.level > 1 {
                app.builder.level -= 1;
                if app.builder.level < 3 {
                    app.builder.subclass_id = None;
                }
                app.builder.progression_table_state.select(Some((app.builder.level - 1) as usize));
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.builder.level < 20 {
                app.builder.level += 1;
            }
            // Ensure progression table selection keeps up with level
            app.builder.progression_table_state.select(Some((app.builder.level - 1) as usize));
            // Auto-trigger subclass modal at level 3 if no subclass yet
            if app.builder.level >= 3 && app.builder.subclass_id.is_none() {
                let has_subclasses = app.class_detail.as_ref().map(|d| !d.subclasses.is_empty()).unwrap_or(false);
                if has_subclasses {
                    app.builder.show_subclass_modal = true;
                    app.builder.subclass_list_state.select(Some(0));
                }
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if app.builder.level >= 3 {
                let has_subclasses = app.class_detail.as_ref().map(|d| !d.subclasses.is_empty()).unwrap_or(false);
                if has_subclasses {
                    app.builder.show_subclass_modal = true;
                    app.builder.subclass_list_state.select(Some(0));
                } else {
                    app.status_msg = "No subclasses available for this class.".to_string();
                }
            } else {
                app.status_msg = format!("Reach level 3 to pick a subclass (currently level {}).", app.builder.level);
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = app.builder.list_state.selected() {
                if let Some(class) = app.classes.get(idx) {
                    // Clone needed values early to avoid holding an immutable borrow across mutable calls
                    let class_id = class.id;
                    let caster_progression = class.caster_progression.clone();

                    // Load class details if not yet loaded
                    if app.class_detail.as_ref().map(|d| d.class.id != class_id).unwrap_or(true) {
                        load_class_detail_for_selected(app);
                    }

                    app.builder.class_id = Some(class_id);
                    // Ensure progression table selection aligns with current level
                    app.builder.progression_table_state.select(Some((app.builder.level - 1) as usize));
                    app.builder.spellcasting_type = caster_progression
                        .filter(|p| !p.is_empty() && p != "none")
                        .unwrap_or_else(|| "none".to_string());

                    // Check connectivity and save draft before advancing
                    if !app.save_draft() {
                        return; // save_draft sets status_msg on failure
                    }

                    // If level 3+ and has subclasses but none chosen, prompt for subclass first
                    if app.builder.level >= 3 && app.builder.subclass_id.is_none() {
                        let has_subclasses = app.class_detail.as_ref().map(|d| !d.subclasses.is_empty()).unwrap_or(false);
                        if has_subclasses {
                            app.builder.show_subclass_modal = true;
                            app.builder.subclass_list_state.select(Some(0));
                            return;
                        }
                    }

                    app.builder.step = CharacterCreationStep::Background;
                    app.builder.list_state.select(Some(0));
                    app.status_msg.clear();
                }
            }
        }
        _ => {}
    }
}

fn load_class_detail_for_selected(app: &mut App) {
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx) {
            let name = class.name.clone();
            let source = class.source_slug.clone();
            let current_id = app.class_detail.as_ref().map(|d| d.class.id);
            if current_id != Some(class.id) {
                let rt = app.rt.clone();
                let client = app.client.clone();
                match rt.block_on(client.get_class_detail(&name, &source)) {
                    Ok(detail) => { app.class_detail = Some(detail); }
                    Err(_) => {}
                }
            }
        }
    }
}
