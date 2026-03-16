use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::app::App;
use crate::models::actions::ActionEntry;
use crate::models::app_state::ActionsSubTab;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2), // Nav bar
        Constraint::Min(0),    // Content
    ])
    .split(area);

    render_nav_bar(app, frame, chunks[0]);

    if let Some(ref actions) = app.char_actions {
        if app.actions_sub_tab == ActionsSubTab::LimitedUse {
            let limited = actions.limited_use.clone();
            render_limited_use_list(app, &limited, frame, chunks[1]);
        } else {
            let list = match app.actions_sub_tab {
                ActionsSubTab::All => &actions.all,
                ActionsSubTab::Attack => &actions.attack,
                ActionsSubTab::Action => &actions.action,
                ActionsSubTab::BonusAction => &actions.bonus_action,
                ActionsSubTab::Reaction => &actions.reaction,
                ActionsSubTab::Other => &actions.other,
                ActionsSubTab::LimitedUse => unreachable!(),
            };
            render_action_list(app, list, frame, chunks[1]);
        }
    } else {
        let msg = "  Loading actions...";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    // --- Render Modal Overlay if open ---
    if let Some((modal_name, modal_desc)) = &app.actions_detail_modal {
        let overlay_area = Rect::new(
            chunks[1].x + 2,
            chunks[1].y + 2,
            chunks[1].width.saturating_sub(4),
            chunks[1].height.saturating_sub(4),
        );

        frame.render_widget(Clear, overlay_area);

        let block = Block::default()
            .title(format!(" {} (Press any key to close) ", modal_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let p = Paragraph::new(modal_desc.as_str())
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(p, overlay_area);
    }
}

fn render_nav_bar(app: &App, frame: &mut Frame, area: Rect) {
    let tabs = vec![
        (ActionsSubTab::All, " All "),
        (ActionsSubTab::Attack, " Attack "),
        (ActionsSubTab::Action, " Action "),
        (ActionsSubTab::BonusAction, " Bonus Action "),
        (ActionsSubTab::Reaction, " Reaction "),
        (ActionsSubTab::Other, " Other "),
        (ActionsSubTab::LimitedUse, " Limited Use "),
    ];

    let mut line = Vec::new();
    line.push(Span::raw("   ")); // padding

    for (idx, (tab, label)) in tabs.iter().enumerate() {
        let style = if app.actions_sub_tab == *tab {
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        line.push(Span::styled(*label, style));
        if idx < tabs.len() - 1 {
            line.push(Span::raw(" | "));
        }
    }

    let p = Paragraph::new(Line::from(line));
    frame.render_widget(p, area);
}

fn render_action_list(app: &App, actions: &[ActionEntry], frame: &mut Frame, area: Rect) {
    if actions.is_empty() {
        let msg = "  No actions in this category.";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let mut total_lines: Vec<Line> = Vec::new();
    total_lines.push(Line::from(""));

    for action in actions {
        let mut header_spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("* {}", action.name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some(source) = &action.source {
            header_spans.push(Span::raw(" "));
            header_spans.push(Span::styled(
                format!("({})", source),
                Style::default().fg(Color::DarkGray),
            ));
        }

        total_lines.push(Line::from(header_spans));

        let mut props = Vec::new();
        if let Some(r) = &action.range {
            props.push(format!("Range: {}", r));
        }
        if let Some(h) = &action.hit_bonus {
            props.push(format!("Hit/DC: {}", h));
        }
        if let Some(d) = &action.damage {
            props.push(format!("Damage: {}", d));
        }

        let mut use_str;
        if let (Some(cu), Some(mu)) = (action.current_uses, action.max_uses) {
            use_str = format!("Uses: {}/{}", cu, mu);
            if let Some(rt) = &action.reset_type {
                use_str.push_str(&format!(" (per {})", rt));
            }
            props.push(use_str);
        }

        if !props.is_empty() {
            total_lines.push(Line::from(Span::styled(
                format!("    {}", props.join(" | ")),
                Style::default().fg(Color::Yellow),
            )));
        }

        if let Some(desc) = &action.description {
            if !desc.is_empty() && desc != "null" {
                let mut desc_clean = desc.replace("\\n", "\n").replace("\"", "");
                if desc_clean.starts_with('[') && desc_clean.ends_with(']') {
                    desc_clean = desc_clean[1..desc_clean.len() - 1].to_string();
                }
                for line in desc_clean.lines() {
                    let cleaned = crate::ui::sheet::features::strip_tags(line);
                    if !cleaned.trim().is_empty() {
                        total_lines.push(Line::from(Span::raw(format!("    {}", cleaned))));
                    }
                }
            }
        }
        total_lines.push(Line::from(""));
        total_lines.push(Line::from(""));
    }

    let p = Paragraph::new(total_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.content_scroll as u16, 0));

    frame.render_widget(p, area);
}

fn render_limited_use_list(app: &mut App, actions: &[ActionEntry], frame: &mut Frame, area: Rect) {
    if actions.is_empty() {
        let msg = "  No limited-use actions.";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let selected = app.actions_list_state.selected().unwrap_or(0);

    let items: Vec<ListItem> = actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let is_selected = i == selected;
            let current = action.current_uses.unwrap_or(0);
            let max = action.max_uses.unwrap_or(0);
            let reset = action.reset_type.as_deref().unwrap_or("Long Rest");

            // Header line
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };
            let name_line = Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("★ {}", action.name), name_style),
            ]);

            // Description snippet
            let desc_line = if let Some(desc) = &action.description {
                let mut d = desc.replace("\\n", " ").replace("\"", "");
                if d.starts_with('[') && d.ends_with(']') {
                    d = d[1..d.len() - 1].to_string();
                }
                let cleaned = crate::ui::sheet::features::strip_tags(&d);
                let snippet: String = cleaned.chars().take(90).collect();
                let snippet = if cleaned.len() > 90 {
                    format!("{}…", snippet)
                } else {
                    snippet
                };
                Line::from(Span::styled(
                    format!("    {}", snippet),
                    Style::default().fg(Color::Gray),
                ))
            } else {
                Line::from("")
            };

            // Counter line  ← /  ←  current / max  →
            let (minus_style, plus_style) = if is_selected {
                (
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::DarkGray),
                )
            };

            let counter_line = Line::from(vec![
                Span::raw("    "),
                Span::styled("[-]", minus_style),
                Span::raw("  "),
                Span::styled(
                    format!("{} / {}", current, max),
                    Style::default()
                        .fg(if is_selected {
                            Color::White
                        } else {
                            Color::Gray
                        })
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled("[+]", plus_style),
                Span::raw("   "),
                Span::styled(format!("⟳ {}", reset), Style::default().fg(Color::DarkGray)),
            ]);

            let hint_line = if is_selected {
                Line::from(Span::styled(
                    "    Press  -  to spend a use  /  +  to recover",
                    Style::default().fg(Color::DarkGray),
                ))
            } else {
                Line::from("")
            };

            ListItem::new(vec![
                name_line,
                desc_line,
                counter_line,
                hint_line,
                Line::from(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 30, 50))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    frame.render_stateful_widget(list, area, &mut app.actions_list_state);
}
