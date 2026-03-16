use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 7 - Spells ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let body = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    let mut is_caster = false;
    let mut spell_ability = String::new();

    if let Some(c_id) = app.builder.class_id {
        if let Some(class) = app.classes.iter().find(|c| c.id == c_id) {
            if let Some(ability) = &class.spellcasting_ability {
                is_caster = true;
                spell_ability = ability.clone();
            }
        }
    }

    if !is_caster {
        let msg = Paragraph::new(vec![
            Line::from("Your selected class does not have the Spellcasting feature at Level 1."),
            Line::from(""),
            Line::from("Press [Enter] to skip to the next step."),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Spells "))
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });
        frame.render_widget(msg, body[0]);

        let help = Paragraph::new("Enter confirm   Esc back to equipment")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, outer[2]);
        return;
    }

    // Determine Spell Save DC and Attack Mod
    // Proficiency Bonus at level 1 is +2
    let prof_bonus = 2;

    // determine index of casting ability
    let ability_idx = match spell_ability.to_lowercase().as_str() {
        "str" => 0,
        "dex" => 1,
        "con" => 2,
        "int" => 3,
        "wis" => 4,
        "cha" => 5,
        _ => 3, // fallback to int
    };

    let score = app.builder.abilities[ability_idx];
    let modifier = crate::utils::ability_modifier(score);

    let spell_save_dc = 8 + prof_bonus + modifier;
    let spell_attack = prof_bonus + modifier;

    let mut info_lines = Vec::new();
    info_lines.push(Line::from(vec![
        Span::styled(
            "Spellcasting Ability: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(spell_ability.to_uppercase()),
    ]));
    info_lines.push(Line::from(vec![
        Span::styled(
            "Spell Save DC: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}", spell_save_dc)),
    ]));
    let sign = if spell_attack >= 0 { "+" } else { "" };
    info_lines.push(Line::from(vec![
        Span::styled(
            "Spell Attack Mod: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}{}", sign, spell_attack)),
    ]));
    
    // Show Prepared Limit for Paladins/Clerics/Druids/Wizards
    let class_name = app.builder.class_id.and_then(|id| app.classes.iter().find(|c| c.id == id).map(|c| c.name.clone())).unwrap_or_default();
    let max_prepared = crate::utils::max_prepared_spells(&class_name, 1, modifier);
    
    info_lines.push(Line::from(""));
    info_lines.push(Line::from(vec![
        Span::styled("Prepared Spells: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}/{}", app.builder.known_spells.len(), max_prepared), Style::default().fg(if app.builder.known_spells.len() == max_prepared as usize { Color::Green } else { Color::Yellow })),
    ]));

    let info_p = Paragraph::new(info_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Spellcasting Details "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(info_p, body[0]);

    // Spell Picker
    let search = app.builder.feat_picker_search.to_lowercase();
    let always_prepared = app.always_prepared_spell_ids();
    
    // Filter spells: Level 1 and matching class
    let available_spells: Vec<&crate::models::compendium::Spell> = app.all_spells.iter().filter(|s| {
        if s.level != 1 { return false; }
        
        // Exclude if already always prepared by a feature (even if future level)
        if always_prepared.contains(&s.id) {
            return false;
        }

        // Filter by class name
        if let Some(classes) = &s.classes {
            // Check if ANY class entry matches our class name
            classes.iter().any(|c| {
                c.get("name").and_then(|v| v.as_str()).map(|n| n.to_lowercase() == class_name.to_lowercase()).unwrap_or(false)
            })
        } else {
            // Fallback: if no classes data, show everything? Or use hardcoded list?
            // For now, let's show everything to avoid blocking, but maybe prefix with "?"
            true
        }
    }).collect();

    let filtered: Vec<_> = available_spells.iter().filter(|s| {
        search.is_empty() || s.name.to_lowercase().contains(&search)
    }).collect();

    use ratatui::widgets::{List, ListItem, ListState};
    let items: Vec<ListItem> = filtered.iter().enumerate().map(|(i, s)| {
        let selected = app.builder.known_spells.contains(&s.id);
        let current = i == app.builder.feat_picker_index;
        
        let prefix = if selected { "✓ " } else { "  " };
        let style = if current {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if selected {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        
        ListItem::new(format!("{}{}", prefix, s.name)).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Level 1 Spells "))
        .highlight_symbol("> ");
    
    let mut state = ListState::default().with_selected(Some(app.builder.feat_picker_index));
    frame.render_stateful_widget(list, body[1], &mut state);
    
    // Help bar update
    let help_text = format!("Space toggle  Enter confirm ({}/{})", app.builder.known_spells.len(), max_prepared);
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let mut is_caster = false;
    if let Some(c_id) = app.builder.class_id {
        if let Some(class) = app.classes.iter().find(|c| c.id == c_id) {
            if class.spellcasting_ability.is_some() {
                is_caster = true;
            }
        }
    }

    if !is_caster {
        match key.code {
            KeyCode::Esc => {
                app.builder.step = CharacterCreationStep::Equipment;
            }
            KeyCode::Enter => {
                app.builder.step = CharacterCreationStep::Details;
                app.builder.focus_index = 0;
            }
            _ => {}
        }
        return;
    }

    // Filter spells logic (same as render)
    let search = app.builder.feat_picker_search.to_lowercase();
    let class_name = app.builder.class_id.and_then(|id| app.classes.iter().find(|c| c.id == id).map(|c| c.name.clone())).unwrap_or_default();
    let always_prepared = app.always_prepared_spell_ids();
    
    let available_spells: Vec<&crate::models::compendium::Spell> = app.all_spells.iter().filter(|s| {
        if s.level != 1 { return false; }
        if always_prepared.contains(&s.id) { return false; }
        if let Some(classes) = &s.classes {
            classes.iter().any(|c| {
                c.get("name").and_then(|v| v.as_str()).map(|n| n.to_lowercase() == class_name.to_lowercase()).unwrap_or(false)
            })
        } else {
            true
        }
    }).collect();

    let filtered: Vec<_> = available_spells.iter().filter(|s| {
        search.is_empty() || s.name.to_lowercase().contains(&search)
    }).collect();

    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Equipment;
            app.builder.known_spells.clear();
            app.builder.feat_picker_search.clear();
            app.builder.feat_picker_index = 0;
        }
        KeyCode::Up => {
            if app.builder.feat_picker_index > 0 {
                app.builder.feat_picker_index -= 1;
            }
        }
        KeyCode::Down => {
            if !filtered.is_empty() && app.builder.feat_picker_index + 1 < filtered.len() {
                app.builder.feat_picker_index += 1;
            }
        }
        KeyCode::Char(c) if c != ' ' => {
            app.builder.feat_picker_search.push(c);
            app.builder.feat_picker_index = 0;
        }
        KeyCode::Backspace => {
            app.builder.feat_picker_search.pop();
            app.builder.feat_picker_index = 0;
        }
        KeyCode::Char(' ') => {
            if let Some(spell) = filtered.get(app.builder.feat_picker_index) {
                if let Some(pos) = app.builder.known_spells.iter().position(|id| *id == spell.id) {
                    app.builder.known_spells.remove(pos);
                } else {
                    app.builder.known_spells.push(spell.id);
                }
            }
        }
        KeyCode::Enter => {
            app.builder.step = CharacterCreationStep::Details;
            app.builder.focus_index = 0;
            app.status_msg.clear();
        }
        _ => {}
    }
}
