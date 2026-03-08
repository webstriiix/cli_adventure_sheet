use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

use crate::app::App;

const CURRENCY_NAMES: [&str; 5] = ["PP", "GP", "EP", "SP", "CP"];
const CURRENCY_COLORS: [Color; 5] = [
    Color::LightBlue,  // PP - platinum
    Color::Yellow,     // GP - gold
    Color::Gray,       // EP - electrum
    Color::White,      // SP - silver
    Color::Rgb(205, 127, 50), // CP - copper
];

fn get_currency_values(ch: &crate::models::character::Character) -> [i32; 5] {
    [ch.pp, ch.gp, ch.ep, ch.sp, ch.cp]
}

fn render_currency_bar(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    let values = get_currency_values(character);
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("  "));

    for (i, (name, &val)) in CURRENCY_NAMES.iter().zip(values.iter()).enumerate() {
        let is_selected = !app.sidebar_focused && i == app.currency_selected;
        let color = CURRENCY_COLORS[i];

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        };

        spans.push(Span::styled(format!("{}: {}", name, val), style));

        if i < 4 {
            spans.push(Span::styled("  |  ", Style::default().fg(Color::DarkGray)));
        }
    }

    let line = Line::from(spans);
    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);
}

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    // Split into currency bar + hint + item table
    let chunks = Layout::vertical([
        Constraint::Length(1), // currency bar
        Constraint::Length(1), // currency help hint
        Constraint::Min(0),    // item table
    ])
    .split(area);

    render_currency_bar(app, frame, chunks[0]);

    // Currency help hint (only when content panel is focused)
    if !app.sidebar_focused {
        let hint = Line::from(vec![
            Span::raw("  "),
            Span::styled("c", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" cycle  ", Style::default().fg(Color::DarkGray)),
            Span::styled("[/]", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" +/-1  ", Style::default().fg(Color::DarkGray)),
            Span::styled("{/}", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" +/-10", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(hint), chunks[1]);
    }

    let table_area = chunks[2];

    if app.char_inventory.is_empty() {
        let msg = if app.sidebar_focused {
            "  Inventory is empty.\n\n  Items will appear here as they are added."
        } else {
            "  Inventory is empty.\n\n  Press 'a' to add an item."
        };
        let text = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, table_area);
        return;
    }

    let header = Row::new(vec!["Item", "Qty", "Equipped", "Attuned"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .char_inventory
        .iter()
        .map(|item| {
            let name = app.item_name(item.item_id);
            let equipped = if item.is_equipped { "Yes" } else { "No" };
            let attuned = if item.is_attuned { "Yes" } else { "No" };

            Row::new(vec![
                name,
                format!("{}", item.quantity),
                equipped.to_string(),
                attuned.to_string(),
            ])
            .style(Style::default().fg(Color::White))
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(10),
    ];

    let mut table = Table::new(rows, widths).header(header);

    if !app.sidebar_focused {
        table = table
            .row_highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
    }

    app.sheet_table_state.select(Some(app.selected_list_index));
    frame.render_stateful_widget(table, table_area, &mut app.sheet_table_state);
}
