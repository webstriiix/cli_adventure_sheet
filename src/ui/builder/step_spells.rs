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
    info_lines.push(Line::from(""));
    info_lines.push(Line::from(Span::styled(
        "Note:",
        Style::default().fg(Color::Yellow),
    )));
    info_lines.push(Line::from(
        "Manage your detailed Spellbook and Prepared Spells directly",
    ));
    info_lines.push(Line::from(
        "from your Character Sheet after finishing the builder.",
    ));

    let info_p = Paragraph::new(info_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Spellcasting Details "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(info_p, body[0]);

    // Spell Picker visual placeholder
    let mut picker_lines = Vec::new();
    picker_lines.push(Line::from(
        "Level 1 spells will be added to your sheet soon.",
    ));
    picker_lines.push(Line::from(""));
    picker_lines.push(Line::from(Span::styled(
        "Press [Enter] to continue to Details.",
        Style::default().add_modifier(Modifier::BOLD),
    )));

    let picker_p = Paragraph::new(picker_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Initial Spells "),
        )
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true });

    frame.render_widget(picker_p, body[1]);

    let help = Paragraph::new("Enter continue step   Esc back to equipment")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Equipment;
            app.builder
                .list_state
                .select(Some(app.builder.equipment_option.unwrap_or(0)));
        }
        KeyCode::Enter => {
            app.builder.step = CharacterCreationStep::Details;
            app.builder.focus_index = 0; // First text field
            app.status_msg.clear();
        }
        _ => {}
    }
}
