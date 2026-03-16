use crate::models::character::Character;

pub const ABILITY_NAMES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
pub const STANDARD_ARRAY: [i32; 6] = [15, 14, 13, 12, 10, 8];

// D&D 5e XP thresholds for levels 1-20
pub const XP_THRESHOLDS: [i32; 20] = [
    0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000, 120000, 140000,
    165000, 195000, 225000, 265000, 305000, 355000,
];

pub fn level_from_xp(xp: i32) -> i32 {
    for (i, &threshold) in XP_THRESHOLDS.iter().enumerate().rev() {
        if xp >= threshold {
            return (i + 1) as i32;
        }
    }
    1
}

pub fn xp_from_level(level: i32) -> i32 {
    let idx = (level - 1).clamp(0, 19) as usize;
    XP_THRESHOLDS[idx]
}

pub fn proficiency_bonus(level: i32) -> i32 {
    match level {
        1..=4 => 2,
        5..=8 => 3,
        9..=12 => 4,
        13..=16 => 5,
        17..=20 => 6,
        _ => 2,
    }
}

pub fn ability_modifier(score: i32) -> i32 {
    (score - 10).div_euclid(2)
}

pub fn format_modifier(m: i32) -> String {
    if m >= 0 {
        format!("+{m}")
    } else {
        format!("{m}")
    }
}

pub fn ability_name(index: usize) -> &'static str {
    ABILITY_NAMES[index]
}

/// Returns the maximum number of prepared spells for a given class and level.
/// Defaults to Level + Modifier for unknown classes, but uses the 2024 PHB fixed table for Paladins.
pub fn max_prepared_spells(class_name: &str, level: i32, modifier: i32) -> i32 {
    match class_name.to_lowercase().as_str() {
        "paladin" => match level {
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 5,
            5..=6 => 6,
            7..=8 => 7,
            9..=10 => 8,
            11..=12 => 10,
            13..=14 => 11,
            15..=16 => 12,
            17..=18 => 14,
            19..=20 => 15,
            _ => 15,
        },
        // Fallback for others (2014 style or generic)
        _ => level.max(1) + modifier,
    }
}

pub fn standard_array_value(index: usize) -> i32 {
    STANDARD_ARRAY[index]
}

/// Returns the max spell slots for a given caster progression, level, and slot index (0=1st level).
/// Uses the standard D&D 5e spell slot table.
/// `caster_progression` should be one of: "full", "1/2", "1/3", "pact", "artificer", or a
/// class name for backwards compatibility.
pub fn spell_slots_max(caster_progression: &str, char_level: i32, slot_idx: usize) -> u8 {
    let caster_level = match caster_progression.to_lowercase().as_str() {
        // Progression-based (from API caster_progression field)
        "full" => char_level,
        "1/2" => (char_level + 1) / 2, // 2024 rules: Level 1 -> 1, Level 2 -> 1, Level 3 -> 2
        "1/3" => (char_level + 2) / 3, // 2024 rules: Level 1 -> 1, Level 4 -> 2
        "pact" => char_level, // pact magic handled separately; use same table for simplicity
        "artificer" => char_level, // artificer has its own progression similar to full
        // Legacy class name fallback
        "wizard" | "sorcerer" | "cleric" | "druid" | "bard" => char_level,
        "paladin" | "ranger" => (char_level + 1) / 2,
        "warlock" => char_level,
        "fighter" | "rogue" => (char_level + 2) / 3,
        _ => 0,
    };
    if caster_level < 1 {
        return 0;
    }
    // Standard full-caster slot table [level][slot_idx 0..8]
    const FULL_CASTER: [[u8; 9]; 20] = [
        [2, 0, 0, 0, 0, 0, 0, 0, 0], // 1
        [3, 0, 0, 0, 0, 0, 0, 0, 0], // 2
        [4, 2, 0, 0, 0, 0, 0, 0, 0], // 3
        [4, 3, 0, 0, 0, 0, 0, 0, 0], // 4
        [4, 3, 2, 0, 0, 0, 0, 0, 0], // 5
        [4, 3, 3, 0, 0, 0, 0, 0, 0], // 6
        [4, 3, 3, 1, 0, 0, 0, 0, 0], // 7
        [4, 3, 3, 2, 0, 0, 0, 0, 0], // 8
        [4, 3, 3, 3, 1, 0, 0, 0, 0], // 9
        [4, 3, 3, 3, 2, 0, 0, 0, 0], // 10
        [4, 3, 3, 3, 2, 1, 0, 0, 0], // 11
        [4, 3, 3, 3, 2, 1, 0, 0, 0], // 12
        [4, 3, 3, 3, 2, 1, 1, 0, 0], // 13
        [4, 3, 3, 3, 2, 1, 1, 0, 0], // 14
        [4, 3, 3, 3, 2, 1, 1, 1, 0], // 15
        [4, 3, 3, 3, 2, 1, 1, 1, 0], // 16
        [4, 3, 3, 3, 2, 1, 1, 1, 1], // 17
        [4, 3, 3, 3, 3, 1, 1, 1, 1], // 18
        [4, 3, 3, 3, 3, 2, 1, 1, 1], // 19
        [4, 3, 3, 3, 3, 2, 2, 1, 1], // 20
    ];
    // Half-casters use same table but at half level
    let idx = (caster_level as usize).saturating_sub(1).min(19);
    if slot_idx >= 9 {
        return 0;
    }
    FULL_CASTER[idx][slot_idx]
}

pub fn ch_ability_score(ch: &Character, key: &str) -> i32 {
    match key {
        "str" => ch.strength,
        "dex" => ch.dexterity,
        "con" => ch.constitution,
        "int" => ch.intelligence,
        "wis" => ch.wisdom,
        "cha" => ch.charisma,
        _ => 0,
    }
}

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
