use crate::utils::spell_slots_max;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let character = match app.active_character.as_ref() {
        Some(c) => c.clone(),
        None => return,
    };

    let level = crate::utils::level_from_xp(character.experience_pts);
    let caster_prog = app.char_caster_progression.clone();

    // Check if this class has any spell slots at all
    let is_caster = (0..9).any(|i| spell_slots_max(&caster_prog, level, i) > 0);

    // ── Layout ──────────────────────────────────────────────────────────
    let chunks = Layout::vertical([
        Constraint::Length(if is_caster { 4 } else { 0 }), // spell slot tracker
        Constraint::Min(0),                                // spell list
    ])
    .split(area);

    // ── Spell Slot Tracker ───────────────────────────────────────────────
    if is_caster {
        render_slots(app, frame, chunks[0], &caster_prog, level);
    }

    // ── Spell List ───────────────────────────────────────────────────────
    let list_area = chunks[1];

    // ── Spellcasting stats bar ────────────────────────────────────────────
    let ability_label = app.spellcasting_ability().unwrap_or("").to_uppercase();
    let dc_str = app.spell_save_dc().map(|dc| format!("DC {dc}")).unwrap_or_default();
    let atk_str = app.spell_attack_bonus().map(|b| crate::utils::format_modifier(b)).unwrap_or_default();
    let conc_label = if let Some(sid) = app.concentrating_on {
        let name = app.spell_name(sid);
        format!("  ◈ Concentrating: {name}")
    } else {
        String::new()
    };

    if !ability_label.is_empty() {
        let stats_line = Paragraph::new(Line::from(vec![
            Span::styled("  Spellcasting: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&ability_label, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("   Save ", Style::default().fg(Color::DarkGray)),
            Span::styled(&dc_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("   Attack ", Style::default().fg(Color::DarkGray)),
            Span::styled(&atk_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(&conc_label, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        // Borrow chunk[1] start — render a 1-line stats bar above the table
        let sub = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(list_area);
        frame.render_widget(stats_line, sub[0]);
        // reassign list_area — we need to shadow it below
        let list_area = sub[1];

        render_spell_table(app, frame, list_area);
        return;
    }

    render_spell_table(app, frame, list_area);
}

fn render_spell_table(app: &mut App, frame: &mut Frame, list_area: Rect) {
    if app.char_spells.is_empty() {
        let msg = if app.sidebar_focused {
            "  No spells known.\n\n  Spells will appear here as they are added."
        } else {
            "  No spells known.\n\n  Press 'a' to add a spell."
        };
        let text = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, list_area);
        return;
    }

    let header = Row::new(vec!["Spell", "Lvl", "School", "Conc", "Prep"])
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .char_spells
        .iter()
        .map(|spell| {
            let name = app.spell_name(spell.spell_id);
            let spell_data = app.all_spells.iter().find(|s| s.id == spell.spell_id);
            let lvl_str = spell_data
                .map(|s| {
                    if s.level == 0 {
                        "Cntrp".to_string()
                    } else {
                        s.level.to_string()
                    }
                })
                .unwrap_or_else(|| "?".to_string());
            let school = spell_data
                .map(|s| s.school.clone())
                .unwrap_or_else(|| "?".to_string());
            let is_conc = app.concentrating_on == Some(spell.spell_id);
            let conc_marker = if is_conc { "◈" } else { " " };
            let prepared = if spell.is_prepared { "Yes" } else { "No" };

            let base_color = if is_conc {
                Color::Yellow
            } else if spell.is_prepared {
                Color::Green
            } else {
                Color::White
            };

            Row::new(vec![name, lvl_str, school, conc_marker.to_string(), prepared.to_string()])
                .style(Style::default().fg(base_color))
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(14),
        Constraint::Length(5),
        Constraint::Length(5),
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
    frame.render_stateful_widget(table, list_area, &mut app.sheet_table_state);
}

fn render_slots(app: &App, frame: &mut Frame, area: Rect, caster_prog: &str, level: i32) {
    let label_style = Style::default().fg(Color::DarkGray);
    let used_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let avail_style = Style::default().fg(Color::Cyan);
    let empty_style = Style::default().fg(Color::DarkGray);

    // Header line
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "  SPELL SLOTS  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "[1-9] use slot  [Shift+1-9] recover  [R] long rest",
            label_style,
        ),
    ]));

    // Build slot display lines — show only levels that have slots
    let mut slot_spans: Vec<Span> = vec![Span::raw("  ")];
    let mut has_any = false;

    for slot_idx in 0..9 {
        let max = spell_slots_max(caster_prog, level, slot_idx);
        if max == 0 {
            continue;
        }
        has_any = true;
        let used = app.spell_slots_used[slot_idx].min(max);
        let remaining = max - used;

        // Level label
        slot_spans.push(Span::styled(format!("{}▸ ", slot_idx + 1), label_style));

        // Used slots as ✗ (red), remaining as ● (cyan), empty as ○
        for _ in 0..used {
            slot_spans.push(Span::styled("✗", used_style));
        }
        for _ in 0..remaining {
            slot_spans.push(Span::styled("●", avail_style));
        }
        slot_spans.push(Span::raw("  "));
    }

    if !has_any {
        slot_spans.push(Span::styled("No spell slots at this level", empty_style));
    }

    let slot_line = Paragraph::new(Line::from(slot_spans));

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(header, chunks[0]);
    frame.render_widget(slot_line, chunks[1]);
    // chunks[2] is a spacer
}
