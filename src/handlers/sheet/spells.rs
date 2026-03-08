use crate::app::App;
use crate::models::app_state::PickerMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_spells_key(app: &mut App, key: KeyEvent) {
    // If spell detail modal is open, any key closes it
    if app.spell_detail_modal.is_some() {
        app.spell_detail_modal = None;
        return;
    }

    // Check for Shift+K first so it isn't swallowed by other handlers
    if key.code == KeyCode::Char('K')
        || (key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::SHIFT))
    {
        open_spell_detail_modal(app);
        return;
    }

    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('a') => {
            app.picker_mode = PickerMode::SpellPicker;
            app.picker_search.clear();
            app.picker_selected = 0;
            app.status_msg.clear();
        }
        KeyCode::Char('d') => {
            app.remove_selected_spell();
        }
        KeyCode::Char('p') => {
            app.toggle_spell_prepared();
        }
        // Expending spell slots: 1-9
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let slot_idx = c.to_digit(10).unwrap() as usize - 1;
            let level = app
                .active_character
                .as_ref()
                .map(|ch| crate::utils::level_from_xp(ch.experience_pts))
                .unwrap_or(1);
            let max = crate::utils::spell_slots_max(&app.char_caster_progression, level, slot_idx);

            if max > 0 && app.spell_slots_used[slot_idx] < max {
                app.spell_slots_used[slot_idx] += 1;
                persist_spell_slot(app, slot_idx);
            }
        }
        // Recovering spell slots: shift+1-9 using characters
        KeyCode::Char(c) if "!@#$%^&*()".contains(c) => {
            let shift_chars = ")!@#$%^&*(";
            if let Some(num) = shift_chars.find(c) {
                if num > 0 && num <= 9 {
                    let slot_idx = num - 1;
                    let level = app
                        .active_character
                        .as_ref()
                        .map(|ch| crate::utils::level_from_xp(ch.experience_pts))
                        .unwrap_or(1);
                    let max = crate::utils::spell_slots_max(&app.char_caster_progression, level, slot_idx);

                    if max > 0 && app.spell_slots_used[slot_idx] > 0 {
                        app.spell_slots_used[slot_idx] -= 1;
                        persist_spell_slot(app, slot_idx);
                    }
                }
            }
        }
        // Concentrate on selected spell (z to toggle)
        KeyCode::Char('z') => {
            let spell_id = app
                .char_spells
                .get(app.selected_list_index)
                .map(|s| s.spell_id);
            if let Some(sid) = spell_id {
                if app.concentrating_on == Some(sid) {
                    app.concentrating_on = None;
                    app.status_msg = "Concentration dropped.".to_string();
                } else {
                    app.concentrating_on = Some(sid);
                    let name = app.spell_name(sid);
                    app.status_msg = format!("Concentrating on {name}.");
                }
            }
        }
        KeyCode::Up => {
            if app.selected_list_index > 0 {
                app.selected_list_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.char_spells.is_empty() && app.selected_list_index + 1 < app.char_spells.len() {
                app.selected_list_index += 1;
            }
        }
        _ => {}
    }
}

pub fn persist_spell_slot(app: &mut App, slot_idx: usize) {
    let id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let rt = app.rt.clone();
    let level = (slot_idx + 1) as i32;
    let expended = app.spell_slots_used[slot_idx] as i32;
    match rt.block_on(app.client.patch_spell_slot(id, level, expended)) {
        Ok(_) => {}
        Err(e) => {
            app.status_msg = format!("Failed to save spell slot: {e}");
        }
    }
}

fn open_spell_detail_modal(app: &mut App) {
    if app.char_spells.is_empty() {
        return;
    }

    let char_spell = match app.char_spells.get(app.selected_list_index) {
        Some(s) => s,
        None => return,
    };

    if let Some(spell) = app.all_spells.iter().find(|s| s.id == char_spell.spell_id) {
        let name = app.spell_name(spell.id);

        // Build description from spell data
        let mut parts: Vec<String> = Vec::new();

        // Spell metadata
        let level_str = if spell.level == 0 {
            "Cantrip".to_string()
        } else {
            format!("Level {}", spell.level)
        };
        let school = match spell.school.as_str() {
            "A" => "Abjuration",
            "C" => "Conjuration",
            "D" => "Divination",
            "E" => "Enchantment",
            "V" => "Evocation",
            "I" => "Illusion",
            "N" => "Necromancy",
            "T" => "Transmutation",
            _ => &spell.school,
        };
        parts.push(format!("{} — {}", level_str, school));

        // Casting time
        if let Some(ct) = &spell.casting_time {
            let ct_str: Vec<String> = ct
                .iter()
                .filter_map(|v| {
                    let num = v.get("number").and_then(|n| n.as_i64()).unwrap_or(1);
                    let unit = v.get("unit").and_then(|u| u.as_str()).unwrap_or("action");
                    Some(format!("{} {}", num, unit))
                })
                .collect();
            if !ct_str.is_empty() {
                parts.push(format!("Casting Time: {}", ct_str.join(", ")));
            }
        }

        // Range
        if let Some(range) = &spell.range {
            let range_str = if let Some(dist) = range.get("distance") {
                let amount = dist.get("amount").and_then(|a| a.as_i64());
                let rtype = dist.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match amount {
                    Some(a) => format!("{} {}", a, rtype),
                    None => rtype.to_string(),
                }
            } else {
                range
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Self")
                    .to_string()
            };
            parts.push(format!("Range: {}", range_str));
        } else {
            parts.push("Range: Self".to_string());
        }

        // Components
        if let Some(comp) = &spell.components {
            let mut comp_parts: Vec<String> = Vec::new();
            if comp.get("v").and_then(|v| v.as_bool()).unwrap_or(false) {
                comp_parts.push("V".to_string());
            }
            if comp.get("s").and_then(|v| v.as_bool()).unwrap_or(false) {
                comp_parts.push("S".to_string());
            }
            if let Some(m) = comp.get("m") {
                if let Some(s) = m.as_str() {
                    comp_parts.push(format!("M ({})", s));
                } else if m.as_bool().unwrap_or(false) {
                    comp_parts.push("M".to_string());
                }
            }
            if !comp_parts.is_empty() {
                parts.push(format!("Components: {}", comp_parts.join(", ")));
            }
        }

        // Duration
        if let Some(dur) = &spell.duration {
            let dur_str: Vec<String> = dur
                .iter()
                .filter_map(|v| {
                    let dtype = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match dtype {
                        "instant" => Some("Instantaneous".to_string()),
                        "timed" => {
                            let amount = v
                                .get("duration")
                                .and_then(|d| d.get("amount"))
                                .and_then(|a| a.as_i64())
                                .unwrap_or(0);
                            let unit = v
                                .get("duration")
                                .and_then(|d| d.get("type"))
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            let conc = if v
                                .get("concentration")
                                .and_then(|c| c.as_bool())
                                .unwrap_or(false)
                            {
                                "Concentration, "
                            } else {
                                ""
                            };
                            Some(format!("{}{} {}", conc, amount, unit))
                        }
                        "permanent" => Some("Until dispelled".to_string()),
                        "special" => Some("Special".to_string()),
                        _ => Some(dtype.to_string()),
                    }
                })
                .collect();
            if !dur_str.is_empty() {
                parts.push(format!("Duration: {}", dur_str.join(", ")));
            }
        }

        // Concentration / Ritual tags
        let mut tags: Vec<&str> = Vec::new();
        if spell.concentration.unwrap_or(false) {
            tags.push("Concentration");
        }
        if spell.ritual.unwrap_or(false) {
            tags.push("Ritual");
        }
        if !tags.is_empty() {
            parts.push(tags.join(", "));
        }

        parts.push(String::new()); // blank line before description

        // Entries (spell description)
        if let Some(entries) = &spell.entries {
            for entry in entries {
                match entry {
                    serde_json::Value::String(s) => {
                        parts.push(crate::ui::sheet::features::strip_tags(s));
                    }
                    other => {
                        let mut sub = Vec::new();
                        crate::ui::sheet::features::extract_entry_text_pub(other, &mut sub);
                        parts.extend(sub);
                    }
                }
            }
        }

        // Higher level entries
        if let Some(higher) = &spell.entries_higher_lvl {
            if let Some(arr) = higher.as_array() {
                for entry in arr {
                    if let Some(entries_inner) = entry.get("entries").and_then(|e| e.as_array()) {
                        parts.push(String::new());
                        parts.push("At Higher Levels:".to_string());
                        for e in entries_inner {
                            if let Some(s) = e.as_str() {
                                parts.push(crate::ui::sheet::features::strip_tags(s));
                            }
                        }
                    }
                }
            }
        }

        let desc = parts.join("\n");
        app.spell_detail_modal = Some((name, desc));
    }
}
