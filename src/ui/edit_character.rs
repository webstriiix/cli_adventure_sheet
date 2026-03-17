use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, LevelUpPrompt};
use crate::models::app_state::{EditSection, MulticlassSection};
use crate::models::compendium::source_id_label;

// Field index constants
const F_NAME: usize = 0;
const F_XP: usize = 1;
const F_LEVEL: usize = 2;
const F_MAX_HP: usize = 3;
const F_CUR_HP: usize = 4;
const F_TEMP_HP: usize = 5;
const F_STR: usize = 6;
const F_DEX: usize = 7;
const F_CON: usize = 8;
const F_INT: usize = 9;
const F_WIS: usize = 10;
const F_CHA: usize = 11;
const F_INSPIRATION: usize = 12;
const F_RACE: usize = 13;
const F_CLASS: usize = 14;
const F_BG: usize = 15;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Overall layout: title + body + help
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);

    // Title
    let title = Paragraph::new(" Edit Character ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    // Body: left column (text fields) | right column (list pickers)
    let body = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(outer[1]);

    render_fields(app, frame, body[0]);
    render_pickers(app, frame, body[1]);

    // Help bar
    let section_hint = match app.edit_section {
        EditSection::Fields => "Tab/↑↓ move field  Enter next field  Ctrl+S or F2 save  Esc cancel",
        EditSection::Race => "↑↓ choose race  Tab next section  Esc back to fields",
        EditSection::Class => "↑↓ choose class  Tab next section  Esc back to fields",
        EditSection::Background => "↑↓ choose background  Tab next section  Esc back to fields",
        EditSection::Multiclass => "↑↓ select  a add class  l/+ level up  Tab next  Esc back",
        EditSection::LevelUpChoice => "↑↓ select  Enter confirm  Esc skip",
    };
    let help = Paragraph::new(section_hint).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, outer[2]);

    // Level-up overlay (rendered on top of everything else)
    if app.edit_section == EditSection::LevelUpChoice {
        render_level_up_overlay(app, frame, area);
    }
}

fn render_fields(app: &App, frame: &mut Frame, area: Rect) {
    let in_fields = app.edit_section == EditSection::Fields;
    let border_color = if in_fields { Color::Cyan } else { Color::Gray };

    let block = Block::default()
        .title(" Character Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Compute level from XP buffer
    let xp: i32 = app.edit_buffers[F_XP].parse().unwrap_or(0);
    let level = crate::utils::level_from_xp(xp);

    let field_defs: &[(&str, usize)] = &[
        ("Name", F_NAME),
        ("XP", F_XP),
        ("Level", F_LEVEL),
        ("Max HP", F_MAX_HP),
        ("Current HP", F_CUR_HP),
        ("Temp HP", F_TEMP_HP),
        ("STR", F_STR),
        ("DEX", F_DEX),
        ("CON", F_CON),
        ("INT", F_INT),
        ("WIS", F_WIS),
        ("CHA", F_CHA),
        ("Inspiration", F_INSPIRATION),
    ];

    let lines: Vec<Line> = field_defs
        .iter()
        .map(|&(label, idx)| {
            let is_focused = in_fields && app.edit_field_index == idx;
            let label_style = if is_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let value_style = if is_focused {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_focused { "> " } else { "  " };

            let value_text = if idx == F_XP {
                format!("{}  (Level {} threshold)", app.edit_buffers[idx], level)
            } else if idx == F_STR
                || idx == F_DEX
                || idx == F_CON
                || idx == F_INT
                || idx == F_WIS
                || idx == F_CHA
            {
                let v: i32 = app.edit_buffers[idx].parse().unwrap_or(0);
                let modifier = (v - 10) / 2;
                let sign = if modifier >= 0 { "+" } else { "" };
                format!("{}  ({}{} mod)", app.edit_buffers[idx], sign, modifier)
            } else {
                app.edit_buffers[idx].clone()
            };

            let cursor = if is_focused { "_" } else { "" };

            Line::from(vec![
                Span::raw(prefix),
                Span::styled(format!("{label:<14}"), label_style),
                Span::styled(format!("{value_text}{cursor}"), value_style),
            ])
        })
        .collect();

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_pickers(app: &mut App, frame: &mut Frame, area: Rect) {
    let picker_chunks = Layout::vertical([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(area);

    render_picker_list(
        app,
        frame,
        picker_chunks[0],
        "Race",
        F_RACE,
        EditSection::Race,
        app.races
            .iter()
            .map(|r| format!("{} [{}]", r.name, source_id_label(r.source_id)))
            .collect(),
        app.edit_race_index,
    );

    render_picker_list(
        app,
        frame,
        picker_chunks[1],
        "Class",
        F_CLASS,
        EditSection::Class,
        app.classes
            .iter()
            .map(|c| format!("{} [{}] (d{})", c.name, c.source_slug, c.hit_die))
            .collect(),
        app.edit_class_index,
    );

    render_picker_list(
        app,
        frame,
        picker_chunks[2],
        "Background",
        F_BG,
        EditSection::Background,
        app.backgrounds
            .iter()
            .map(|b| format!("{} [{}]", b.name, source_id_label(b.source_id)))
            .collect(),
        app.edit_bg_index,
    );

    render_multiclass_panel(app, frame, picker_chunks[3]);
}

fn render_multiclass_panel(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_active = app.edit_section == EditSection::Multiclass;
    let border_color = if is_active {
        Color::Yellow
    } else {
        Color::Gray
    };

    match app.multiclass_section {
        MulticlassSection::Add if is_active => {
            // Show class picker for adding
            let items: Vec<ListItem> = app
                .classes
                .iter()
                .map(|c| {
                    ListItem::new(format!("{} (d{})", c.name, c.hit_die))
                        .style(Style::default().fg(Color::White))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" Add Multiclass — Enter to confirm  Esc cancel ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");
            frame.render_stateful_widget(list, area, &mut app.multiclass_add_state);
        }
        _ => {
            // Show current multiclass list
            let block = Block::default()
                .title(if is_active {
                    " Multiclass [a add] [l level up] "
                } else {
                    " Multiclass [Enter to manage] "
                })
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));

            if app.char_classes.is_empty() {
                let inner = block.inner(area);
                frame.render_widget(block, area);
                frame.render_widget(
                    Paragraph::new("  No multiclass entries.")
                        .style(Style::default().fg(Color::DarkGray)),
                    inner,
                );
            } else {
                let items: Vec<ListItem> = app
                    .char_classes
                    .iter()
                    .map(|cc| {
                        let name = app
                            .classes
                            .iter()
                            .find(|c| c.id == cc.class_id)
                            .map(|c| c.name.as_str())
                            .unwrap_or("?");
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("  {}", name), Style::default().fg(Color::White)),
                            Span::styled(
                                format!("  Lv {}", cc.level),
                                Style::default().fg(Color::Cyan),
                            ),
                        ]))
                    })
                    .collect();

                let list = List::new(items)
                    .block(block)
                    .highlight_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                    .highlight_symbol("> ");

                if is_active {
                    let mut state = ratatui::widgets::ListState::default()
                        .with_selected(Some(app.multiclass_selected));
                    frame.render_stateful_widget(list, area, &mut state);
                } else {
                    frame.render_widget(list, area);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_picker_list(
    app: &mut App,
    frame: &mut Frame,
    area: Rect,
    title: &str,
    field_idx: usize,
    section: EditSection,
    items_text: Vec<String>,
    selected_idx: usize,
) {
    let is_active = app.edit_section == section;
    let is_focused_field =
        app.edit_section == EditSection::Fields && app.edit_field_index == field_idx;

    let border_color = if is_active {
        Color::Yellow
    } else if is_focused_field {
        Color::Cyan
    } else {
        Color::Gray
    };

    let selected_name = items_text
        .get(selected_idx)
        .cloned()
        .unwrap_or_else(|| "—".to_string());

    let block_title = if is_active {
        format!(" {title}: {selected_name} ")
    } else {
        format!(" {title}: {selected_name} [Enter to change] ")
    };

    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if is_active {
        let list_items: Vec<ListItem> = items_text
            .iter()
            .map(|name| ListItem::new(name.clone()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(list_items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        let state = match section {
            EditSection::Race => &mut app.edit_race_state,
            EditSection::Class => &mut app.edit_class_state,
            EditSection::Background => &mut app.edit_bg_state,
            EditSection::Fields | EditSection::Multiclass | EditSection::LevelUpChoice => return,
        };
        frame.render_stateful_widget(list, area, state);
    } else {
        // Collapsed: just show the block with the selected name in title
        frame.render_widget(block, area);
    }
}

fn render_level_up_overlay(app: &mut App, frame: &mut Frame, area: Rect) {
    use ratatui::layout::Alignment;
    use ratatui::widgets::{Clear, List, ListItem, ListState};

    // Centered popup: 70% wide, 60% tall
    let popup_area = {
        let w = area.width * 70 / 100;
        let h = area.height * 60 / 100;
        let x = area.x + (area.width - w) / 2;
        let y = area.y + (area.height - h) / 2;
        Rect::new(x, y, w, h)
    };

    frame.render_widget(Clear, popup_area);

    let prompt: LevelUpPrompt = match &app.level_up_current {
        Some(p) => p.clone(),
        None => return,
    };

    match &prompt {
        LevelUpPrompt::SubclassChoice { class_name, .. } => {
            let title = format!(" Level Up: Choose Subclass for {} ", class_name);
            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(popup_area);
            frame.render_widget(block, popup_area);

            if let Some(detail) = &app.class_detail {
                let items: Vec<ListItem> = detail
                    .subclasses
                    .iter()
                    .map(|swf| {
                        let s = &swf.subclass;
                        let source = if s.source_slug.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", s.source_slug)
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("  {}", s.name),
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(source, Style::default().fg(Color::Cyan)),
                            Span::styled(
                                format!("  (unlocks lv {})", s.unlock_level),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]))
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(" > ");
                let mut state = ListState::default().with_selected(Some(app.picker_selected));
                frame.render_stateful_widget(list, inner, &mut state);
            } else {
                frame.render_widget(
                    Paragraph::new("  Loading subclasses...")
                        .style(Style::default().fg(Color::DarkGray)),
                    inner,
                );
            }
        }
        LevelUpPrompt::AsiOrFeat { class_name } => {
            let feat_mode = app.asi_feat_mode;
            let title = if feat_mode {
                format!(" Level Up: Choose Feat for {} ", class_name)
            } else {
                format!(" Level Up: ASI or Feat — {} ", class_name)
            };

            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(popup_area);
            frame.render_widget(block, popup_area);

            if feat_mode {
                // Feat picker
                let chunks = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ])
                .split(inner);

                frame.render_widget(
                    Paragraph::new(format!("  Search: {}▌", app.picker_search))
                        .style(Style::default().fg(Color::Yellow)),
                    chunks[1],
                );

                let character = app.active_character.clone();
                let filtered = app.filtered_feats(character, None);
                let items: Vec<ListItem> = filtered
                    .iter()
                    .map(|f| {
                        let tag = if f.prerequisite.is_some() { " *" } else { "" };
                        ListItem::new(format!("  {}{}", f.name, tag))
                            .style(Style::default().fg(Color::White))
                    })
                    .collect();

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(" > ");
                let mut state = ListState::default().with_selected(Some(app.picker_selected));
                frame.render_stateful_widget(list, chunks[2], &mut state);
            } else {
                use crate::utils::ABILITY_NAMES;
                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  [A] +2 to one ability   [S] +1/+1 to two abilities   [D] +1/+1/+1 to three   [F] Choose a Feat instead",
                        Style::default().fg(Color::White),
                    )),
                    Line::from(""),
                ];
                let style_a = if app.asi_choice_index == 0 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let style_b = if app.asi_choice_index == 1 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let style_c = if app.asi_choice_index == 2 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                use crate::app::AsiMode;
                match app.asi_mode {
                    AsiMode::PlusOneThree => {
                        lines.push(Line::from(vec![
                            Span::raw("  +1 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_a]),
                                style_a,
                            ),
                            Span::raw("   +1 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_b]),
                                style_b,
                            ),
                            Span::raw("   +1 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_c]),
                                style_c,
                            ),
                        ]));
                    }
                    AsiMode::PlusOneTwo => {
                        lines.push(Line::from(vec![
                            Span::raw("  +2 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_a]),
                                style_a,
                            ),
                            Span::raw("   +1 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_b]),
                                style_b,
                            ),
                        ]));
                    }
                    AsiMode::PlusTwo => {
                        lines.push(Line::from(vec![
                            Span::raw("  +2 to: "),
                            Span::styled(
                                format!("{:<3}", ABILITY_NAMES[app.asi_ability_a]),
                                style_a,
                            ),
                        ]));
                    }
                }
                lines.push(Line::from("  (Use Tab/Shift+Tab and ↑↓ to change)"));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Enter = confirm   Esc = skip",
                    Style::default().fg(Color::Green),
                )));
                frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
            }
        }
    }
}
