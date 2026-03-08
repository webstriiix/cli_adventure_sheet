use crate::app::App;
use crate::models::app_state::CharacterCreationStep;
use crate::models::compendium::source_id_label;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);

    let title = Paragraph::new(" Character Builder: Background Bonus Feat ")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, outer[0]);

    let body = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(outer[1]);

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
    let filtered: Vec<_> = app
        .all_feats
        .iter()
        .filter(|f| {
            f.prerequisite.is_none()
                && (search.is_empty() || f.name.to_lowercase().contains(&search))
        })
        .collect();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|f| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("  {}", f.name)),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(f.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    use ratatui::widgets::ListState;
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Choose Background Feat "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    let mut state = ListState::default().with_selected(Some(app.builder.feat_picker_index));
    frame.render_stateful_widget(list, search_chunks[2], &mut state);

    // Detail panel
    let detail_lines = if let Some(feat) = filtered.get(app.builder.feat_picker_index) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    feat.name.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("[{}]", source_id_label(feat.source_id)),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
        ];
        if feat.prerequisite.is_some() {
            lines.push(Line::from(Span::styled(
                "* Has prerequisites",
                Style::default().fg(Color::Red),
            )));
            lines.push(Line::from(""));
        }
        // Render all entries recursively (handles strings, lists, sections)
        for line in crate::utils::entries_to_lines(&feat.entries) {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines
    } else {
        vec![Line::from(Span::styled(
            "No feats found",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let detail = Paragraph::new(detail_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Feat Details "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, body[1]);

    let help =
        "↑↓ select  Enter confirm  Tab skip (no bg feat)  Esc back to background".to_string();
    let help_w = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_w, outer[2]);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let search = app.builder.feat_picker_search.to_lowercase();
    let count = app
        .all_feats
        .iter()
        .filter(|f| {
            f.prerequisite.is_none()
                && (search.is_empty() || f.name.to_lowercase().contains(&search))
        })
        .count();

    match key.code {
        KeyCode::Esc => {
            // Back to background selection
            let idx = app
                .builder
                .bg_id
                .and_then(|id| app.backgrounds.iter().position(|b| b.id == id))
                .unwrap_or(0);
            app.builder.list_state.select(Some(idx));
            app.builder.step = CharacterCreationStep::Background;
        }
        KeyCode::Up => {
            if app.builder.feat_picker_index > 0 {
                app.builder.feat_picker_index -= 1;
            }
        }
        KeyCode::Down => {
            if app.builder.feat_picker_index + 1 < count {
                app.builder.feat_picker_index += 1;
            }
        }
        KeyCode::Char(c) => {
            app.builder.feat_picker_search.push(c);
            app.builder.feat_picker_index = 0;
        }
        KeyCode::Backspace => {
            app.builder.feat_picker_search.pop();
            app.builder.feat_picker_index = 0;
        }
        KeyCode::Tab => {
            // Skip — no background feat
            app.builder.background_feat_id = None;
            app.builder.feat_picker_search.clear();
            app.builder.feat_picker_index = 0;
            if app.builder.language_count > 0 {
                app.builder.step = CharacterCreationStep::Languages;
            } else {
                app.builder.step = CharacterCreationStep::Proficiencies;
            }
            app.builder.list_state.select(Some(0));
        }
        KeyCode::Enter => {
            let feat_id = {
                let s = app.builder.feat_picker_search.to_lowercase();
                app.all_feats
                    .iter()
                    .filter(|f| {
                        f.prerequisite.is_none()
                            && (s.is_empty() || f.name.to_lowercase().contains(&s))
                    })
                    .nth(app.builder.feat_picker_index)
                    .map(|f| f.id)
            };
            app.builder.background_feat_id = feat_id;
            app.builder.feat_picker_search.clear();
            app.builder.feat_picker_index = 0;
            if app.builder.language_count > 0 {
                app.builder.step = CharacterCreationStep::Languages;
            } else {
                app.builder.step = CharacterCreationStep::Proficiencies;
            }
            app.builder.list_state.select(Some(0));
            app.status_msg = "Background feat selected.".to_string();
        }
        _ => {}
    }
}
