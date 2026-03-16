use crate::app::App;
use crate::models::compendium::Spell;
use crate::utils::spell_slots_max;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

// ── Spell level tab labels ─────────────────────────────────────────────────
const LEVEL_TABS: &[&str] = &["All", "- 0 -", "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th"];

// ── Main render entry ──────────────────────────────────────────────────────
pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let character = match app.active_character.as_ref() {
        Some(c) => c.clone(),
        None => return,
    };

    let level = crate::utils::level_from_xp(character.experience_pts);
    let caster_prog = app.char_caster_progression.clone();
    let is_caster = (0..9).any(|i| spell_slots_max(&caster_prog, level, i) > 0);

    // Determine how many header rows we need
    let stats_height = 4u16; // bordered boxes for modifier/attack/dc
    let slots_height = if is_caster { 2u16 } else { 0 };
    let tabs_height = 1u16;
    let header_total = stats_height + slots_height + tabs_height;

    let chunks = Layout::vertical([
        Constraint::Length(header_total),
        Constraint::Min(0),
    ])
    .split(area);

    render_header(app, frame, chunks[0], &caster_prog, level, is_caster);
    render_spell_list(app, frame, chunks[1]);
}

// ── Header: stats + slots + tabs ───────────────────────────────────────────
fn render_header(app: &App, frame: &mut Frame, area: Rect, caster_prog: &str, level: i32, is_caster: bool) {
    let stats_height = 4u16;
    let slots_height = if is_caster { 2u16 } else { 0 };
    let tabs_height = 1u16;

    let chunks = Layout::vertical([
        Constraint::Length(stats_height),
        Constraint::Length(slots_height),
        Constraint::Length(tabs_height),
    ])
    .split(area);

    // ── Multi-class spellcasting stats ──────────────────────────────────
    render_stats(app, frame, chunks[0]);

    // ── Spell slots ─────────────────────────────────────────────────────
    if is_caster {
        render_slots(app, frame, chunks[1], caster_prog, level);
    }

    // ── Level tabs ──────────────────────────────────────────────────────
    render_level_tabs(app, frame, chunks[2]);
}

fn render_stats(app: &App, frame: &mut Frame, area: Rect) {
    let classes = app.spellcasting_classes();
    if classes.is_empty() {
        return;
    }

    let border_style = Style::default().fg(Color::DarkGray);
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

    // Build box data: (title, value_string) for each stat
    // Combine multi-class values with " / "
    let mod_values: Vec<String> = classes.iter().map(|(_, m, _, _)| crate::utils::format_modifier(*m)).collect();
    let atk_values: Vec<String> = classes.iter().map(|(_, _, a, _)| a.to_string()).collect();
    let dc_values: Vec<String> = classes.iter().map(|(_, _, _, d)| d.to_string()).collect();

    // Prepared count: show (current / max)
    let character = app.active_character.as_ref().unwrap();
    let level = crate::utils::level_from_xp(character.experience_pts);
    let always_prepared = app.always_prepared_spell_ids();
    let prepared_current = app
        .char_spells
        .iter()
        .filter(|s| s.is_prepared && !always_prepared.contains(&s.spell_id))
        .count();
    let prepared_values: Vec<String> = classes
        .iter()
        .map(|(name, m, _, _)| {
            let max = crate::utils::max_prepared_spells(name, level, *m);
            format!("{}/{}", prepared_current, max)
        })
        .collect();

    let boxes: Vec<(&str, String)> = vec![
        ("Modifier", mod_values.join(" / ")),
        ("Spell Attack", atk_values.join(" / ")),
        ("Save DC", dc_values.join(" / ")),
        ("Prepared", prepared_values.join(" / ")),
    ];

    let box_count = boxes.len();
    let box_width: u16 = 16;
    let gap: u16 = 1;
    let total_width = box_width * box_count as u16 + gap * (box_count as u16 - 1);

    // Center horizontally
    let left_pad = area.width.saturating_sub(total_width) / 2;

    for (i, (label, value)) in boxes.iter().enumerate() {
        let x = area.x + left_pad + (box_width + gap) * i as u16;
        let box_area = Rect::new(x, area.y, box_width, 4);

        if box_area.x + box_area.width > area.x + area.width {
            break;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);
        let inner = block.inner(box_area);
        frame.render_widget(block, box_area);

        // Label on first line, value centered on second line
        let content = Paragraph::new(vec![
            Line::from(Span::styled(format!(" {}", label), label_style)),
            Line::from(Span::styled(value.clone(), value_style)),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(content, inner);
    }
}

fn render_slots(app: &App, frame: &mut Frame, area: Rect, caster_prog: &str, level: i32) {
    let label_style = Style::default().fg(Color::DarkGray);
    let used_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let avail_style = Style::default().fg(Color::Cyan);

    let mut spans: Vec<Span> = vec![
        Span::styled("  Spell Slots  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];

    for slot_idx in 0..9 {
        let max = spell_slots_max(caster_prog, level, slot_idx);
        if max == 0 {
            continue;
        }
        let used = app.spell_slots_used[slot_idx].min(max);
        let remaining = max - used;

        spans.push(Span::styled(format!("{}▸ ", slot_idx + 1), label_style));
        for _ in 0..used {
            spans.push(Span::styled("✗", used_style));
        }
        for _ in 0..remaining {
            spans.push(Span::styled("●", avail_style));
        }
        spans.push(Span::raw("  "));
    }

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);

    frame.render_widget(Paragraph::new(Line::from(spans)), rows[0]);
    // rows[1] is spacer
}

fn render_level_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let active_tab = app.spell_level_tab_index;
    let mut spans: Vec<Span> = vec![Span::raw("  ")];

    // Only show tabs up to the max level the character has spells for
    let max_spell_level = app.char_spells.iter().filter_map(|cs| {
        app.all_spells.iter().find(|s| s.id == cs.spell_id).map(|s| s.level)
    }).max().unwrap_or(0);

    let max_tab = std::cmp::min((max_spell_level + 1) as usize + 1, LEVEL_TABS.len());
    let max_tab = std::cmp::max(max_tab, 2); // Always show at least "All" and "- 0 -"

    for (i, label) in LEVEL_TABS.iter().enumerate().take(max_tab) {
        if i == active_tab {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::raw(" "));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

// ── Spell list with level grouping ─────────────────────────────────────────
fn render_spell_list(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.char_spells.is_empty() {
        let msg = if app.sidebar_focused {
            "  No spells known.\n\n  Spells will appear here as they are added."
        } else {
            "  No spells known.\n\n  Press 'a' to add a spell."
        };
        let text = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
        return;
    }

    // Build a sorted list of (level, spell_data, char_spell) tuples
    let spell_entries: Vec<(i32, &Spell, bool, bool)> = app
        .char_spells_filtered()
        .into_iter()
        .map(|cs| {
            let spell = app.all_spells.iter().find(|s| s.id == cs.spell_id).unwrap();
            let is_conc = app.concentrating_on == Some(cs.spell_id);
            (spell.level, spell, cs.is_prepared, is_conc)
        })
        .collect();

    // Get spell attack bonus for Hit/DC column
    let spell_atk = app.spell_attack_bonus();

    // Build rows with level group headers, tracking which table row
    // corresponds to each spell so we can map selected_list_index correctly.
    let mut rows: Vec<Row> = Vec::new();
    let mut current_level: Option<i32> = None;
    // spell_index_to_table_row[i] = table row index for the i-th filtered spell
    let mut spell_index_to_table_row: Vec<usize> = Vec::new();

    for (_spell_idx, (lvl, spell, is_prepared, is_conc)) in spell_entries.iter().enumerate() {
        // Insert level group header if level changed
        if current_level != Some(*lvl) {
            current_level = Some(*lvl);
            let header_label = level_group_label(*lvl);
            rows.push(
                Row::new(vec![header_label, String::new(), String::new(), String::new(), String::new(), String::new()])
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .bottom_margin(0),
            );
            // Column headers for this group
            rows.push(
                Row::new(vec![
                    "Name".to_string(),
                    "Time".to_string(),
                    "Range".to_string(),
                    "Hit / DC".to_string(),
                    "Effect".to_string(),
                    "Notes".to_string(),
                ])
                .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)),
            );
        }

        // Record the table row index for this spell
        spell_index_to_table_row.push(rows.len());

        let name = spell.name.clone();
        let time = format_casting_time(spell);
        let range = format_range(spell);
        let hit_dc = if spell.school == "V" || spell.school == "E" {
            spell_atk.map(|a| crate::utils::format_modifier(a)).unwrap_or_else(|| "--".to_string())
        } else {
            "--".to_string()
        };
        let effect = "--".to_string();
        let notes = format_notes(spell);

        let base_color = if *is_conc {
            Color::Yellow
        } else if *is_prepared {
            Color::Green
        } else {
            Color::White
        };

        rows.push(
            Row::new(vec![name, time, range, hit_dc, effect, notes])
                .style(Style::default().fg(base_color)),
        );
    }

    let widths = [
        Constraint::Min(18),       // Name
        Constraint::Length(6),     // Time
        Constraint::Length(8),     // Range
        Constraint::Length(8),     // Hit / DC
        Constraint::Length(10),    // Effect
        Constraint::Min(12),       // Notes
    ];

    let mut table = Table::new(rows, widths);

    if !app.sidebar_focused {
        table = table
            .row_highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
    }

    // Map the logical spell index to the actual table row index
    let table_row = spell_index_to_table_row
        .get(app.selected_list_index)
        .copied()
        .unwrap_or(0);
    app.sheet_table_state.select(Some(table_row));
    frame.render_stateful_widget(table, area, &mut app.sheet_table_state);
}

// ── Helper: level group header label ───────────────────────────────────────
fn level_group_label(level: i32) -> String {
    match level {
        0 => "── Cantrip ──".to_string(),
        1 => "── 1st Level ──".to_string(),
        2 => "── 2nd Level ──".to_string(),
        3 => "── 3rd Level ──".to_string(),
        n => format!("── {}th Level ──", n),
    }
}

// ── Helper: format casting time ────────────────────────────────────────────
fn format_casting_time(spell: &Spell) -> String {
    if let Some(ct) = &spell.casting_time {
        if let Some(first) = ct.first() {
            let num = first.get("number").and_then(|n| n.as_i64()).unwrap_or(1);
            let unit = first.get("unit").and_then(|u| u.as_str()).unwrap_or("action");
            return match unit {
                "action" => format!("{}A", num),
                "bonus" => format!("{}BA", num),
                "reaction" => format!("{}R", num),
                "minute" => format!("{}m", num),
                "hour" => format!("{}h", num),
                other => format!("{}{}", num, other),
            };
        }
    }
    "1A".to_string()
}

// ── Helper: format range ───────────────────────────────────────────────────
fn format_range(spell: &Spell) -> String {
    if let Some(range) = &spell.range {
        if let Some(rtype) = range.get("type").and_then(|t| t.as_str()) {
            match rtype {
                "point" => {
                    if let Some(dist) = range.get("distance") {
                        let amount = dist.get("amount").and_then(|a| a.as_i64());
                        let dtype = dist.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        return match (amount, dtype) {
                            (Some(a), "feet") => format!("{}ft.", a),
                            (Some(a), "miles") => format!("{}mi.", a),
                            (None, "touch") => "Touch".to_string(),
                            (None, "self") => "Self".to_string(),
                            (Some(a), t) => format!("{}{}", a, t),
                            _ => dtype.to_string(),
                        };
                    }
                }
                "special" => return "Special".to_string(),
                "self" => return "Self".to_string(),
                _ => {}
            }
        }
    }
    "Self".to_string()
}

// ── Helper: format notes (components + duration) ───────────────────────────
fn format_notes(spell: &Spell) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Duration prefix
    if let Some(dur) = &spell.duration {
        if let Some(first) = dur.first() {
            let dtype = first.get("type").and_then(|t| t.as_str()).unwrap_or("");
            match dtype {
                "timed" => {
                    let amount = first
                        .get("duration")
                        .and_then(|d| d.get("amount"))
                        .and_then(|a| a.as_i64())
                        .unwrap_or(0);
                    let unit = first
                        .get("duration")
                        .and_then(|d| d.get("type"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    let unit_short = match unit {
                        "round" => "Rnd",
                        "minute" => "m",
                        "hour" => "h",
                        "day" => "d",
                        other => other,
                    };
                    if first.get("concentration").and_then(|c| c.as_bool()).unwrap_or(false) {
                        parts.push(format!("C: {}{}", amount, unit_short));
                    } else {
                        parts.push(format!("D: {}{}", amount, unit_short));
                    }
                }
                _ => {}
            }
        }
    }

    // Components
    if let Some(comp) = &spell.components {
        let mut comp_parts: Vec<&str> = Vec::new();
        if comp.get("v").and_then(|v| v.as_bool()).unwrap_or(false) {
            comp_parts.push("V");
        }
        if comp.get("s").and_then(|v| v.as_bool()).unwrap_or(false) {
            comp_parts.push("S");
        }
        if comp.get("m").is_some() {
            comp_parts.push("M");
        }
        if !comp_parts.is_empty() {
            parts.push(comp_parts.join("/"));
        }
    }

    parts.join(", ")
}
