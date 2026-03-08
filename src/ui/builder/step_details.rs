use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

// Field indices
const F_NAME: usize = 0;
const F_AGE: usize = 1;
const F_HEIGHT: usize = 2;
const F_WEIGHT: usize = 3;
const F_APPEARANCE: usize = 4;
const F_ALIGNMENT: usize = 5;
const F_TRAIT: usize = 6;
const F_IDEAL: usize = 7;
const F_BOND: usize = 8;
const F_FLAW: usize = 9;
const FIELD_COUNT: usize = 10;

const ALIGNMENTS: [(&str, &str); 9] = [
    (
        "Lawful Good",
        "LG — Principled. Follows rules, helps others.",
    ),
    (
        "Neutral Good",
        "NG — Altruistic. Does what's best, no agenda.",
    ),
    (
        "Chaotic Good",
        "CG — Free spirit. Does right, ignores rules.",
    ),
    (
        "Lawful Neutral",
        "LN — Bound by code or tradition above all.",
    ),
    (
        "True Neutral",
        "TN — Balanced. Avoids extremes of any axis.",
    ),
    (
        "Chaotic Neutral",
        "CN — Does whatever, freedom above all else.",
    ),
    ("Lawful Evil", "LE — Methodical. Uses rules to gain power."),
    (
        "Neutral Evil",
        "NE — Self-serving. No loyalty, pure self-interest.",
    ),
    (
        "Chaotic Evil",
        "CE — Destructive. Cruelty and chaos for its own sake.",
    ),
];

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Step 7 — Character Details ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // Body: left (basic fields) | middle (alignment picker) | right (personality fields)
    let body = Layout::horizontal([
        Constraint::Percentage(28),
        Constraint::Percentage(32),
        Constraint::Percentage(40),
    ])
    .split(outer[1]);

    render_basic_fields(app, frame, body[0]);
    render_alignment_picker(app, frame, body[1]);
    render_personality_fields(app, frame, body[2]);

    // Footer
    let help = if app.builder.focus_index == F_ALIGNMENT {
        "↑↓ choose alignment  Tab next field  Enter next/submit  Esc back"
    } else {
        "Tab/↑↓ navigate fields  Type to enter  Enter next/submit  Esc back"
    };
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, outer[2]);
}

fn render_basic_fields(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let rows = Layout::vertical([
        Constraint::Length(3), // Name
        Constraint::Length(3), // Age
        Constraint::Length(3), // Height
        Constraint::Length(3), // Weight
        Constraint::Length(4), // Appearance
        Constraint::Min(0),
    ])
    .margin(1)
    .split(area);

    let fields = [
        ("Character Name *", &app.builder.name, F_NAME),
        ("Age", &app.builder.age, F_AGE),
        ("Height", &app.builder.height, F_HEIGHT),
        ("Weight", &app.builder.weight, F_WEIGHT),
        ("Eyes / Hair / Skin", &app.builder.appearance, F_APPEARANCE),
    ];

    for (i, (label, val, idx)) in fields.iter().enumerate() {
        let focused = app.builder.focus_index == *idx;
        let border_style = if focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let display = if focused {
            format!("{}█", val)
        } else if val.is_empty() {
            if *idx == F_NAME {
                "⚠ Required".to_string()
            } else {
                "—".to_string()
            }
        } else {
            (*val).clone()
        };
        let p = Paragraph::new(display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(*label)
                    .border_style(border_style),
            )
            .style(if focused {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            });
        if i < rows.len() - 1 {
            frame.render_widget(p, rows[i]);
        }
    }
}

fn render_alignment_picker(app: &mut App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let focused = app.builder.focus_index == F_ALIGNMENT;
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    // Parse current alignment to find selected index
    let selected_idx = ALIGNMENTS
        .iter()
        .position(|(name, _)| *name == app.builder.alignment.as_str());

    // Keep list_state for alignment in sync when focused
    let align_selected = selected_idx.unwrap_or(0);
    if focused {
        app.builder
            .alignment_list_state
            .select(Some(align_selected));
    }

    let items: Vec<ListItem> = ALIGNMENTS
        .iter()
        .map(|(name, desc)| {
            ListItem::new(vec![
                Line::from(Span::styled(*name, Style::default().fg(Color::White))),
                Line::from(Span::styled(*desc, Style::default().fg(Color::DarkGray))),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Alignment ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(if focused { "> " } else { "  " });

    frame.render_stateful_widget(list, area, &mut app.builder.alignment_list_state);
}

fn render_personality_fields(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    // Get suggestions from background entries
    let suggestions = get_bg_suggestions(app);

    let rows = Layout::vertical([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .margin(1)
    .split(area);

    let fields = [
        (
            "Personality Trait",
            &app.builder.trait_text,
            F_TRAIT,
            suggestions.0.as_str(),
        ),
        ("Ideal", &app.builder.ideal, F_IDEAL, suggestions.1.as_str()),
        ("Bond", &app.builder.bond, F_BOND, suggestions.2.as_str()),
        ("Flaw", &app.builder.flaw, F_FLAW, suggestions.3.as_str()),
    ];

    for (i, (label, val, idx, hint)) in fields.iter().enumerate() {
        let focused = app.builder.focus_index == *idx;
        let border_style = if focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display = if focused {
            format!("{}█", val)
        } else if val.is_empty() {
            format!("(e.g. {})", hint)
        } else {
            (*val).clone()
        };

        let text_style = if val.is_empty() && !focused {
            Style::default().fg(Color::DarkGray)
        } else if focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let p = Paragraph::new(display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(*label)
                    .border_style(border_style),
            )
            .style(text_style)
            .wrap(Wrap { trim: true });
        frame.render_widget(p, rows[i]);
    }
}

/// Returns (trait_hint, ideal_hint, bond_hint, flaw_hint) from the selected background
fn get_bg_suggestions(app: &App) -> (String, String, String, String) {
    // Static suggestions per background name — fallback when entries aren't parseable
    let bg_name = app
        .builder
        .bg_id
        .and_then(|id| app.backgrounds.iter().find(|b| b.id == id))
        .map(|b| b.name.as_str())
        .unwrap_or("");

    match bg_name {
        "Soldier" | "soldier" => (
            "I'm always polite and respectful.".into(),
            "Greater Good — peace is worth fighting for.".into(),
            "I fight for those who can't fight themselves.".into(),
            "I made a terrible mistake in battle I must atone for.".into(),
        ),
        "Criminal" | "criminal" | "Spy" => (
            "I always have a plan for when things go wrong.".into(),
            "Freedom — chains are meant to be broken.".into(),
            "I'm trying to pay off an old debt.".into(),
            "I turn tail and run when things look bad.".into(),
        ),
        "Sage" | "sage" => (
            "I use polysyllabic words that convey intelligence.".into(),
            "Knowledge — the path to power and self-improvement.".into(),
            "I have an ancient text that holds terrible secrets.".into(),
            "I speak without thinking, often insulting others.".into(),
        ),
        "Acolyte" | "acolyte" => (
            "I idolize a hero of my religion.".into(),
            "Faith — trust in my deity guides me.".into(),
            "I seek to prove my faith through action.".into(),
            "I am inflexible in my thinking.".into(),
        ),
        "Folk Hero" | "folk hero" => (
            "I judge people by their actions, not words.".into(),
            "People deserve to live free from tyranny.".into(),
            "I protect those who cannot protect themselves.".into(),
            "The tyrant who threatened my people still lives.".into(),
        ),
        "Noble" | "noble" => (
            "My favor, once lost, is lost forever.".into(),
            "Nobility obligates me to act with honor.".into(),
            "I will face any challenge to win my family's approval.".into(),
            "I secretly believe everyone is beneath me.".into(),
        ),
        _ => (
            "I have a strong sense of fair play.".into(),
            "Aspiration — I seek to better myself.".into(),
            "I have a goal that drives everything I do.".into(),
            "I have trouble trusting people.".into(),
        ),
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let focused = app.builder.focus_index;

    // Alignment field uses Up/Down to scroll the picker list
    if focused == F_ALIGNMENT {
        match key.code {
            KeyCode::Esc => {
                app.builder.step = CharacterCreationStep::Spells;
                app.status_msg.clear();
                return;
            }
            KeyCode::Up => {
                let cur = app.builder.alignment_list_state.selected().unwrap_or(0);
                let next = if cur > 0 {
                    cur - 1
                } else {
                    ALIGNMENTS.len() - 1
                };
                app.builder.alignment_list_state.select(Some(next));
                app.builder.alignment = ALIGNMENTS[next].0.to_string();
                return;
            }
            KeyCode::Down => {
                let cur = app.builder.alignment_list_state.selected().unwrap_or(0);
                let next = if cur + 1 < ALIGNMENTS.len() {
                    cur + 1
                } else {
                    0
                };
                app.builder.alignment_list_state.select(Some(next));
                app.builder.alignment = ALIGNMENTS[next].0.to_string();
                return;
            }
            KeyCode::Tab => {
                app.builder.focus_index = F_TRAIT;
                return;
            }
            KeyCode::BackTab => {
                app.builder.focus_index = F_APPEARANCE;
                return;
            }
            KeyCode::Enter => {
                // Move to next field
                app.builder.focus_index = F_TRAIT;
                return;
            }
            _ => return,
        }
    }

    match key.code {
        KeyCode::Esc => {
            if app.builder.spellcasting_type == "none" {
                app.builder.step = CharacterCreationStep::Equipment;
            } else {
                app.builder.step = CharacterCreationStep::Spells;
            }
            app.status_msg.clear();
        }
        KeyCode::Up | KeyCode::BackTab => {
            if app.builder.focus_index > 0 {
                app.builder.focus_index -= 1;
            } else {
                app.builder.focus_index = FIELD_COUNT - 1;
            }
        }
        KeyCode::Down | KeyCode::Tab => {
            if app.builder.focus_index < FIELD_COUNT - 1 {
                app.builder.focus_index += 1;
            } else {
                app.builder.focus_index = 0;
            }
        }
        KeyCode::Backspace => {
            get_focused_field(app).pop();
        }
        KeyCode::Char(c) => {
            get_focused_field(app).push(c);
        }
        KeyCode::Enter => {
            if app.builder.focus_index < FIELD_COUNT - 1 {
                app.builder.focus_index += 1;
            } else {
                // Last field: try to advance to summary
                advance_to_summary(app);
            }
        }
        _ => {}
    }
}

fn advance_to_summary(app: &mut App) {
    if app.builder.name.trim().is_empty() {
        app.status_msg = "Character Name is required!".to_string();
        app.builder.focus_index = F_NAME;
        return;
    }
    app.builder.step = CharacterCreationStep::Summary;
    app.builder.focus_index = 0;
    app.status_msg.clear();
}

fn get_focused_field(app: &mut App) -> &mut String {
    match app.builder.focus_index {
        F_NAME => &mut app.builder.name,
        F_AGE => &mut app.builder.age,
        F_HEIGHT => &mut app.builder.height,
        F_WEIGHT => &mut app.builder.weight,
        F_APPEARANCE => &mut app.builder.appearance,
        F_ALIGNMENT => &mut app.builder.alignment,
        F_TRAIT => &mut app.builder.trait_text,
        F_IDEAL => &mut app.builder.ideal,
        F_BOND => &mut app.builder.bond,
        F_FLAW => &mut app.builder.flaw,
        _ => &mut app.builder.name,
    }
}
