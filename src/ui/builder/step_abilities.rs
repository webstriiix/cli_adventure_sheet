use crate::app::App;
use crate::models::app_state::{AbilityMode, CharacterCreationStep};
use crate::utils::{ABILITY_NAMES, STANDARD_ARRAY};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

    let title = Paragraph::new(" Character Builder: Step 5 - Ability Scores ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // 2) Body
    let body = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    // Left side: Mode toggle + Available Array
    let left_vert = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Min(0),
    ])
    .split(body[0]);

    let mode_str = match app.builder.ability_mode {
        AbilityMode::StandardArray => "Mode: Standard Array",
        AbilityMode::Manual => "Mode: Manual Entry",
    };

    let mode_p = Paragraph::new(vec![
        Line::from(Span::styled(
            mode_str,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        )),
        Line::from("Press [Tab] to toggle between modes."),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Generation Method "),
    );
    frame.render_widget(mode_p, left_vert[0]);

    if app.builder.ability_mode == AbilityMode::StandardArray {
        let mut avail = Vec::new();
        for (i, &val) in STANDARD_ARRAY.iter().enumerate() {
            if app.builder.standard_pool[i] {
                avail.push(Span::styled(
                    format!(" {} ", val),
                    Style::default().fg(Color::Green),
                ));
            } else {
                avail.push(Span::styled(" -- ", Style::default().fg(Color::DarkGray)));
            }
        }

        let pool_p = Paragraph::new(Line::from(avail))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Available Scores "),
            );
        frame.render_widget(pool_p, left_vert[1]);

        let instructions = Paragraph::new(vec![
            Line::from("Select an ability on the right and press [Enter] to assign the highest available score from the array."),
            Line::from("Press [Backspace] on an assigned ability to return it to the pool."),
        ]).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, left_vert[2]);
    } else {
        let instructions = Paragraph::new(vec![
            Line::from("Select an ability on the right and use [Left/Right] or [-/+] to adjust the score manually."),
            Line::from("Consult your DM regarding Point Buy or Rolling rules."),
        ]).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, left_vert[1]);
    }

    // Right side: six ability scores + racial bonuses
    // We want to calculate the racial bonuses if a race was picked
    let mut racial_bonuses = [0; 6];
    if let Some(race_id) = app.builder.race_id {
        if let Some(race) = app.races.iter().find(|r| r.id == race_id) {
            for bonus in &race.ability_bonuses {
                if let Some(obj) = bonus.as_object() {
                    for (k, v) in obj {
                        if let Some(val) = v.as_i64() {
                            match k.to_lowercase().as_str() {
                                "str" => racial_bonuses[0] += val as i32,
                                "dex" => racial_bonuses[1] += val as i32,
                                "con" => racial_bonuses[2] += val as i32,
                                "int" => racial_bonuses[3] += val as i32,
                                "wis" => racial_bonuses[4] += val as i32,
                                "cha" => racial_bonuses[5] += val as i32,
                                "choose" => {} // Ignore any "choose 2" logic for a simple CLI implementation
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    let mut hp_preview = 0;
    let init_preview;

    let dex_idx = 1;
    let con_idx = 2;
    let str_or_default = app.builder.abilities[dex_idx] + racial_bonuses[dex_idx] + app.builder.bg_ability_bonuses[dex_idx];
    let con_or_default = app.builder.abilities[con_idx] + racial_bonuses[con_idx] + app.builder.bg_ability_bonuses[con_idx];

    let con_mod = if con_or_default > 0 {
        crate::utils::ability_modifier(con_or_default)
    } else {
        0
    };
    init_preview = if str_or_default > 0 {
        crate::utils::ability_modifier(str_or_default)
    } else {
        0
    };

    if let Some(c_id) = app.builder.class_id {
        if let Some(c) = app.classes.iter().find(|x| x.id == c_id) {
            hp_preview = c.hit_die as i32 + con_mod;
        }
    }

    let right_vert = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(4), // derived stats box
    ])
    .margin(1)
    .split(body[1]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Assign Scores ");
    frame.render_widget(block, body[1]);

    for i in 0..6 {
        let is_focused = app.builder.ability_focus == i;
        let base_val = app.builder.abilities[i];
        let racial = racial_bonuses[i];

        let style = if is_focused {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Format: STR   Base: 15 (Race: +1) = 16  [+3]
        let bg_bonus = app.builder.bg_ability_bonuses[i];
        let total = base_val + racial + bg_bonus;
        let modifier = (total - 10) / 2;

        let mut bonus_parts = Vec::new();
        if racial > 0 {
            bonus_parts.push(format!("Race: +{}", racial));
        }
        if bg_bonus > 0 {
            bonus_parts.push(format!("BG: +{}", bg_bonus));
        }
        let racial_str = if bonus_parts.is_empty() {
            "".to_string()
        } else {
            format!("({})", bonus_parts.join(", "))
        };
        let base_str = if base_val == 0 {
            "--".to_string()
        } else {
            base_val.to_string()
        };
        let sign = if modifier >= 0 { "+" } else { "" };

        let line = if base_val == 0 && app.builder.ability_mode == AbilityMode::StandardArray {
            format!("{:<4} Base: {}", ABILITY_NAMES[i], "Unassigned")
        } else {
            format!(
                "{:<4} Base: {:<2} {:<10} = {:<2}  [{}{}]",
                ABILITY_NAMES[i], base_str, racial_str, total, sign, modifier
            )
        };

        let p = Paragraph::new(line).style(style);
        frame.render_widget(p, right_vert[i]);
    }

    // Derived statistics preview
    let init_str = if init_preview >= 0 {
        format!("+{}", init_preview)
    } else {
        init_preview.to_string()
    };
    let spell_str = if app.builder.spellcasting_type != "none" {
        " | Save DC: TBD" // Could expand if desired
    } else {
        ""
    };

    let derived_p = Paragraph::new(vec![
        Line::from(format!("Hit Points (Level 1): {}", hp_preview)),
        Line::from(format!("Initiative: {}{}", init_str, spell_str)),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Class Previews "),
    )
    .style(Style::default().fg(Color::LightCyan));

    frame.render_widget(derived_p, right_vert[6]);

    // 5) Help
    let help =
        Paragraph::new("Tab toggle mode   ↑↓ select   Enter/Arrows assign   Esc back to class")
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            if app.builder.skip_subclass {
                app.builder.step = CharacterCreationStep::Class;
                let idx = app
                    .builder
                    .class_id
                    .and_then(|id| app.classes.iter().position(|r| r.id == id))
                    .unwrap_or(0);
                app.builder.list_state.select(Some(idx));
            } else {
                app.builder.step = CharacterCreationStep::Subclass;
            }
        }
        KeyCode::Tab => {
            app.builder.ability_mode = match app.builder.ability_mode {
                AbilityMode::StandardArray => {
                    app.builder.abilities = [10; 6];
                    AbilityMode::Manual
                }
                AbilityMode::Manual => {
                    app.builder.abilities = [0; 6];
                    app.builder.standard_pool = vec![true; 6]; // reset
                    AbilityMode::StandardArray
                }
            };
            app.builder.ability_focus = 0;
            app.status_msg.clear();
        }
        KeyCode::Up => {
            if app.builder.ability_focus > 0 {
                app.builder.ability_focus -= 1;
            } else {
                app.builder.ability_focus = 5;
            }
        }
        KeyCode::Down => {
            if app.builder.ability_focus < 5 {
                app.builder.ability_focus += 1;
            } else {
                app.builder.ability_focus = 0;
            }
        }
        KeyCode::Right | KeyCode::Char('+') => {
            if app.builder.ability_mode == AbilityMode::Manual
                && app.builder.abilities[app.builder.ability_focus] < 20
            {
                app.builder.abilities[app.builder.ability_focus] += 1;
            }
        }
        KeyCode::Left | KeyCode::Char('-') => {
            if app.builder.ability_mode == AbilityMode::Manual
                && app.builder.abilities[app.builder.ability_focus] > 3
            {
                app.builder.abilities[app.builder.ability_focus] -= 1;
            }
        }
        KeyCode::Enter => {
            if app.builder.ability_mode == AbilityMode::StandardArray {
                if app.builder.abilities[app.builder.ability_focus] != 0 {
                    if app.builder.all_abilities_set() {
                        app.status_msg.clear();
                        app.builder.step = CharacterCreationStep::Background;
                        app.builder.list_state.select(Some(0));
                    }
                } else if let Some(pool_idx) = app
                    .builder
                    .standard_pool
                    .iter()
                    .position(|&available| available)
                {
                    app.builder.abilities[app.builder.ability_focus] = STANDARD_ARRAY[pool_idx];
                    app.builder.standard_pool[pool_idx] = false;

                    if app.builder.all_abilities_set() {
                        app.status_msg.clear();
                        app.builder.step = CharacterCreationStep::Background;
                        app.builder.list_state.select(Some(0));
                    }
                }
            } else if app.builder.ability_mode == AbilityMode::Manual {
                app.status_msg.clear();
                app.builder.step = CharacterCreationStep::Background;
                app.builder.list_state.select(Some(0));
            }
        }
        KeyCode::Backspace => {
            if app.builder.ability_mode == AbilityMode::StandardArray
                && app.builder.abilities[app.builder.ability_focus] != 0
            {
                let val = app.builder.abilities[app.builder.ability_focus];
                for (i, &v) in STANDARD_ARRAY.iter().enumerate() {
                    if v == val && !app.builder.standard_pool[i] {
                        app.builder.standard_pool[i] = true;
                        break;
                    }
                }
                app.builder.abilities[app.builder.ability_focus] = 0;
            }
        }
        _ => {}
    }
}
