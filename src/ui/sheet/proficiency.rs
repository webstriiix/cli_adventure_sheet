use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    let level = crate::utils::level_from_xp(character.experience_pts);
    let prof_bonus = crate::utils::proficiency_bonus(level);

    // Find the class data for this character
    let class = app.classes.iter().find(|c| c.name == app.char_class_name);

    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(2), // prof bonus
        Constraint::Length(1), // spacer
        Constraint::Length(1), // armor header
        Constraint::Length(2), // armor
        Constraint::Length(1), // spacer
        Constraint::Length(1), // weapons header
        Constraint::Length(2), // weapons
        Constraint::Length(1), // spacer
        Constraint::Length(1), // saves header
        Constraint::Min(2),    // saves
    ])
    .split(area);

    // ── Proficiency Bonus ──
    let header = Paragraph::new(Span::styled(
        "  PROFICIENCY BONUS",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(header, chunks[0]);

    let bonus_text = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            crate::utils::format_modifier(prof_bonus),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  (Level {level})"),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    frame.render_widget(bonus_text, chunks[1]);

    // ── Armor Proficiencies ──
    let armor_header = Paragraph::new(Span::styled(
        "  ARMOR PROFICIENCIES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(armor_header, chunks[3]);

    let armor_text = class
        .map(|c| {
            if let Some(proficiencies) = &c.armor_proficiencies {
                if proficiencies.is_empty() {
                    "  None".to_string()
                } else {
                    format!("  {}", proficiencies.join(", "))
                }
            } else {
                "  None".to_string()
            }
        })
        .unwrap_or_else(|| "  Unknown".to_string());

    let armor = Paragraph::new(armor_text).style(Style::default().fg(Color::White));
    frame.render_widget(armor, chunks[4]);

    // ── Weapon Proficiencies ──
    let weapon_header = Paragraph::new(Span::styled(
        "  WEAPON PROFICIENCIES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(weapon_header, chunks[6]);

    let weapon_text = class
        .map(|c| {
            if let Some(proficiencies) = &c.weapon_proficiencies {
                if proficiencies.is_empty() {
                    "  None".to_string()
                } else {
                    format!("  {}", proficiencies.join(", "))
                }
            } else {
                "  None".to_string()
            }
        })
        .unwrap_or_else(|| "  Unknown".to_string());

    let weapons = Paragraph::new(weapon_text).style(Style::default().fg(Color::White));
    frame.render_widget(weapons, chunks[7]);

    // ── Saving Throws ──
    let saves_header = Paragraph::new(Span::styled(
        "  SAVING THROW PROFICIENCIES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(saves_header, chunks[9]);

    let saves_text = class
        .map(|c| {
            if let Some(proficiencies) = &c.proficiency_saves {
                if proficiencies.is_empty() {
                    "  None".to_string()
                } else {
                    format!("  {}", proficiencies.join(", "))
                }
            } else {
                "  None".to_string()
            }
        })
        .unwrap_or_else(|| "  Unknown".to_string());

    let saves = Paragraph::new(saves_text).style(Style::default().fg(Color::Green));
    frame.render_widget(saves, chunks[10]);
}
