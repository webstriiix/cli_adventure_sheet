use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

use crate::app::App;

// (Skill name, Ability abbreviation, ability index into [STR,DEX,CON,INT,WIS,CHA])
const SKILLS: [(&str, &str, usize); 18] = [
    ("Acrobatics",     "DEX", 1),
    ("Animal Handling","WIS", 4),
    ("Arcana",         "INT", 3),
    ("Athletics",      "STR", 0),
    ("Deception",      "CHA", 5),
    ("History",        "INT", 3),
    ("Insight",        "WIS", 4),
    ("Intimidation",   "CHA", 5),
    ("Investigation",  "INT", 3),
    ("Medicine",       "WIS", 4),
    ("Nature",         "INT", 3),
    ("Perception",     "WIS", 4),
    ("Performance",    "CHA", 5),
    ("Persuasion",     "CHA", 5),
    ("Religion",       "INT", 3),
    ("Sleight of Hand","DEX", 1),
    ("Stealth",        "DEX", 1),
    ("Survival",       "WIS", 4),
];

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    let level = crate::utils::level_from_xp(character.experience_pts);
    let prof_bonus = crate::utils::proficiency_bonus(level);

    let scores = [
        character.strength,
        character.dexterity,
        character.constitution,
        character.intelligence,
        character.wisdom,
        character.charisma,
    ];

    let header = Row::new(vec!["", "  Skill", "Ability", "Modifier"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = SKILLS
        .iter()
        .map(|(skill, ability, idx)| {
            let base_mod = crate::utils::ability_modifier(scores[*idx]);
            let skill_lower = skill.to_lowercase();
            let proficient = app.has_skill_prof(&skill_lower);
            let expert = app.has_expertise(&skill_lower);
            let bonus = if expert {
                prof_bonus * 2
            } else if proficient {
                prof_bonus
            } else {
                0
            };
            let total_mod = base_mod + bonus;

            let dot = if expert {
                Span::styled("◆ ", Style::default().fg(Color::Cyan))
            } else if proficient {
                Span::styled("● ", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ ", Style::default().fg(Color::DarkGray))
            };

            let name_style = if expert {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if proficient {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let mod_style = if expert {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if proficient {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                ratatui::widgets::Cell::from(Line::from(dot)),
                ratatui::widgets::Cell::from(format!("  {skill}")).style(name_style),
                ratatui::widgets::Cell::from(ability.to_string()).style(Style::default().fg(Color::DarkGray)),
                ratatui::widgets::Cell::from(crate::utils::format_modifier(total_mod)).style(mod_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(20),
        Constraint::Length(8),
        Constraint::Length(8),
    ];

    // Proficiency bonus info line
    let info = Paragraph::new(Line::from(vec![
        Span::styled("  Prof Bonus: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            crate::utils::format_modifier(prof_bonus),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("   ● proficient (+{prof_bonus})"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("   ◆ expertise (+{})", prof_bonus * 2),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    // Split area: info line at top, table below
    let chunks = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(2),
        ratatui::layout::Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(info, chunks[0]);

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, chunks[1]);
}
