use crate::app::App;
use crate::models::app_state::{BuilderState, Screen, CharacterCreationStep};
use crate::utils::ABILITY_NAMES;
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

    // 1) Layout
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Final Summary ")
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

    // Left Column
    let mut left_lines = Vec::new();
    left_lines.push(Line::from(vec![Span::styled(
        &app.builder.name,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    left_lines.push(Line::from(""));

    let race_name = app
        .builder
        .race_id
        .and_then(|id| {
            app.races
                .iter()
                .find(|r| r.id == id)
                .map(|r| r.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    left_lines.push(Line::from(format!("Race:       {}", race_name)));

    let class_name = app
        .builder
        .class_id
        .and_then(|id| {
            app.classes
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    left_lines.push(Line::from(format!("Class:      {} (Level 1)", class_name)));

    let bg_name = app
        .builder
        .bg_id
        .and_then(|id| {
            app.backgrounds
                .iter()
                .find(|b| b.id == id)
                .map(|b| b.name.clone())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    left_lines.push(Line::from(format!("Background: {}", bg_name)));
    if !app.builder.skill_choices.is_empty() {
        left_lines.push(Line::from(format!(
            "Skills Picked: {}",
            app.builder.skill_choices.join(", ")
        )));
    }
    left_lines.push(Line::from(""));
    left_lines.push(Line::from(Span::styled(
        "Ability Scores:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for i in 0..6 {
        let base = app.builder.abilities[i];
        let bg = app.builder.bg_ability_bonuses[i];
        let total = base + bg;
        let modifier = (total - 10) / 2;
        let sign = if modifier >= 0 { "+" } else { "" };
        let bg_str = if bg > 0 {
            format!(" (BG: +{})", bg)
        } else {
            String::new()
        };
        left_lines.push(Line::from(format!(
            "  {}: {:<2}{} [{}{}]",
            ABILITY_NAMES[i], total, bg_str, sign, modifier
        )));
    }

    // Derived statistics calculation
    let dex_mod = (app.builder.abilities[1] + app.builder.bg_ability_bonuses[1] - 10) / 2;
    let con_mod = (app.builder.abilities[2] + app.builder.bg_ability_bonuses[2] - 10) / 2;
    let mut hp_max = 0;

    if let Some(c_id) = app.builder.class_id {
        if let Some(c) = app.classes.iter().find(|x| x.id == c_id) {
            hp_max = c.hit_die as i32 + con_mod;
        }
    }

    left_lines.push(Line::from(""));
    left_lines.push(Line::from(Span::styled(
        "Combat Stats:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    left_lines.push(Line::from(format!("  Hit Points: {}", hp_max)));
    left_lines.push(Line::from(format!("  Initiative: {:+}", dex_mod)));
    left_lines.push(Line::from(format!("  Armor Class: {}", 10 + dex_mod))); // base barebones AC

    if app.builder.spellcasting_type != "none" {
        left_lines.push(Line::from(""));
        left_lines.push(Line::from(Span::styled(
            "Spellcasting:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        left_lines.push(Line::from(format!(
            "  Type: {}",
            app.builder.spellcasting_type
        )));
    }

    let left_p = Paragraph::new(left_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Core Identity & Combat "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(left_p, body[0]);

    // Right Column
    let mut right_lines = Vec::new();

    right_lines.push(Line::from(Span::styled(
        "Physical Characteristics:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    right_lines.push(Line::from(format!("  Age:        {}", app.builder.age)));
    right_lines.push(Line::from(format!("  Height:     {}", app.builder.height)));
    right_lines.push(Line::from(format!("  Weight:     {}", app.builder.weight)));
    right_lines.push(Line::from(format!(
        "  Appearance: {}",
        app.builder.appearance
    )));
    right_lines.push(Line::from(""));

    right_lines.push(Line::from(Span::styled(
        "Roleplay:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    right_lines.push(Line::from(format!(
        "  Alignment:  {}",
        app.builder.alignment
    )));
    right_lines.push(Line::from(format!(
        "  Traits:     {}",
        app.builder.trait_text
    )));
    right_lines.push(Line::from(format!("  Ideals:     {}", app.builder.ideal)));
    right_lines.push(Line::from(format!("  Bonds:      {}", app.builder.bond)));
    right_lines.push(Line::from(format!("  Flaws:      {}", app.builder.flaw)));
    right_lines.push(Line::from(""));

    let equip_str = match app.builder.equipment_option {
        Some(0) => "Standard Class & Background Package",
        Some(1) => "Starting Gold (buy your own)",
        _ => "None selected",
    };
    right_lines.push(Line::from(Span::styled(
        "Equipment Choice:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    right_lines.push(Line::from(format!("  {}", equip_str)));

    let right_p = Paragraph::new(right_lines)
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: true });
    frame.render_widget(right_p, body[1]);

    // 3) Footer — show error/status if present, otherwise help
    let footer_text = if !app.status_msg.is_empty() {
        app.status_msg.clone()
    } else {
        "Enter confirm & save   Esc back to details   Q quit to list".to_string()
    };
    let footer_color = if !app.status_msg.is_empty() {
        Color::Red
    } else {
        Color::Yellow
    };
    let help = Paragraph::new(footer_text).style(
        Style::default()
            .fg(footer_color)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Details;
            app.status_msg.clear();
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            // Cancel completely
            app.builder = BuilderState::default();
            app.screen = Screen::CharacterList;
            app.status_msg.clear();
        }
        KeyCode::Enter => {
            crate::handlers::builder::submit_character_from_builder(app);
        }
        _ => {}
    }
}
