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
    ])
    .split(area);

    let top =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(body[0]);

    let option_a_selected = app.builder.equipment_option == Some(0);
    let option_b_selected = app.builder.equipment_option == Some(1);
    let focus_a = app.builder.focus_index == 0;
    let focus_b = app.builder.focus_index == 1;

    // ── Left panel: Option A — Starting Equipment ────────────────────────────
    let left_border = if option_a_selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else if focus_a {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_a = if option_a_selected {
        " ✓ Option A: Starting Equipment (Selected) "
    } else {
        " Option A: Starting Equipment "
    };

    let items: Vec<ListItem> = app
        .builder
        .equipment_options
        .iter()
        .map(|line| {
            // Section headers (── Class equipment ──) get a distinct style
            if line.starts_with("──") {
                ListItem::new(Line::from(Span::styled(
                    line.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )))
            } else if line.is_empty() {
                ListItem::new(Line::from(""))
            } else {
                ListItem::new(Line::from(Span::styled(
                    line.clone(),
                    if option_a_selected {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    },
                )))
            }
        })
        .collect();

    let items_or_empty: Vec<ListItem> = if items.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  (no equipment data — check class selection)",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        items
    };

    let eq_list = List::new(items_or_empty)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_a)
                .border_style(left_border),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    if focus_a {
        frame.render_stateful_widget(eq_list, top[0], &mut app.builder.list_state);
    } else {
        frame.render_widget(eq_list, top[0]);
    }

    // ── Right panel: Option B — Starting Gold ────────────────────────────────
    let right_border = if option_b_selected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else if focus_b {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_b = if option_b_selected {
        " ✓ Option B: Starting Gold (Selected) "
    } else {
        " Option B: Starting Gold "
    };

    // Build gold breakdown text
    let class_name = app
        .builder
        .class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Class".to_string());
    let bg_name = app
        .builder
        .bg_id
        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "Background".to_string());

    let additional_from_bg = app
        .builder
        .class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .and_then(|c| {
            c.starting_equipment
                .get("additionalFromBackground")
                .and_then(|v| v.as_bool())
        })
        .unwrap_or(false);

    let class_gold = app
        .builder
        .class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .map(|c| App::extract_option_b_gold_pub(&c.starting_equipment))
        .unwrap_or(0);

    let bg_gold: i32 = if additional_from_bg {
        app.builder
            .bg_id
            .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
            .and_then(|b| b.starting_equipment.as_ref())
            .map(|bg_eq| {
                let wrapped = serde_json::json!({ "defaultData": bg_eq });
                App::extract_option_b_gold_pub(&wrapped)
            })
            .unwrap_or(0)
    } else {
        0
    };

    let total_gold = class_gold + bg_gold;

    let mut gold_lines: Vec<Line> = Vec::new();
    if class_gold > 0 {
        gold_lines.push(Line::from(vec![
            Span::styled(
                format!("  {} gold:  ", class_name),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} GP", class_gold),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if bg_gold > 0 {
        gold_lines.push(Line::from(vec![
            Span::styled(
                format!("  {} gold:  ", bg_name),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} GP", bg_gold),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    if total_gold > 0 && (class_gold > 0 || bg_gold > 0) {
        gold_lines.push(Line::from(""));
        gold_lines.push(Line::from(vec![
            Span::styled("  Total:          ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} GP", total_gold),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    if gold_lines.is_empty() {
        gold_lines.push(Line::from(Span::styled(
            "  (no gold data available)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    gold_lines.push(Line::from(""));
    gold_lines.push(Line::from(Span::styled(
        "  Press → or Tab to select this option.",
        Style::default().fg(Color::DarkGray),
    )));

    let gold_p = Paragraph::new(gold_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title_b)
                .border_style(right_border),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(gold_p, top[1]);

    // ── Bottom: Character summary ────────────────────────────────────────────
    render_character_summary(app, frame, body[1]);
}

fn render_character_summary(app: &App, frame: &mut Frame, area: Rect) {
    let class_name = app
        .builder
        .class_id
        .and_then(|id| app.classes.iter().find(|c| c.id == id))
        .map(|c| c.name.as_str())
        .unwrap_or("—");
    let race_name = app
        .builder
        .race_id
        .and_then(|id| app.races.iter().find(|r| r.id == id))
        .map(|r| r.name.as_str())
        .unwrap_or("—");
    let bg_name = app
        .builder
        .bg_id
        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
        .map(|b| b.name.as_str())
        .unwrap_or("—");

    let equip_status = match app.builder.equipment_option {
        Some(0) => Span::styled(
            "Option A selected (Starting Equipment)",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Some(1) => Span::styled(
            "Option B selected (Starting Gold)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        _ => Span::styled(
            "← → to choose equipment option, then Enter",
            Style::default().fg(Color::DarkGray),
        ),
    };

    let lines = vec![
        Line::from(Span::styled(
            "── Character Summary ──",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name:        ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.builder.name.clone(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Class:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} (Level {})", class_name, app.builder.level),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Species:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(race_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Background:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(bg_name, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Equipment:   ", Style::default().fg(Color::DarkGray)),
            equip_status,
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to Create Character!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let summary = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Final Review ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(summary, area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Abilities;
            app.builder.list_state.select(Some(0));
            app.builder.focus_index = 0;
            app.status_msg.clear();
        }
        // Tab / Right — move focus to Option B (gold)
        KeyCode::Tab | KeyCode::Right => {
            app.builder.focus_index = 1;
            app.builder.equipment_option = Some(1);
        }
        // Left — move focus to Option A (equipment bundle)
        KeyCode::Left | KeyCode::BackTab => {
            app.builder.focus_index = 0;
            app.builder.equipment_option = Some(0);
        }
        // Scroll Option A list
        KeyCode::Up => {
            if app.builder.focus_index == 0 {
                let n = app.builder.equipment_options.len();
                let i = app
                    .builder
                    .list_state
                    .selected()
                    .map(|i| if i > 0 { i - 1 } else { n.saturating_sub(1) })
                    .unwrap_or(0);
                app.builder.list_state.select(Some(i));
            }
        }
        KeyCode::Down => {
            if app.builder.focus_index == 0 {
                let n = app.builder.equipment_options.len();
                let i = app
                    .builder
                    .list_state
                    .selected()
                    .map(|i| if i + 1 < n { i + 1 } else { 0 })
                    .unwrap_or(0);
                app.builder.list_state.select(Some(i));
            }
        }
        // Enter on Option A pane selects Option A; Enter on B pane was already handled above
        KeyCode::Char(' ') | KeyCode::Enter if app.builder.focus_index == 0 && key.code == KeyCode::Char(' ') => {
            app.builder.equipment_option = Some(0);
        }
        KeyCode::Enter => {
            // Validate
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
            // Default to Option A if user never explicitly chose
            if app.builder.equipment_option.is_none() {
                app.builder.equipment_option = Some(0);
            }
            crate::app::events::builder::submit_character_from_builder(app);
        }
        _ => {}
    }
}
