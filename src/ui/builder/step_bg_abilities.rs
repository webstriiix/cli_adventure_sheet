use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
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

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let bg_name = app
        .builder
        .bg_id
        .and_then(|id| {
            app.backgrounds
                .iter()
                .find(|b| b.id == id)
                .map(|b| b.name.clone())
        })
        .unwrap_or_else(|| "Background".to_string());

    let title = Paragraph::new(format!(
        " Character Builder: {} — Ability Score Improvement ",
        bg_name
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let a = app.builder.bg_ability_focus;  // +2 target
    let b = app.builder.bg_ability_step as usize; // +1 target (reusing bg_ability_step as index)

    let lines = vec![
        Line::from(Span::styled(
            "  Background Ability Score Improvement",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "  +2 to: {} (Tab)    +1 to: {} (Shift+Tab)",
            ABILITY_NAMES[a], ABILITY_NAMES[b]
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Enter to confirm",
            Style::default().fg(Color::Green),
        )),
    ];

    let body_p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Background Ability Bonuses "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(body_p, outer[1]);

    let help = Paragraph::new("Tab cycle +2   Shift+Tab cycle +1   Enter confirm   Esc back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.bg_ability_bonuses = [0; 6];
            app.builder.bg_ability_step = 0;
            app.builder.bg_ability_focus = 0;
            let idx = app
                .builder
                .bg_id
                .and_then(|id| app.backgrounds.iter().position(|b| b.id == id))
                .unwrap_or(0);
            app.builder.list_state.select(Some(idx));
            app.builder.step = CharacterCreationStep::Background;
        }
        KeyCode::BackTab => {
            // Cycle +1 ability
            let next = (app.builder.bg_ability_step as usize + 1) % 6;
            app.builder.bg_ability_step = next as u8;
        }
        KeyCode::Tab => {
            // Cycle +2 ability
            app.builder.bg_ability_focus = (app.builder.bg_ability_focus + 1) % 6;
        }
        KeyCode::Enter => {
            // Apply the +2/+1 and advance
            let a = app.builder.bg_ability_focus;
            let b = app.builder.bg_ability_step as usize;
            app.builder.bg_ability_bonuses = [0; 6];
            app.builder.bg_ability_bonuses[a] += 2;
            app.builder.bg_ability_bonuses[b] += 1;
            advance_from_bg_abilities(app);
        }
        _ => {}
    }
}

fn advance_from_bg_abilities(app: &mut App) {
    let bg = app
        .builder
        .bg_id
        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id));

    let grants_feat = bg.map(|b| b.grants_bonus_feat).unwrap_or(false);

    app.builder.feat_picker_search.clear();
    app.builder.feat_picker_index = 0;

    if grants_feat {
        app.builder.step = CharacterCreationStep::BackgroundFeat;
    } else if app.builder.language_count > 0 {
        app.builder.step = CharacterCreationStep::Languages;
    } else {
        app.builder.step = CharacterCreationStep::Proficiencies;
    }
    app.builder.list_state.select(Some(0));
}
