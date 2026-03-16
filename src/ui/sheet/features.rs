use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};

use crate::app::{App, FeaturesSubTab};
use crate::models::features::Feature;

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(2), // Nav bar
        Constraint::Min(0),    // Content
    ])
    .split(area);

    render_nav_bar(app, frame, chunks[0]);

    match app.features_sub_tab {
        FeaturesSubTab::All => render_all(app, frame, chunks[1]),
        FeaturesSubTab::ClassFeatures => render_class_features(app, frame, chunks[1]),
        FeaturesSubTab::SpeciesTraits => render_species_traits(app, frame, chunks[1]),
        FeaturesSubTab::Feats => render_feats(app, frame, chunks[1]),
    }
}

fn render_nav_bar(app: &App, frame: &mut Frame, area: Rect) {
    let tabs = vec![
        (FeaturesSubTab::All, " All "),
        (FeaturesSubTab::ClassFeatures, " Class Features "),
        (FeaturesSubTab::SpeciesTraits, " Species Traits "),
        (FeaturesSubTab::Feats, " Feats "),
    ];

    let mut line = Vec::new();
    line.push(Span::raw("   ")); // padding

    for (idx, (tab, label)) in tabs.into_iter().enumerate() {
        let style = if app.features_sub_tab == tab {
            Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        line.push(Span::styled(label, style));
        if idx < 3 {
            line.push(Span::raw(" | "));
        }
    }

    let p = Paragraph::new(Line::from(line));
    frame.render_widget(p, area);
}

fn render_all(app: &mut App, frame: &mut Frame, area: Rect) {
    let has_features = !app.char_class_features.is_empty();
    let has_traits = !app.char_race_traits.is_empty();
    let has_feats = !app.char_feats.is_empty();

    if !has_features && !has_traits && !has_feats {
        let msg = "  No features or traits.\n\n  Features will appear here as they are gained.";
        let subclass_line = if !app.char_subclass_name.is_empty() {
            format!("  Subclass: {}\n\n", app.char_subclass_name)
        } else {
            String::new()
        };
        let full = format!("{}{}", subclass_line, msg);
        let text = Paragraph::new(full).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(text, area);
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();

    // Subclass banner
    if !app.char_subclass_name.is_empty() {
        let line = Line::from(vec![
            Span::styled("  Subclass: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.char_subclass_name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        items.push(ListItem::new(line));
        items.push(ListItem::new(Line::from("")));
    }

    // === CLASS FEATURES (names only) ===
    if has_features {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  ── {} Class Features ──", app.char_class_name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))));
        for f in &app.char_class_features {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    • "),
                Span::styled(
                    format!("{} (Lv {})", f.name, f.level),
                    Style::default().fg(Color::White),
                ),
            ])));
        }
        items.push(ListItem::new(Line::from("")));
    }

    // === SPECIES TRAITS (names only) ===
    if has_traits {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  ── {} Species Traits ──", app.char_race_name),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))));
        for (name, _) in &app.char_race_traits {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    • "),
                Span::styled(name.clone(), Style::default().fg(Color::White)),
            ])));
        }
        items.push(ListItem::new(Line::from("")));
    }

    // === FEATS ===
    if has_feats {
        items.push(ListItem::new(Line::from(Span::styled(
            "  ── Feats ──",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))));
        for feat in &app.char_feats {
            let name = app.feat_name(feat.feat_id);
            let level = feat
                .gained_at_level
                .map(|l| format!(" (Lv {})", l))
                .unwrap_or_default();
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    • "),
                Span::styled(
                    format!("{}{}", name, level),
                    Style::default().fg(Color::White),
                ),
            ])));
        }
    }

    let list = List::new(items);
    frame.render_widget(list, area);
}

fn render_class_features(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.char_class_features.is_empty() {
        let msg = "  No class features found.\n  Make sure the class data has been imported.";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let mut total_lines: Vec<Line> = Vec::new();

    // Section header
    total_lines.push(Line::from(Span::styled(
        format!("  === {} FEATURES ===", app.char_class_name.to_uppercase()),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    total_lines.push(Line::from(""));

    for f in &app.char_class_features {
        let feature = f.interpret();
        let badge = feature_badge(&feature);
        let entries_text = extract_class_feature_text(&f.entries);

        let mut header_spans = vec![Span::styled(
            format!("* {} • Lv {}", f.name, f.level),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )];
        if let Some((label, color)) = badge {
            header_spans.push(Span::raw(" "));
            header_spans.push(Span::styled(
                label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }
        let mut lines = vec![Line::from(header_spans)];

        if !entries_text.is_empty() {
            lines.extend(desc_to_lines(&entries_text, "  "));
        } else {
            lines.push(Line::from(Span::styled(
                "  (No description available from compendium)",
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::from("")); // spacing

        total_lines.extend(lines);
    }

    let p = Paragraph::new(total_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.content_scroll as u16, 0));
    frame.render_widget(p, area);
}

fn render_species_traits(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.char_race_traits.is_empty() {
        let msg = "  No species traits found.\n  Make sure the race data has been imported.";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let mut total_lines: Vec<Line> = Vec::new();

    // Section header
    total_lines.push(Line::from(Span::styled(
        format!(
            "  === {} SPECIES TRAITS ===",
            app.char_race_name.to_uppercase()
        ),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    total_lines.push(Line::from(""));

    for (name, desc) in &app.char_race_traits {
        let mut lines = vec![Line::from(Span::styled(
            format!("* {}", name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))];

        if !desc.is_empty() {
            lines.extend(desc_to_lines(desc, "  "));
        }
        lines.push(Line::from("")); // spacing

        total_lines.extend(lines);
    }

    let p = Paragraph::new(total_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.content_scroll as u16, 0));
    frame.render_widget(p, area);
}

fn render_feats(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.char_feats.is_empty() {
        let msg = "  No feats yet.\n  Feats are gained at ASI milestones, from your background, or from your species.";
        frame.render_widget(
            Paragraph::new(msg).style(Style::default().fg(Color::DarkGray)),
            area,
        );
        return;
    }

    let mut total_lines: Vec<Line> = Vec::new();

    total_lines.push(Line::from(Span::styled(
        "  === FEATS ===",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    total_lines.push(Line::from(""));

    for f in &app.char_feats {
        let title = app.feat_name(f.feat_id);
        let source = match f.source_type.as_str() {
            "background" => "Background",
            "race" | "species" => "Species",
            "level" => "Level-up",
            other => other,
        };
        let level_part = f
            .gained_at_level
            .map(|l| format!(" • Lv {}", l))
            .unwrap_or_default();

        let desc = get_feat_desc(app, f.feat_id);

        // Get badge from interpreter
        let badge = app
            .all_feats
            .iter()
            .find(|feat| feat.id == f.feat_id)
            .map(|feat| feature_badge(&feat.interpret()));

        let mut header_spans = vec![Span::styled(
            format!("* {} ({}{}) ", title, source, level_part),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )];
        if let Some(Some((label, color))) = badge {
            header_spans.push(Span::styled(
                label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }

        let mut lines = vec![Line::from(header_spans)];

        if !desc.is_empty() {
            lines.extend(desc_to_lines(&desc, "  "));
        }

        // Show uses if applicable
        let max = f.uses_max.unwrap_or(0);
        if max > 0 {
            let rem = f.uses_remaining.unwrap_or(0);
            let recharge = f.recharge_on.as_deref().unwrap_or("?");
            lines.push(Line::from(Span::styled(
                format!("   | {}/{} uses • recharge on {}", rem, max, recharge),
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines.push(Line::from("")); // spacing
        total_lines.extend(lines);
    }

    let p = Paragraph::new(total_lines)
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((app.content_scroll as u16, 0));
    frame.render_widget(p, area);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Map a [`Feature`] to a short label + colour for display in the sheet.
/// Returns `None` for [`Feature::StaticFeat`] (no badge needed).
fn feature_badge(feature: &Feature) -> Option<(&'static str, Color)> {
    match feature {
        Feature::Asi { .. } => Some(("[ASI]", Color::Magenta)),
        Feature::WeaponMastery { .. } => Some(("[WM]", Color::Yellow)),
        Feature::SkillChoice { .. } => Some(("[Skill]", Color::Green)),
        Feature::GrantsOriginFeat { .. } => Some(("[Feat]", Color::Cyan)),
        Feature::Choice { .. } => Some(("[Pick]", Color::LightBlue)),
        Feature::GrantsSpell { .. } => Some(("[Spell]", Color::LightBlue)),
        Feature::StaticFeat(_) => None,
    }
}

/// Strip 5e-tools {@tag text|source} markup, keeping readable display text.
///
/// Tag formats:
/// - `{@dice 1d20}`            → "1d20"
/// - `{@ability DEX}`          → "DEX"
/// - `{@skill Perception}`     → "Perception"
/// - `{@condition incapacitated}` → "incapacitated"
/// - `{@initiative}`           → "initiative"
/// - `{@hit +5}`               → "+5"
/// - `{@b bold text}`          → "bold text"
/// - `{@i italic text}`        → "italic text"
/// - `{@creature Goblin}`      → "Goblin"
/// - `{@item Sword|PHB}`       → "Sword"
pub fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'@') {
            // Collect full tag content until matching '}'
            let mut inner = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
            }
            // inner = "@tag rest" e.g. "@dice 1d20", "@skill Perception|PHB", "@b bold"
            // Split off the tag keyword (everything after the '@' up to first space)
            let without_at = inner.trim_start_matches('@');
            if let Some(space_idx) = without_at.find(' ') {
                // There is display text after the tag keyword
                let display_part = &without_at[space_idx + 1..];
                // Take only the part before the first '|' (drop source reference)
                let display = display_part
                    .splitn(2, '|')
                    .next()
                    .unwrap_or(display_part)
                    .trim();
                if !display.is_empty() {
                    out.push_str(display);
                }
            } else {
                // No space → bare tag like {@initiative} or {@dice}
                // Use the tag keyword itself as a human-readable placeholder
                let tag_name = without_at.trim();
                let readable = match tag_name {
                    "initiative" => "initiative roll",
                    "dice" => "roll",
                    "hit" => "attack roll",
                    "damage" => "damage roll",
                    "d20" => "d20",
                    _ => tag_name,
                };
                out.push_str(readable);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Word-wrap a string to `max_width` chars, returning lines with `indent` prefix.
fn wrap_text(text: &str, indent: &str, max_width: usize) -> Vec<String> {
    let usable = max_width.saturating_sub(indent.len());
    if text.len() <= usable {
        return vec![format!("{}{}", indent, text)];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= usable {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(format!("{}{}", indent, current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(format!("{}{}", indent, current));
    }
    lines
}

/// Recursively extract readable text from a JSON entry value.
/// Public wrapper for extracting readable text from a JSON entry value.
pub fn extract_entry_text_pub(entry: &serde_json::Value, parts: &mut Vec<String>) {
    extract_entry_text(entry, parts);
}

fn extract_entry_text(entry: &serde_json::Value, parts: &mut Vec<String>) {
    match entry {
        serde_json::Value::String(s) => {
            let cleaned = strip_tags(s);
            if !cleaned.is_empty() {
                parts.push(cleaned);
            }
        }
        serde_json::Value::Array(arr) => {
            for e in arr {
                extract_entry_text(e, parts);
            }
        }
        serde_json::Value::Object(o) => {
            let entry_type = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match entry_type {
                "entries" | "inset" | "inlineBlock" | "inline" | "section" => {
                    if let Some(name) = o.get("name").and_then(|v| v.as_str()) {
                        parts.push(format!("{}:", name));
                    }
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            extract_entry_text(e, parts);
                        }
                    }
                }
                "list" => {
                    if let Some(arr) = o.get("items").and_then(|v| v.as_array()) {
                        for item in arr {
                            let mut sub = Vec::new();
                            extract_entry_text(item, &mut sub);
                            for s in sub {
                                parts.push(format!("• {}", s));
                            }
                        }
                    }
                }
                "item" => {
                    // Name/entry pairs (e.g. feature sub-items)
                    let name = o.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(entry_val) = o.get("entry") {
                        let mut sub = Vec::new();
                        extract_entry_text(entry_val, &mut sub);
                        if !name.is_empty() {
                            parts.push(format!("{}: {}", name, sub.join(" ")));
                        } else {
                            parts.extend(sub);
                        }
                    } else if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        if !name.is_empty() {
                            parts.push(format!("{}:", name));
                        }
                        for e in arr {
                            extract_entry_text(e, parts);
                        }
                    } else if !name.is_empty() {
                        parts.push(name.to_string());
                    }
                }
                "options" => {
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            extract_entry_text(e, parts);
                        }
                    }
                }
                "table" => {
                    // Render table caption if present, then rows as text
                    if let Some(caption) = o.get("caption").and_then(|v| v.as_str()) {
                        parts.push(format!("[{}]", caption));
                    }
                    if let Some(rows) = o.get("rows").and_then(|v| v.as_array()) {
                        for row in rows {
                            if let Some(cells) = row.as_array() {
                                let cell_texts: Vec<String> = cells
                                    .iter()
                                    .map(|c| match c {
                                        serde_json::Value::String(s) => strip_tags(s),
                                        other => {
                                            let mut sub = Vec::new();
                                            extract_entry_text(other, &mut sub);
                                            sub.join(" ")
                                        }
                                    })
                                    .collect();
                                parts.push(cell_texts.join("  |  "));
                            }
                        }
                    }
                }
                "abilityDc" => {
                    let name = o
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Save DC");
                    let attrs: Vec<String> = o
                        .get("attributes")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|a| a.as_str().map(|s| s.to_uppercase()))
                                .collect()
                        })
                        .unwrap_or_default();
                    parts.push(format!(
                        "{} = 8 + proficiency bonus + {} modifier",
                        name,
                        attrs.join("/")
                    ));
                }
                "abilityAttackMod" => {
                    let name = o
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Attack modifier");
                    let attrs: Vec<String> = o
                        .get("attributes")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|a| a.as_str().map(|s| s.to_uppercase()))
                                .collect()
                        })
                        .unwrap_or_default();
                    parts.push(format!(
                        "{} = proficiency bonus + {} modifier",
                        name,
                        attrs.join("/")
                    ));
                }
                "quote" => {
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            let mut sub = Vec::new();
                            extract_entry_text(e, &mut sub);
                            for s in sub {
                                parts.push(format!("\"{}\"", s));
                            }
                        }
                    }
                }
                _ => {
                    // Generic fallback: try "entries" or "items" or "name"
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        if let Some(name) = o.get("name").and_then(|v| v.as_str()) {
                            parts.push(format!("{}:", name));
                        }
                        for e in arr {
                            extract_entry_text(e, parts);
                        }
                    } else if let Some(arr) = o.get("items").and_then(|v| v.as_array()) {
                        for e in arr {
                            extract_entry_text(e, parts);
                        }
                    } else if let Some(name) = o.get("name").and_then(|v| v.as_str()) {
                        parts.push(format!("  - {}", name));
                    }
                }
            }
        }
        _ => {}
    }
}

/// Extract flat text from class feature entries (Vec<JsonValue>).
fn extract_class_feature_text(entries: &Option<Vec<serde_json::Value>>) -> String {
    let arr = match entries {
        Some(a) => a,
        None => return String::new(),
    };
    let mut parts: Vec<String> = Vec::new();
    for e in arr {
        extract_entry_text(e, &mut parts);
    }
    parts.join("\n")
}

fn get_feat_desc(app: &App, feat_id: i32) -> String {
    let feat_db = app.all_feats.iter().find(|f| f.id == feat_id);
    if let Some(f) = feat_db {
        if let Some(arr) = f.entries.as_array() {
            let mut parts: Vec<String> = Vec::new();
            for e in arr {
                extract_entry_text(e, &mut parts);
            }
            parts.join("\n")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

/// Turn a raw description string into word-wrapped `Line` items, with `indent` prefix.
pub fn desc_to_lines<'a>(text: &str, indent: &str) -> Vec<Line<'a>> {
    const MAX_WIDTH: usize = 90;
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let raw_line = raw_line.trim_end();
        if raw_line.is_empty() {
            lines.push(Line::from(""));
        } else {
            for wrapped in wrap_text(raw_line, indent, MAX_WIDTH) {
                lines.push(Line::from(wrapped));
            }
        }
    }
    lines
}
