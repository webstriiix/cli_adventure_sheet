use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let body = Layout::vertical([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]).split(area);

    let top = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(body[0]);

    // ── Left: Starting equipment choices ──
    let items: Vec<ListItem> = app.builder.equipment_options.iter().enumerate().map(|(i, opt)| {
        let is_selected = app.builder.equipment_choices.contains(&i);
        let style = if is_selected {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let check = if is_selected { "✓ " } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(check, Style::default().fg(Color::Green)),
            Span::styled(opt.clone(), style),
        ]))
    }).collect();

    let eq_border_focus = if app.builder.focus_index == 0 {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let eq_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Starting Equipment ").border_style(eq_border_focus))
        .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    frame.render_stateful_widget(eq_list, top[0], &mut app.builder.list_state);

    // ── Right: Gold alternative ──
    let gold_selected = app.builder.focus_index == 1;
    let gold_style = if gold_selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let gold_text = if let Some(gp) = app.builder.starting_gold {
        format!("Starting Gold: {}gp\n\nPress Tab to toggle between\nEquipment Packs and Gold.", gp)
    } else {
        "Starting Gold Option:\n\nPress → / Tab to use gold\ninstead of an equipment pack.".to_string()
    };
    let gold_p = Paragraph::new(gold_text)
        .block(Block::default().borders(Borders::ALL).title(" Starting Gold (Alternative) ").border_style(gold_style))
        .style(gold_style)
        .wrap(Wrap { trim: true });
    frame.render_widget(gold_p, top[1]);

    // ── Bottom: Character summary ──
    render_character_summary(app, frame, body[1]);
}

fn render_character_summary(app: &App, frame: &mut Frame, area: Rect) {
    let class_name = app.builder.class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .map(|c| c.name.as_str())
        .unwrap_or("—");
    let race_name = app.builder.race_id
        .and_then(|id| app.races.iter().find(|r| r.id == id))
        .map(|r| r.name.as_str())
        .unwrap_or("—");
    let bg_name = app.builder.bg_id
        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
        .map(|b| b.name.as_str())
        .unwrap_or("—");

    let lines = vec![
        Line::from(Span::styled("── Character Summary ──", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.builder.name.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Class:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} (Level {})", class_name, app.builder.level), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Species:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(race_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Background:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(bg_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press Enter to Create Character!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
    ];

    let summary = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Final Review ").border_style(Style::default().fg(Color::DarkGray)))
        .wrap(Wrap { trim: true });
    frame.render_widget(summary, area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Abilities;
            app.builder.list_state.select(Some(0));
            app.status_msg.clear();
        }
        KeyCode::Tab | KeyCode::Right => {
            app.builder.focus_index = if app.builder.focus_index == 0 { 1 } else { 0 };
        }
        KeyCode::Left => {
            app.builder.focus_index = 0;
        }
        KeyCode::Up => {
            if app.builder.focus_index == 0 {
                let n = app.builder.equipment_options.len();
                let i = app.builder.list_state.selected().map(|i| if i > 0 { i - 1 } else { n.saturating_sub(1) }).unwrap_or(0);
                app.builder.list_state.select(Some(i));
            }
        }
        KeyCode::Down => {
            if app.builder.focus_index == 0 {
                let n = app.builder.equipment_options.len();
                let i = app.builder.list_state.selected().map(|i| if i + 1 < n { i + 1 } else { 0 }).unwrap_or(0);
                app.builder.list_state.select(Some(i));
            }
        }
        KeyCode::Char(' ') => {
            // Toggle equipment selection
            if app.builder.focus_index == 0 {
                if let Some(i) = app.builder.list_state.selected() {
                    if app.builder.equipment_choices.contains(&i) {
                        app.builder.equipment_choices.retain(|&x| x != i);
                    } else {
                        app.builder.equipment_choices.push(i);
                    }
                }
            }
        }
        KeyCode::Enter => {
            // Validate we have a name and class
            if app.builder.name.trim().is_empty() {
                app.status_msg = "Character name is required!".to_string();
                app.builder.step = CharacterCreationStep::Background;
                return;
            }
            if app.builder.class_id.is_none() {
                app.status_msg = "Please select a class first.".to_string();
                app.builder.step = CharacterCreationStep::Class;
                return;
            }

            // Finalize and submit
            crate::app::events::builder::submit_character_from_builder(app);
        }
        _ => {}
    }
}
