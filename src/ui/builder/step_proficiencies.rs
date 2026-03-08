use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

const ALL_SKILLS: [&str; 18] = [
    "Acrobatics",
    "Animal Handling",
    "Arcana",
    "Athletics",
    "Deception",
    "History",
    "Insight",
    "Intimidation",
    "Investigation",
    "Medicine",
    "Nature",
    "Perception",
    "Performance",
    "Persuasion",
    "Religion",
    "Sleight of Hand",
    "Stealth",
    "Survival",
];

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // 1) Layout
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 5 - Proficiencies ")
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

    // Determine allowed skills to pick and the count
    let mut allowed_skills = Vec::new();
    let mut pick_count = 0;

    if let Some(c_id) = app.builder.class_id {
        if let Some(class) = app.classes.iter().find(|c| c.id == c_id) {
            if let Some(arr) = class.skill_choices.as_array() {
                if let Some(first) = arr.first() {
                    if let Some(choose) = first.get("choose") {
                        pick_count =
                            choose.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
                        if let Some(from) = choose.get("from").and_then(|v| v.as_array()) {
                            for val in from {
                                if let Some(s) = val.as_str() {
                                    if s.to_lowercase() == "any" {
                                        allowed_skills =
                                            ALL_SKILLS.iter().map(|&sk| sk.to_string()).collect();
                                    } else {
                                        // capitalize first letter for display
                                        let cap = s
                                            .to_string()
                                            .split_whitespace()
                                            .map(|w| {
                                                let mut c = w.chars();
                                                match c.next() {
                                                    None => String::new(),
                                                    Some(f) => f.to_uppercase().chain(c).collect(),
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" ");
                                        allowed_skills.push(cap);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // if no choices (e.g. backend missing data), fallback
    if allowed_skills.is_empty() {
        pick_count = 0;
    } else {
        allowed_skills.sort();
    }

    let current_picks = app.builder.skill_choices.len();
    let header = format!(
        " Choose {} Skills ({}/{}) ",
        pick_count, current_picks, pick_count
    );

    let mut items = Vec::new();
    for skill in &allowed_skills {
        let is_selected = app.builder.skill_choices.contains(skill);
        let prefix = if is_selected { "[X]" } else { "[ ]" };

        let style = if is_selected {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", prefix), style),
            Span::styled(skill.clone(), style),
        ])));
    }

    if allowed_skills.is_empty() {
        items.push(ListItem::new("No skill choices required for this class."));
        app.builder.list_state.select(None);
    } else if app.builder.list_state.selected().is_none() {
        app.builder.list_state.select(Some(0));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(header))
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // Right Side: Info
    let mut right_lines = Vec::new();
    right_lines.push(Line::from(""));
    right_lines.push(Line::from(Span::styled(
        "Class Skills",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Yellow),
    )));
    right_lines.push(Line::from(""));
    right_lines.push(Line::from(
        "Your class grants you proficiency in a select number of skills.",
    ));
    if pick_count > 0 {
        right_lines.push(Line::from(format!(
            "You must select exactly {}.",
            pick_count
        )));
    } else {
        right_lines.push(Line::from("You have no choices to make here."));
    }
    right_lines.push(Line::from(""));

    // Warn if they try to proceed without picking enough
    if !app.status_msg.is_empty() {
        right_lines.push(Line::from(Span::styled(
            app.status_msg.clone(),
            Style::default().fg(Color::Red),
        )));
    }

    let info_p =
        Paragraph::new(right_lines).block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(info_p, body[1]);

    // 3) Footer
    let help = Paragraph::new("↑↓ select   Space toggle   Enter confirm   Esc back to background")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // We duplicate the allowed skills resolution logic here to know what we're toggling
    let mut allowed_skills = Vec::new();
    let mut pick_count = 0;

    if let Some(c_id) = app.builder.class_id {
        if let Some(class) = app.classes.iter().find(|c| c.id == c_id) {
            if let Some(arr) = class.skill_choices.as_array() {
                if let Some(first) = arr.first() {
                    if let Some(choose) = first.get("choose") {
                        pick_count =
                            choose.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as usize;
                        if let Some(from) = choose.get("from").and_then(|v| v.as_array()) {
                            for val in from {
                                if let Some(s) = val.as_str() {
                                    if s.to_lowercase() == "any" {
                                        allowed_skills =
                                            ALL_SKILLS.iter().map(|&sk| sk.to_string()).collect();
                                    } else {
                                        // capitalize first letter for display
                                        let cap = s
                                            .to_string()
                                            .split_whitespace()
                                            .map(|w| {
                                                let mut c = w.chars();
                                                match c.next() {
                                                    None => String::new(),
                                                    Some(f) => f.to_uppercase().chain(c).collect(),
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" ");
                                        allowed_skills.push(cap);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !allowed_skills.is_empty() {
        allowed_skills.sort();
    } else {
        pick_count = 0;
    }

    match key.code {
        KeyCode::Esc => {
            if app.builder.language_count > 0 {
                app.builder.step = CharacterCreationStep::Languages;
            } else {
                app.builder.step = CharacterCreationStep::Background;
            }
            app.status_msg.clear();
            let idx = app
                .builder
                .bg_id
                .and_then(|id| app.backgrounds.iter().position(|r| r.id == id))
                .unwrap_or(0);
            app.builder.list_state.select(Some(idx));
        }
        KeyCode::Up => {
            if !allowed_skills.is_empty() {
                let i = match app.builder.list_state.selected() {
                    Some(i) => {
                        if i > 0 {
                            i - 1
                        } else {
                            allowed_skills.len().saturating_sub(1)
                        }
                    }
                    None => 0,
                };
                app.builder.list_state.select(Some(i));
            }
        }
        KeyCode::Down => {
            if !allowed_skills.is_empty() {
                let i = match app.builder.list_state.selected() {
                    Some(i) => {
                        if i + 1 < allowed_skills.len() {
                            i + 1
                        } else {
                            0
                        }
                    }
                    None => 0,
                };
                app.builder.list_state.select(Some(i));
            }
        }
        KeyCode::Char(' ') => {
            if !allowed_skills.is_empty() {
                if let Some(idx) = app.builder.list_state.selected() {
                    if let Some(skill) = allowed_skills.get(idx) {
                        app.status_msg.clear();
                        if app.builder.skill_choices.contains(skill) {
                            app.builder.skill_choices.retain(|s| s != skill);
                        } else {
                            if app.builder.skill_choices.len() < pick_count {
                                app.builder.skill_choices.push(skill.clone());
                            } else {
                                app.status_msg.push_str(
                                    "You have reached the maximum number of skill choices.",
                                );
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Enter => {
            if app.builder.skill_choices.len() == pick_count {
                app.status_msg.clear();
                app.builder.step = CharacterCreationStep::Equipment;
                app.builder.list_state.select(Some(0));
            } else {
                app.status_msg = format!("You must select exactly {} skills.", pick_count);
            }
        }
        _ => {}
    }
}
