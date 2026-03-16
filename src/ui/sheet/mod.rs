mod actions;
mod background_info;
mod core_stats;
pub mod features;
mod inventory;
mod notes;
mod proficiency;
mod skills;
mod spells;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::models::app_state::{PickerMode, SheetTab};

pub fn render(app: &mut App, frame: &mut Frame) {
    // Ensure "always prepared" spells from features (like Divine Smite) are in the list
    app.sync_always_prepared_spells();
    // Ensure scaling resources like Lay on Hands and Channel Divinity are updated
    app.sync_resource_limits();

    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3), // top bar
        Constraint::Min(10),   // sidebar + content
        Constraint::Length(1), // help bar
    ])
    .split(area);

    render_top_bar(app, frame, chunks[0]);

    // Sidebar (20 cols) | Content (rest)
    let body = Layout::horizontal([Constraint::Length(22), Constraint::Min(30)]).split(chunks[1]);

    render_sidebar(app, frame, body[0]);
    render_content(app, frame, body[1]);
    render_help_bar(app, frame, chunks[2]);

    // Render picker overlay if active
    if app.picker_mode != PickerMode::None {
        render_picker_overlay(app, frame, area);
    }

    // Render action detail modal if open
    if let Some((ref name, ref description)) = app.actions_detail_modal {
        render_action_detail_modal(name, description, frame, area);
    }

    // Render spell detail modal if open
    if let Some((ref name, ref description)) = app.spell_detail_modal {
        render_action_detail_modal(name, description, frame, area);
    }
}

fn render_top_bar(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    let level = crate::utils::level_from_xp(character.experience_pts);
    let hp_color = if character.current_hp <= character.max_hp / 4 {
        Color::Red
    } else if character.current_hp <= character.max_hp / 2 {
        Color::Yellow
    } else {
        Color::Green
    };

    let line = Line::from(vec![
        Span::styled(
            format!("  {} ", character.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("Lvl {level}  "), Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}  ", app.char_race_name),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            if !app.char_classes.is_empty() {
                format!("{}  ", app.all_classes_display())
            } else if app.char_subclass_name.is_empty() {
                format!("{}  ", app.char_class_name)
            } else {
                format!("{} ({})  ", app.char_class_name, app.char_subclass_name)
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("HP ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{}", character.current_hp, character.max_hp),
            Style::default().fg(hp_color).add_modifier(Modifier::BOLD),
        ),
        if character.temp_hp > 0 {
            Span::styled(
                format!(" (+{})", character.temp_hp),
                Style::default().fg(Color::Blue),
            )
        } else {
            Span::raw("")
        },
        Span::raw("  "),
        if character.inspiration {
            Span::styled(
                "INSPIRED",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
        // Concentration indicator
        if app.concentrating_on.is_some() {
            Span::styled(
                "  ◈ CONC",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
        // Active conditions summary
        if !app.conditions.is_empty() {
            Span::styled(
                format!("  ⚠ {}", app.conditions.join(", ")),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ]);

    let bar = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Character Sheet ")
            .title_style(Style::default().fg(Color::Cyan))
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(bar, area);
}

fn render_sidebar(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = SheetTab::ALL
        .iter()
        .enumerate()
        .map(|(i, tab): (usize, &SheetTab)| {
            let is_active = i == app.sheet_tab_index;
            let prefix = if is_active { " > " } else { "   " };

            let style = if is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(format!("{prefix}{}", tab.label())).style(style)
        })
        .collect();

    let border_color = if app.sidebar_focused {
        Color::Cyan
    } else {
        Color::Gray
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Menu ")
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(list, area);
}

fn render_content(app: &mut App, frame: &mut Frame, area: Rect) {
    let border_color = if !app.sidebar_focused {
        Color::Cyan
    } else {
        Color::Gray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", app.sheet_tab.label()))
        .title_style(Style::default().fg(Color::Cyan))
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.sheet_tab {
        SheetTab::CoreStats => core_stats::render(app, frame, inner),
        SheetTab::Skills => skills::render(app, frame, inner),
        SheetTab::Actions => actions::render(app, frame, inner),
        SheetTab::Inventory => inventory::render(app, frame, inner),
        SheetTab::Spells => spells::render(app, frame, inner),
        SheetTab::Features => features::render(app, frame, inner),
        SheetTab::Proficiency => proficiency::render(app, frame, inner),
        SheetTab::Background => background_info::render(app, frame, inner),
        SheetTab::Notes => notes::render(app, frame, inner),
    }
}

fn render_help_bar(app: &App, frame: &mut Frame, area: Rect) {
    let help = if app.editing_notes {
        Line::from(vec![
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" save & exit  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" new line  "),
            Span::styled("Type", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to write"),
        ])
    } else if app.picker_mode != PickerMode::None {
        Line::from(vec![
            Span::styled("Type", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to search  "),
            Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" select  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" confirm  "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" cancel"),
        ])
    } else if app.sidebar_focused {
        Line::from(vec![
            Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" navigate  "),
            Span::styled("Enter/→", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" view  "),
            Span::styled("E", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" edit character  "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" back  "),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" quit"),
        ])
    } else {
        // Content-focused help varies by tab
        match app.sheet_tab {
            SheetTab::Notes => Line::from(vec![
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" edit  "),
                Span::styled("Esc/Left", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back to menu  "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
            SheetTab::Inventory => Line::from(vec![
                Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" add  "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" remove  "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" equip  "),
                Span::styled("t", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" attune  "),
                Span::styled("+/-", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" qty  "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" currency  "),
                Span::styled("[/]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" adjust  "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back  "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
            SheetTab::Spells => Line::from(vec![
                Span::styled("a", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" add  "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" remove  "),
                Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" prepared  "),
                Span::styled("z", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" concentrate  "),
                Span::styled("K", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" details  "),
                Span::styled("Tab/S-Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" level filter  "),
                Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" select  "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back"),
            ]),
            SheetTab::Features => Line::from(vec![
                Span::styled("A", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" ASI/Feat  "),
                Span::styled("w", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" mastery  "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" subclass  "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" remove  "),
                Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" select  "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back"),
            ]),
            SheetTab::CoreStats => Line::from(vec![
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" inspiration  "),
                Span::styled("s/d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" death saves  "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" conditions  "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back"),
            ]),
            _ => Line::from(vec![
                Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" scroll  "),
                Span::styled("Esc/Left", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" back to menu  "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" quit"),
            ]),
        }
    };

    // Show status message if present
    let bar = if !app.status_msg.is_empty() {
        let mut spans = vec![
            Span::styled(
                format!(" {} ", app.status_msg),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(" | "),
        ];
        spans.extend(help.spans);
        Paragraph::new(Line::from(spans)).style(Style::default().fg(Color::DarkGray))
    } else {
        Paragraph::new(help).style(Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(bar, area);
}

fn render_picker_overlay(app: &mut App, frame: &mut Frame, area: Rect) {
    // Widen popup when item detail panel is open
    let detail_open = app.picker_mode == PickerMode::ItemPicker && app.show_item_detail;
    let popup_width = if detail_open {
        (area.width.saturating_sub(4)).min(area.width)
    } else {
        50.min(area.width.saturating_sub(4))
    };
    let popup_height = 22.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let title = match app.picker_mode {
        PickerMode::ItemPicker => " Add Item  [Ctrl+K] toggle details ",
        PickerMode::SpellPicker => " Add Spell ",
        PickerMode::FeatPicker => " Choose Feat ",
        PickerMode::AsiFeatChoice => " ASI / Feat Choice ",
        PickerMode::ConditionPicker => " Conditions  [Enter] toggle  [Esc] close ",
        PickerMode::SubclassPicker => " Choose Subclass  [Enter] confirm  [Esc] cancel ",
        PickerMode::WeaponMasteryPicker => " Weapon Mastery  [Enter] confirm  [Esc] cancel ",
        PickerMode::None => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    match app.picker_mode {
        PickerMode::FeatPicker => render_feat_picker(app, frame, inner),
        PickerMode::AsiFeatChoice => render_asi_choice(app, frame, inner),
        PickerMode::ItemPicker => render_list_picker(app, frame, inner, true),
        PickerMode::SpellPicker => render_list_picker(app, frame, inner, false),
        PickerMode::ConditionPicker => render_condition_picker(app, frame, inner),
        PickerMode::SubclassPicker => {
            render_subclass_picker(app, frame, popup_area);
        }
        PickerMode::WeaponMasteryPicker => {
            render_weapon_mastery_choice(app, frame, popup_area);
        }
        PickerMode::None => {}
    }
}

fn render_weapon_mastery_choice(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // search label
        Constraint::Min(0),    // list
    ])
    .split(area);

    let search_label = format!("Search: {}_", app.picker_search);
    let search_para = Paragraph::new(search_label).style(Style::default().fg(Color::Cyan));
    frame.render_widget(search_para, chunks[0]);

    let weapons = app.filtered_mastery_weapons();
    let mut items: Vec<ListItem> = Vec::new();

    for weapon in weapons {
        let is_selected = app.char_weapon_masteries.contains(&weapon.name);
        let checkbox = if is_selected {
            Span::styled(
                "[X] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
        };
        let mastery_text = weapon.mastery.unwrap_or_default().join(", ");

        items.push(ListItem::new(Line::from(vec![
            checkbox,
            Span::styled(weapon.name.clone(), Style::default().fg(Color::White)),
            Span::styled(
                format!("  ({})", mastery_text),
                Style::default().fg(Color::DarkGray),
            ),
        ])));
    }

    let list = ratatui::widgets::List::new(items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut state = ratatui::widgets::ListState::default().with_selected(Some(app.picker_selected));
    frame.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_feat_picker(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(1), // search label
        Constraint::Length(1), // search input
        Constraint::Length(1), // hint
        Constraint::Min(1),    // results
    ])
    .split(area);

    let label = Paragraph::new("  Search feats:").style(Style::default().fg(Color::White));
    frame.render_widget(label, chunks[0]);

    let input = Paragraph::new(format!("  {}▌", app.picker_search))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[1]);

    let character = app.active_character.clone();
    let feats: Vec<ListItem> = app
        .filtered_feats(character, None)
        .iter()
        .map(|f| {
            let prereq_tag = if f.prerequisite.is_some() { " *" } else { "" };
            ListItem::new(format!("  {}{}", f.name, prereq_tag))
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let hint = Paragraph::new("  * = has prerequisites  Enter=add  Esc=cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, chunks[2]);

    if feats.is_empty() {
        frame.render_widget(
            Paragraph::new("  No feats found").style(Style::default().fg(Color::DarkGray)),
            chunks[3],
        );
    } else {
        let list = List::new(feats)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" > ");
        app.picker_list_state.select(Some(app.picker_selected));
        frame.render_stateful_widget(list, chunks[3], &mut app.picker_list_state);
    }
}

fn render_asi_choice(app: &App, frame: &mut Frame, area: Rect) {
    use crate::utils::ABILITY_NAMES;

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "  Ability Score Improvement",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [A] +2/+1 split   [S] +1/+1/+1 split   [F] Choose a Feat instead",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
    ];

    use crate::app::AsiMode;
    match app.asi_mode {
        AsiMode::PlusOneThree => {
            lines.push(Line::from(format!(
                "  +1 to: {} (Tab)    +1 to: {} (Shift+Tab)    +1 to: {} (Ctrl+Tab)",
                ABILITY_NAMES[app.asi_ability_a],
                ABILITY_NAMES[app.asi_ability_b],
                ABILITY_NAMES[app.asi_ability_c]
            )));
        }
        AsiMode::PlusOneTwo => {
            lines.push(Line::from(format!(
                "  +2 to: {} (Tab to change A)    +1 to: {} (Shift+Tab to change B)",
                ABILITY_NAMES[app.asi_ability_a], ABILITY_NAMES[app.asi_ability_b]
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press Enter to confirm",
        Style::default().fg(Color::Green),
    )));

    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

fn render_condition_picker(app: &App, frame: &mut Frame, area: Rect) {
    use crate::handlers::sheet::ALL_CONDITIONS;
    use ratatui::widgets::{List, ListItem, ListState};

    let items: Vec<ListItem> = ALL_CONDITIONS
        .iter()
        .map(|&cond| {
            let active = app.conditions.iter().any(|c| c == cond);
            let (marker, color) = if active {
                ("✓ ", Color::Red)
            } else {
                ("  ", Color::White)
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    marker,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(cond, Style::default().fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(app.picker_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_subclass_picker(app: &mut App, frame: &mut Frame, area: Rect) {
    use ratatui::widgets::ListState;
    let detail = match &app.class_detail {
        Some(d) => d.clone(),
        None => {
            frame.render_widget(
                Paragraph::new("  No subclass data available.")
                    .style(Style::default().fg(Color::DarkGray)),
                area,
            );
            return;
        }
    };

    let chunks = Layout::vertical([
        Constraint::Length(2), // header / class name
        Constraint::Min(1),    // subclass list
    ])
    .split(area);

    let class_name = &detail.class.name;
    let header = Paragraph::new(format!("  Subclasses for {}:", class_name)).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = detail
        .subclasses
        .iter()
        .map(|swf| {
            let s = &swf.subclass;
            let source = if s.source_slug.is_empty() {
                String::new()
            } else {
                format!(" [{}]", s.source_slug)
            };
            let unlock = format!(" (unlocks at level {})", s.unlock_level);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}", s.name),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(source, Style::default().fg(Color::Cyan)),
                Span::styled(unlock, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("  No subclasses found.").style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    } else {
        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" > ");
        let mut state = ListState::default().with_selected(Some(app.picker_selected));
        frame.render_stateful_widget(list, chunks[1], &mut state);
    }
}

fn render_list_picker(app: &mut App, frame: &mut Frame, area: Rect, is_items: bool) {
    let detail_open = is_items && app.show_item_detail;

    // Split horizontally into list pane and (optional) detail pane
    let panes = if detail_open {
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area)
    } else {
        Layout::horizontal([Constraint::Percentage(100)]).split(area)
    };
    let list_area = panes[0];

    let chunks = Layout::vertical([
        Constraint::Length(1), // search label
        Constraint::Length(1), // search input
        Constraint::Length(1), // spacer
        Constraint::Min(1),    // results
    ])
    .split(list_area);

    let label = Paragraph::new("  Search:").style(Style::default().fg(Color::White));
    frame.render_widget(label, chunks[0]);

    let input = Paragraph::new(format!("  {}▌", app.picker_search))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, chunks[1]);

    // Build list items
    let list_items: Vec<ListItem> = if is_items {
        app.filtered_items()
            .iter()
            .map(|item| {
                let itype = item.item_type.as_deref().unwrap_or("misc");
                ListItem::new(format!("{} ({itype})", item.name))
                    .style(Style::default().fg(Color::White))
            })
            .collect()
    } else {
        app.filtered_spells()
            .iter()
            .map(|spell| {
                let level_str = if spell.level == 0 {
                    "Cantrip".to_string()
                } else {
                    format!("Lvl {}", spell.level)
                };
                ListItem::new(format!("{} ({level_str})", app.spell_name(spell.id)))
                    .style(Style::default().fg(Color::White))
            })
            .collect()
    };

    if list_items.is_empty() {
        let no_results =
            Paragraph::new("   No results found").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_results, chunks[3]);
    } else {
        let list = List::new(list_items)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" > ");
        app.picker_list_state.select(Some(app.picker_selected));
        frame.render_stateful_widget(list, chunks[3], &mut app.picker_list_state);
    }

    // Detail panel
    if detail_open {
        let detail_area = panes[1];
        let filtered = app.filtered_items();
        let mut lines: Vec<Line> = Vec::new();

        if let Some(item) = filtered.get(app.picker_selected) {
            // Name
            lines.push(Line::from(vec![Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            // Type / Rarity / Magic
            let itype = item.item_type.as_deref().unwrap_or("misc");
            let rarity = item.rarity.as_deref().unwrap_or("none");
            let magic_tag = if item.is_magic.unwrap_or(false) {
                " ✦ Magic"
            } else {
                ""
            };
            lines.push(Line::from(format!(
                "Type: {}  |  Rarity: {}{}",
                itype, rarity, magic_tag
            )));

            // Weight / Value
            let weight_str = item
                .weight
                .as_deref()
                .map(|w| format!("{}lb", w))
                .unwrap_or_else(|| "—".to_string());
            let value_str = match item.value_cp {
                Some(cp) if cp >= 100 => format!("{} gp", cp / 100),
                Some(cp) if cp >= 10 => format!("{} sp", cp / 10),
                Some(cp) => format!("{} cp", cp),
                None => "—".to_string(),
            };
            lines.push(Line::from(format!(
                "Weight: {}  |  Value: {}",
                weight_str, value_str
            )));
            lines.push(Line::from(""));

            // Attunement / Equip notice
            if item.requires_attune.unwrap_or(false) {
                lines.push(Line::from(vec![Span::styled(
                    "⚠ Requires Attunement",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
            // Armour and weapons need to be equipped to be effective
            let needs_equip = matches!(itype, "HA" | "MA" | "LA" | "S" | "M" | "R" | "A" | "MNT");
            if needs_equip {
                lines.push(Line::from(vec![Span::styled(
                    "⚔ Must be equipped to use",
                    Style::default().fg(Color::Cyan),
                )]));
            }
            if item.requires_attune.unwrap_or(false) || needs_equip {
                lines.push(Line::from(""));
            }

            // Properties
            if let Some(props) = &item.properties {
                if !props.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        "Properties: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]));
                    lines.push(Line::from(format!("  {}", props.join(", "))));
                    lines.push(Line::from(""));
                }
            }

            // Description entries
            if let Some(entries) = &item.entries.clone() {
                if let Some(arr) = entries.as_array() {
                    lines.push(Line::from(vec![Span::styled(
                        "Description:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]));
                    for entry in arr {
                        let text = match entry {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        // Wrap long lines by word
                        lines.push(Line::from(format!(" {}", text)));
                    }
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  No item selected",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let detail = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::LEFT)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(detail, detail_area);
    }
}

fn render_action_detail_modal(name: &str, description: &str, frame: &mut Frame, area: Rect) {
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", name))
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    if description.is_empty() {
        lines.push(Line::from(Span::styled(
            "No description available.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for paragraph in description.split('\n') {
            if paragraph.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(Span::styled(
                    paragraph.to_string(),
                    Style::default().fg(Color::White),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press any key to close",
        Style::default().fg(Color::DarkGray),
    )));

    let content = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(content, inner);
}
