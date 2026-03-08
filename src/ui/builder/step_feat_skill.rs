/// Builder step: pick skill proficiencies granted by the pending feat.
///
/// Driven by `BuilderState::builder_pending_feature = Feature::SkillChoice { choose }`.
/// On confirm, the chosen skill names are appended to `BuilderState::feat_skill_choices`
/// and the step advances to `Summary`.
use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crate::models::features::Feature;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// All standard D&D 5e (2024) skills.
pub const ALL_SKILLS: &[&str] = &[
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

fn choose_count(app: &App) -> u8 {
    match &app.builder.builder_pending_feature {
        Some(Feature::SkillChoice { choose }) => *choose,
        _ => 1,
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let choose = choose_count(app);

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let title_text = format!(
        " Character Builder: Choose {} Skill{} (Feat) ",
        choose,
        if choose == 1 { "" } else { "s" }
    );
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // ── Skill list ────────────────────────────────────────────────────────────
    let search = app.builder.feat_picker_search.to_lowercase();
    let filtered: Vec<&str> = ALL_SKILLS
        .iter()
        .copied()
        .filter(|s| search.is_empty() || s.to_lowercase().contains(&search))
        .collect();

    let body = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);

    let search_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(body[0]);

    frame.render_widget(
        Paragraph::new("  Search:").style(Style::default().fg(Color::DarkGray)),
        search_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(format!("  {}▌", app.builder.feat_picker_search))
            .style(Style::default().fg(Color::Yellow)),
        search_chunks[1],
    );

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let chosen = app
                .builder
                .feat_skill_choices
                .iter()
                .any(|c| c.as_str() == *s);
            let current = i == app.builder.feat_picker_index;
            let prefix = if chosen { "✓ " } else { "  " };
            let style = if current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if chosen {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{}", prefix, s)).style(style)
        })
        .collect();

    use ratatui::widgets::ListState;
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Skills ({}/{} chosen) ",
            app.builder.feat_skill_choices.len(),
            choose
        )))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default().with_selected(Some(app.builder.feat_picker_index));
    frame.render_stateful_widget(list, search_chunks[2], &mut state);

    // ── Chosen list (right panel) ─────────────────────────────────────────────
    let chosen_items: Vec<ListItem> = app
        .builder
        .feat_skill_choices
        .iter()
        .map(|s| ListItem::new(format!("  • {}", s)).style(Style::default().fg(Color::Green)))
        .collect();
    let chosen_list = List::new(chosen_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Chosen Skills "),
    );
    frame.render_widget(chosen_list, body[1]);

    let chosen_count = app.builder.feat_skill_choices.len() as u8;
    let help = if chosen_count >= choose {
        "Enter confirm  Esc cancel".to_string()
    } else {
        format!(
            "Space toggle  Enter confirm  ↑↓ navigate  ({}/{} chosen)",
            chosen_count, choose
        )
    };
    let help_w = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_w, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let choose = choose_count(app) as usize;
    let search = app.builder.feat_picker_search.to_lowercase();
    let filtered: Vec<&str> = ALL_SKILLS
        .iter()
        .copied()
        .filter(|s| search.is_empty() || s.to_lowercase().contains(&search))
        .collect();

    match key.code {
        KeyCode::Esc => {
            app.builder.feat_skill_choices.clear();
            app.builder.builder_pending_feature = None;
            app.builder.feat_picker_search.clear();
            app.builder.feat_picker_index = 0;
            app.builder.step = CharacterCreationStep::Summary;
        }
        KeyCode::Up => {
            if app.builder.feat_picker_index > 0 {
                app.builder.feat_picker_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.builder.feat_picker_index + 1 < filtered.len() {
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
        KeyCode::Char(' ') | KeyCode::Enter => {
            let search = app.builder.feat_picker_search.to_lowercase();
            let filtered: Vec<&str> = ALL_SKILLS
                .iter()
                .copied()
                .filter(|s| search.is_empty() || s.to_lowercase().contains(&search))
                .collect();

            if let Some(&skill) = filtered.get(app.builder.feat_picker_index) {
                let already = app
                    .builder
                    .feat_skill_choices
                    .iter()
                    .position(|c| c.as_str() == skill);
                if let Some(idx) = already {
                    app.builder.feat_skill_choices.remove(idx);
                } else if app.builder.feat_skill_choices.len() < choose {
                    app.builder.feat_skill_choices.push(skill.to_string());
                    app.status_msg = format!("Skill: {} selected", skill);
                }
            }

            if app.builder.feat_skill_choices.len() >= choose {
                app.builder.builder_pending_feature = None;
                app.builder.feat_picker_search.clear();
                app.builder.feat_picker_index = 0;
                app.builder.step = CharacterCreationStep::Summary;
                app.status_msg = format!(
                    "Feat skills confirmed: {}",
                    app.builder.feat_skill_choices.join(", ")
                );
            }
        }
        _ => {}
    }
}
