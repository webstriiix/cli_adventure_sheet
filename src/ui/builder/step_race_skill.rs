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

/// All skills available for Skillful proficiency pick.
const ALL_SKILLS: &[&str] = &[
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

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Human – Skillful ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    // Already-chosen skills (background + class)
    let already: Vec<String> = app
        .builder
        .skill_choices
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let picked = app
        .builder
        .race_skill_choice
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    let items: Vec<ListItem> = ALL_SKILLS
        .iter()
        .map(|skill| {
            let lower = skill.to_lowercase();
            let (marker, color) = if lower == picked {
                ("✓ ", Color::Green)
            } else if already.contains(&lower) {
                ("• ", Color::DarkGray) // already have it, but can still pick
            } else {
                ("  ", Color::White)
            };
            ListItem::new(format!("{}{}", marker, skill)).style(Style::default().fg(color))
        })
        .collect();

    use ratatui::widgets::ListState;
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Skillful – Bonus Skill Proficiency "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default().with_selected(Some(
        app.builder
            .feat_picker_index
            .min(ALL_SKILLS.len().saturating_sub(1)),
    ));
    frame.render_stateful_widget(list, body[0], &mut state);

    // Right panel: explain the trait
    let desc_lines = vec![
        Line::from(Span::styled(
            "Skillful",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("You gain proficiency in one skill of your choice."),
        Line::from(""),
        Line::from(Span::styled(
            "This is a trait of the Human species (XPHB).",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        if !picked.is_empty() {
            Line::from(Span::styled(
                format!(
                    "Selected: {}",
                    app.builder.race_skill_choice.as_deref().unwrap_or("")
                ),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                "No skill chosen yet.",
                Style::default().fg(Color::DarkGray),
            ))
        },
    ];
    let detail = Paragraph::new(desc_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Trait Details "),
    );
    frame.render_widget(detail, body[1]);

    let help = Paragraph::new("↑↓ select   Enter confirm   Tab skip   Esc back to race")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let count = ALL_SKILLS.len();
    match key.code {
        KeyCode::Esc => {
            // Back to race selection
            let idx = app
                .builder
                .race_id
                .and_then(|id| app.races.iter().position(|r| r.id == id))
                .unwrap_or(0);
            app.builder.list_state.select(Some(idx));
            app.builder.feat_picker_index = 0;
            app.builder.step = CharacterCreationStep::Race;
        }
        KeyCode::Up => {
            if app.builder.feat_picker_index > 0 {
                app.builder.feat_picker_index -= 1;
            } else {
                app.builder.feat_picker_index = count.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if app.builder.feat_picker_index + 1 < count {
                app.builder.feat_picker_index += 1;
            } else {
                app.builder.feat_picker_index = 0;
            }
        }
        KeyCode::Tab => {
            // Skip – no bonus skill
            app.builder.race_skill_choice = None;
            app.builder.feat_picker_index = 0;
            app.builder.step = CharacterCreationStep::RaceFeat;
        }
        KeyCode::Enter => {
            let chosen = ALL_SKILLS
                .get(app.builder.feat_picker_index)
                .map(|s| s.to_string());
            app.builder.race_skill_choice = chosen;
            app.builder.feat_picker_index = 0;
            app.builder.step = CharacterCreationStep::RaceFeat;
            app.status_msg = "Skill proficiency chosen.".to_string();
        }
        _ => {}
    }
}
