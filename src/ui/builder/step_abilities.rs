use crate::app::App;
use crate::models::app_state::{CharacterCreationStep, AbilityMethod};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

const ABILITIES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
const STANDARD_ARRAY: [i32; 6] = [15, 14, 13, 12, 10, 8];
const POINT_BUY_BUDGET: i32 = 27;

fn points_spent(scores: &[i32; 6]) -> i32 {
    scores.iter().map(|&s| point_cost(s)).sum()
}

fn point_cost(score: i32) -> i32 {
    match score {
        8 => 0, 9 => 1, 10 => 2, 11 => 3, 12 => 4,
        13 => 5, 14 => 7, 15 => 9,
        _ => 0,
    }
}

fn modifier(score: i32) -> i32 { (score - 10) / 2 }

fn mod_str(score: i32) -> String {
    let m = modifier(score);
    if m >= 0 { format!("+{}", m) } else { format!("{}", m) }
}

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let method = app.builder.ability_method;
    let body = Layout::vertical([
        Constraint::Length(3), // method selector
        Constraint::Min(0),
    ]).split(area);

    // Method toggle bar
    let methods = [
        ("Standard Array", AbilityMethod::StandardArray),
        ("Point Buy", AbilityMethod::PointBuy),
        ("Manual Entry", AbilityMethod::Manual),
    ];
    let mut method_spans: Vec<Span> = Vec::new();
    for (i, (label, m)) in methods.iter().enumerate() {
        if i > 0 { method_spans.push(Span::raw("  |  ")); }
        let is_active = *m == method;
        method_spans.push(Span::styled(
            if is_active { format!("[{}]", label) } else { label.to_string() },
            if is_active {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ));
    }
    method_spans.push(Span::styled("  (Tab to switch)", Style::default().fg(Color::DarkGray)));
    let method_p = Paragraph::new(Line::from(method_spans))
        .block(Block::default().borders(Borders::ALL).title(" Ability Score Method ").border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(method_p, body[0]);

    // Abilities table
    let content = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).split(body[1]);
    render_ability_table(app, frame, content[0]);
    render_summary(app, frame, content[1]);
}

fn render_ability_table(app: &mut App, frame: &mut Frame, area: Rect) {
    let method = app.builder.ability_method;
    let spent = if method == AbilityMethod::PointBuy { points_spent(&app.builder.ability_scores) } else { 0 };
    let budget_left = POINT_BUY_BUDGET - spent;

    let method_hint = match method {
        AbilityMethod::StandardArray => "Using standard array: 15,14,13,12,10,8 (assigned top→down)",
        AbilityMethod::PointBuy => &format!("Point Buy — Budget: {} remaining / {}", budget_left, POINT_BUY_BUDGET)[..],
        AbilityMethod::Manual => "Manual: +/- to adjust selected score",
    };

    let header = Row::new(vec![
        Cell::from("Ability").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Score").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Mod").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("Cost").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = ABILITIES.iter().enumerate().map(|(i, &ab)| {
        let is_selected = app.builder.ability_cursor == i;
        let score = if method == AbilityMethod::StandardArray {
            STANDARD_ARRAY[i]
        } else {
            app.builder.ability_scores[i]
        };

        let cost_str = if method == AbilityMethod::PointBuy {
            format!("{}", point_cost(score))
        } else {
            "—".to_string()
        };

        let style = if is_selected {
            Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        Row::new(vec![
            Cell::from(ab),
            Cell::from(format!("{:>2}", score)),
            Cell::from(mod_str(score)),
            Cell::from(cost_str),
        ]).style(style)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(8), Constraint::Length(6), Constraint::Length(5), Constraint::Length(5)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(" Ability Scores — {} ", method_hint)).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(table, area);
}

fn render_summary(app: &App, frame: &mut Frame, area: Rect) {
    let method = app.builder.ability_method;
    let mut lines = vec![
        Line::from(Span::styled("Score Summary", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    for (i, &ab) in ABILITIES.iter().enumerate() {
        let score = if method == AbilityMethod::StandardArray { STANDARD_ARRAY[i] } else { app.builder.ability_scores[i] };
        lines.push(Line::from(vec![
            Span::styled(format!("{:>3}: ", ab), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:>2} ", score), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("({})", mod_str(score)), Style::default().fg(Color::Cyan)),
        ]));
    }

    // Hint about next step
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("Press Enter to continue", Style::default().fg(Color::DarkGray))));

    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Preview ").border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(p, area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let method = app.builder.ability_method;
    match key.code {
        KeyCode::Esc => {
            app.builder.step = CharacterCreationStep::Species;
            app.builder.list_state.select(Some(0));
            app.status_msg.clear();
        }
        KeyCode::Tab => {
            app.builder.ability_method = match method {
                AbilityMethod::StandardArray => AbilityMethod::PointBuy,
                AbilityMethod::PointBuy => AbilityMethod::Manual,
                AbilityMethod::Manual => AbilityMethod::StandardArray,
            };
        }
        KeyCode::Up => {
            if app.builder.ability_cursor > 0 {
                app.builder.ability_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.builder.ability_cursor < 5 {
                app.builder.ability_cursor += 1;
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if method == AbilityMethod::PointBuy || method == AbilityMethod::Manual {
                let i = app.builder.ability_cursor;
                let cur = app.builder.ability_scores[i];
                let budget_left = POINT_BUY_BUDGET - points_spent(&app.builder.ability_scores);
                let new_score = cur + 1;
                let delta_cost = point_cost(new_score) - point_cost(cur);
                if new_score <= 15 {
                    if method == AbilityMethod::Manual || delta_cost <= budget_left {
                        app.builder.ability_scores[i] = new_score;
                    } else {
                        app.status_msg = format!("Not enough points (need {}, have {}).", delta_cost, budget_left);
                    }
                }
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if method == AbilityMethod::PointBuy || method == AbilityMethod::Manual {
                let i = app.builder.ability_cursor;
                let cur = app.builder.ability_scores[i];
                if cur > 8 {
                    app.builder.ability_scores[i] = cur - 1;
                }
            }
        }
        KeyCode::Enter => {
            // Apply standard array if selected
            if method == AbilityMethod::StandardArray {
                app.builder.ability_scores = STANDARD_ARRAY;
            }

            if !app.save_draft() { return; }
            app.builder.step = CharacterCreationStep::Equipment;
            app.builder.list_state.select(Some(0));
            app.status_msg.clear();
        }
        _ => {}
    }
}
