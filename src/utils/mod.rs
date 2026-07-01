pub mod storage;
pub mod weapon_mastery;
pub mod weapon_properties;
use ratatui::layout::{Constraint, Layout, Rect};

/// Strip 5e-tools {@tag content|source} markup, keeping only the text content.
pub fn strip_entry_tags(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'@') {
            let mut inner = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
            }
            let without_at = inner.trim_start_matches('@');
            if let Some(space_idx) = without_at.find(' ') {
                let display_part = &without_at[space_idx + 1..];
                let display = display_part
                    .splitn(2, '|')
                    .next()
                    .unwrap_or(display_part)
                    .trim();
                if !display.is_empty() {
                    out.push_str(display);
                }
            } else {
                let tag = without_at.trim();
                let readable = match tag {
                    "initiative" => "initiative roll",
                    "dice" => "roll",
                    "hit" => "attack roll",
                    "damage" => "damage roll",
                    _ => tag,
                };
                out.push_str(readable);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Recursively flatten a 5e-tools JSON entries array into plain text strings.
/// Handles: plain strings, {type:"entries"} sections, {type:"list"} bullet lists.
pub fn entries_to_lines(entries: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(arr) = entries.as_array() {
        for entry in arr {
            collect_entry(entry, &mut out);
        }
    }
    out
}

fn collect_entry(entry: &serde_json::Value, out: &mut Vec<String>) {
    match entry {
        serde_json::Value::String(s) => {
            let cleaned = strip_entry_tags(s);
            if !cleaned.trim().is_empty() {
                out.push(cleaned);
            }
        }
        serde_json::Value::Object(o) => {
            let t = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match t {
                "entries" => {
                    if let Some(name) = o.get("name").and_then(|v| v.as_str()) {
                        out.push(format!("{}:", name));
                    }
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            collect_entry(e, out);
                        }
                    }
                }
                "list" => {
                    if let Some(items) = o.get("items").and_then(|v| v.as_array()) {
                        for item in items {
                            let mut sub = Vec::new();
                            collect_entry(item, &mut sub);
                            for s in sub {
                                out.push(format!("• {}", s));
                            }
                        }
                    }
                }
                "item" => {
                    // Named list item: bold name + description
                    let name = o.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let mut sub = Vec::new();
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            collect_entry(e, &mut sub);
                        }
                    }
                    if !name.is_empty() {
                        let detail = sub.join(" ");
                        out.push(format!("{}: {}", name, detail));
                    } else {
                        out.extend(sub);
                    }
                }
                _ => {
                    // Fallback: try entries or items
                    if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                        for e in arr {
                            collect_entry(e, out);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
