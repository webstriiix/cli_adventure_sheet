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

    let level = crate::models::rules::level_from_xp(character.experience_pts);
    let prof_bonus = crate::models::rules::proficiency_bonus(level);



    let header = Row::new(vec!["", "  Skill", "Ability", "Modifier"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = SKILLS
        .iter()
        .map(|(skill, ability, _idx)| {
            let (proficient, expert, total_mod) = character.get_skill_modifier(
                skill,
                ability,
                &app.char_proficiencies,
                &app.char_chosen_skills,
                &app.char_expertise_skills,
            );

            let dot = if expert {
                Span::styled("◆ ", Style::default().fg(Color::Cyan))
            } else if proficient {
                Span::styled("● ", Style::default().fg(Color::Green))
            } else {
                Span::styled("○ ", Style::default().fg(Color::DarkGray))
            };

            let is_selected = app.sidebar_focused == false && app.selected_list_index == SKILLS.iter().position(|(s, _, _)| *s == *skill).unwrap_or(99);
            
            let mut name_style = if expert {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if proficient {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let mut mod_style = if expert {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if proficient {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            if is_selected {
                name_style = name_style.bg(Color::Rgb(50, 50, 80));
                mod_style = mod_style.bg(Color::Rgb(50, 50, 80));
            }

            Row::new(vec![
                ratatui::widgets::Cell::from(Line::from(dot)),
                ratatui::widgets::Cell::from(format!("  {skill}")).style(name_style),
                ratatui::widgets::Cell::from(ability.to_string()).style(Style::default().fg(Color::DarkGray)),
                ratatui::widgets::Cell::from(crate::models::rules::format_modifier(total_mod)).style(mod_style),
            ])
            .style(if is_selected { Style::default().bg(Color::Rgb(50, 50, 80)) } else { Style::default() })
        })
        .collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(20),
        Constraint::Length(8),
        Constraint::Length(8),
    ];

    // Proficiency bonus info line
    let mut info_spans = vec![
        Span::styled("  Prof Bonus: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            crate::models::rules::format_modifier(prof_bonus),
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
    ];

    if !app.sidebar_focused {
        info_spans.push(Span::styled(
            "   [Enter/Space] to toggle proficiency",
            Style::default().fg(Color::Yellow),
        ));
    }

    let info = Paragraph::new(Line::from(info_spans));

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
