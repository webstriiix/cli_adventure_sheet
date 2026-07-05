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

    let step = app.builder.bg_ability_step;
    let choice = app.builder.bg_ability_choices.get(step);

    let title_text = format!(
        " Character Builder: {} — Ability Score Improvement ({}/{}) ",
        bg_name,
        step + 1,
        app.builder.bg_ability_choices.len()
    );
    let title = Paragraph::new(title_text)
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let mut lines = vec![
        Line::from(Span::styled(
            "  Select Ability Bonus",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if let Some(c) = choice {
        for (i, opt) in c.options.iter().enumerate() {
            let weight = c.weights[i];
            let style = if i == app.builder.bg_ability_focus {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            
            lines.push(Line::from(Span::styled(
                format!("  [{}] +{} to {}", if i == app.builder.bg_ability_focus { "*" } else { " " }, weight, opt.to_uppercase()),
                style
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press Enter to confirm",
        Style::default().fg(Color::Green),
    )));

    let body_p = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Background Ability Bonuses "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(body_p, outer[1]);

    let help = Paragraph::new("Tab cycle options   Enter confirm   Esc back")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.bg_ability_bonuses = [0; 6];
            app.builder.bg_ability_choices = Vec::new();
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
        KeyCode::Tab => {
            if let Some(choice) = app.builder.bg_ability_choices.get(app.builder.bg_ability_step) {
                app.builder.bg_ability_focus = (app.builder.bg_ability_focus + 1) % choice.options.len();
            }
        }
        KeyCode::Enter => {
            let step = app.builder.bg_ability_step;
            if let Some(choice) = app.builder.bg_ability_choices.get_mut(step) {
                choice.selected_idx = Some(app.builder.bg_ability_focus);
                
                // Apply bonus (map string to index)
                let ability = &choice.options[app.builder.bg_ability_focus];
                let weight = choice.weights[app.builder.bg_ability_focus];
                
                let ability_idx = match ability.as_str() {
                    "str" => 0, "dex" => 1, "con" => 2, "int" => 3, "wis" => 4, "cha" => 5,
                    _ => 0,
                };
                app.builder.bg_ability_bonuses[ability_idx] += weight;
            }
            
            if step + 1 < app.builder.bg_ability_choices.len() {
                app.builder.bg_ability_step += 1;
                app.builder.bg_ability_focus = 0;
            } else {
                advance_from_bg_abilities(app);
            }
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
