use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use serde_json::Value as JsonValue;

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let _character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    // Find the background data
    let bg = app.backgrounds.iter().find(|b| b.name == app.char_bg_name);

    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Length(2), // name
        Constraint::Length(1), // spacer
        Constraint::Length(1), // skills header
        Constraint::Length(2), // skills
        Constraint::Length(1), // spacer
        Constraint::Length(1), // tools header
        Constraint::Length(2), // tools
        Constraint::Length(1), // spacer
        Constraint::Length(1), // languages header
        Constraint::Min(2),    // languages
    ])
    .split(area);

    // ── Background Name ──
    let header = Paragraph::new(Span::styled(
        "  BACKGROUND",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(header, chunks[0]);

    let name = Paragraph::new(Line::from(vec![Span::styled(
        format!("  {}", app.char_bg_name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )]));
    frame.render_widget(name, chunks[1]);

    // ── Skill Proficiencies ──
    let skills_header = Paragraph::new(Span::styled(
        "  SKILL PROFICIENCIES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(skills_header, chunks[3]);

    let skills_text = bg
        .map(|b| {
            let names = extract_proficiency_names(b.skill_proficiencies.as_deref());
            if names.is_empty() {
                "  None".to_string()
            } else {
                format!("  {}", names.join(", "))
            }
        })
        .unwrap_or_else(|| "  Unknown".to_string());

    let skills = Paragraph::new(skills_text).style(Style::default().fg(Color::White));
    frame.render_widget(skills, chunks[4]);

    // ── Tool Proficiencies ──
    let tools_header = Paragraph::new(Span::styled(
        "  TOOL PROFICIENCIES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(tools_header, chunks[6]);

    let tools_text = bg
        .map(|b| {
            let names = extract_proficiency_names(b.tool_proficiencies.as_deref());
            if names.is_empty() {
                "  None".to_string()
            } else {
                format!("  {}", names.join(", "))
            }
        })
        .unwrap_or_else(|| "  Unknown".to_string());

    let tools = Paragraph::new(tools_text).style(Style::default().fg(Color::White));
    frame.render_widget(tools, chunks[7]);

    // ── Languages ──
    let lang_header = Paragraph::new(Span::styled(
        "  LANGUAGES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(lang_header, chunks[9]);

    let lang_text = bg
        .map(|b| format!("  {} additional language(s)", b.language_count.unwrap_or(0)))
        .unwrap_or_else(|| "  Unknown".to_string());

    let languages = Paragraph::new(lang_text).style(Style::default().fg(Color::White));
    frame.render_widget(languages, chunks[10]);
}

/// Extract proficiency names from a `Vec<JsonValue>` where each element is either:
/// - a plain string: `"history"`
/// - an object with skill keys set to `true`: `{"history": true, "intimidation": true}`
fn extract_proficiency_names(entries: Option<&[JsonValue]>) -> Vec<String> {
    let entries = match entries {
        Some(e) if !e.is_empty() => e,
        _ => return Vec::new(),
    };
    let mut names = Vec::new();
    for entry in entries {
        if let Some(s) = entry.as_str() {
            let mut name = s.to_string();
            // title-case first letter
            if let Some(c) = name.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            names.push(name);
        } else if let Some(obj) = entry.as_object() {
            for (key, val) in obj {
                if val.as_bool().unwrap_or(false) {
                    let mut name = key.clone();
                    if let Some(c) = name.get_mut(0..1) {
                        c.make_ascii_uppercase();
                    }
                    names.push(name);
                }
            }
        }
    }
    names
}
