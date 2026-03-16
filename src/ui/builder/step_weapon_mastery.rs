/// Builder step: pick weapon masteries granted by the pending feat.
///
/// Driven by `BuilderState::builder_pending_feature = Feature::WeaponMastery { choose }`.
/// On confirm, the chosen weapon names are appended to `BuilderState::weapon_mastery_choices`
/// and the step advances to `Class` (the normal post-feat step).
use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crate::models::features::Feature;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// How many masteries the current pending feature requires.
fn choose_count(app: &App) -> u8 {
    match &app.builder.builder_pending_feature {
        Some(Feature::WeaponMastery { choose }) => *choose,
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
        " Character Builder: Choose {} Weapon Master{} ",
        choose,
        if choose == 1 { "y" } else { "ies" }
    );
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

    // ── Search + item list ────────────────────────────────────────────────────
    let search_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(body[0]);

    let search_label = Paragraph::new("  Search:").style(Style::default().fg(Color::DarkGray));
    frame.render_widget(search_label, search_chunks[0]);

    let search_input = Paragraph::new(format!("  {}▌", app.builder.feat_picker_search))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(search_input, search_chunks[1]);

    let search = app.builder.feat_picker_search.to_lowercase();
    let weapons = app.filtered_mastery_weapons();
    let filtered: Vec<_> = weapons
        .iter()
        .filter(|w| search.is_empty() || w.name.to_lowercase().contains(&search))
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let chosen = app.builder.weapon_mastery_choices.contains(&w.name);
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
            ListItem::new(format!("{}{}", prefix, w.name)).style(style)
        })
        .collect();

    use ratatui::widgets::ListState;
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Weapons with Mastery ({}/{} chosen) ",
            app.builder.weapon_mastery_choices.len(),
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

    // ── Detail panel ─────────────────────────────────────────────────────────
    let detail_lines = if let Some(weapon) = filtered.get(app.builder.feat_picker_index) {
        let mastery_name = weapon
            .mastery
            .as_ref()
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| "—".to_string());

        let description = match mastery_name.to_lowercase().as_str() {
            "cleave" => "Allows you to make an additional attack against a second target adjacent to your first.",
            "graze" => "Deals damage (equal to ability modifier) even when you miss your attack.",
            "nick" => "Allows an additional attack as part of the Attack action (not a Bonus Action).",
            "push" => "Pushes your target 10 feet away after a successful hit.",
            "sap" => "Gives the target disadvantage on their next attack roll.",
            "slow" => "Reduces the target's speed by 10 feet until the start of your next turn.",
            "topple" => "Forces the target to make a CON save or fall prone.",
            "vex" => "Gives you advantage on your next attack roll against the same target.",
            _ => "No description available.",
        };

        vec![
            Line::from(Span::styled(
                weapon.name.clone(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Mastery: ", Style::default().fg(Color::DarkGray)),
                Span::styled(mastery_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled(description, Style::default().fg(Color::White))),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No weapons found",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let detail = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Weapon Details "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, body[1]);

    let chosen_count = app.builder.weapon_mastery_choices.len() as u8;
    let help = if chosen_count >= choose {
        format!(
            "Enter/Space confirm  Esc clear selection  ({}/{} chosen)",
            chosen_count, choose
        )
    } else {
        format!(
            "Space toggle  Enter confirm when done  ↑↓ navigate  ({}/{} chosen)",
            chosen_count, choose
        )
    };
    let help_w = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_w, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let choose = choose_count(app) as usize;

    let search = app.builder.feat_picker_search.to_lowercase();
    let weapons = app.filtered_mastery_weapons();
    let filtered_count = weapons
        .iter()
        .filter(|w| search.is_empty() || w.name.to_lowercase().contains(&search))
        .count();

    match key.code {
        KeyCode::Esc => {
            // Clear chosen and go back to whatever called us (Summary or Class)
            app.builder.weapon_mastery_choices.clear();
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
            if app.builder.feat_picker_index + 1 < filtered_count {
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
        // Space or Enter: toggle selection
        KeyCode::Char(' ') | KeyCode::Enter => {
            let search = app.builder.feat_picker_search.to_lowercase();
            let weapons = app.filtered_mastery_weapons();
            let filtered: Vec<_> = weapons
                .iter()
                .filter(|w| search.is_empty() || w.name.to_lowercase().contains(&search))
                .collect();

            if let Some(weapon) = filtered.get(app.builder.feat_picker_index) {
                let name = weapon.name.clone();
                let already = app
                    .builder
                    .weapon_mastery_choices
                    .iter()
                    .position(|n| n == &name);
                if let Some(idx) = already {
                    // De-select
                    app.builder.weapon_mastery_choices.remove(idx);
                } else if app.builder.weapon_mastery_choices.len() < choose {
                    // Select
                    app.builder.weapon_mastery_choices.push(name.clone());
                    app.status_msg = format!("Weapon mastery: {} selected", name);
                }
            }

            // Auto-advance when quota met
            if app.builder.weapon_mastery_choices.len() >= choose {
                app.builder.builder_pending_feature = None;
                app.builder.feat_picker_search.clear();
                app.builder.feat_picker_index = 0;
                
                if app.builder.skip_subclass {
                    app.builder.step = CharacterCreationStep::Abilities;
                } else {
                    app.builder.step = CharacterCreationStep::Subclass;
                }

                app.status_msg = format!(
                    "Weapon masteries confirmed: {}",
                    app.builder.weapon_mastery_choices.join(", ")
                );
            }
        }
        _ => {}
    }
}
