use crate::app::App;
use crate::models::{UpdateCharacterRequest, app_state::PickerMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_inventory_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('a') => {
            // Open item picker
            app.picker_mode = PickerMode::ItemPicker;
            app.picker_search.clear();
            app.picker_selected = 0;
            app.status_msg.clear();
        }
        KeyCode::Char('d') => {
            // Delete selected inventory item
            app.remove_selected_inventory_item();
        }
        KeyCode::Char('e') => app.toggle_inventory_equipped(),
        KeyCode::Char('t') => app.toggle_inventory_attuned(),
        KeyCode::Char('+') => app.update_inventory_quantity(1),
        KeyCode::Char('-') => app.update_inventory_quantity(-1),
        // Currency management:
        //   'c'       — cycle selected currency (PP → GP → EP → SP → CP)
        //   ']' / '[' — increase / decrease selected currency by 1
        //   '}' / '{' — increase / decrease selected currency by 10
        KeyCode::Char('c') => {
            app.currency_selected = (app.currency_selected + 1) % 5;
            let names = ["PP", "GP", "EP", "SP", "CP"];
            app.status_msg = format!("Currency: {}", names[app.currency_selected]);
        }
        KeyCode::Char(']') => adjust_currency(app, 1),
        KeyCode::Char('[') => adjust_currency(app, -1),
        KeyCode::Char('}') => adjust_currency(app, 10),
        KeyCode::Char('{') => adjust_currency(app, -10),
        KeyCode::Up => {
            if app.selected_list_index > 0 {
                app.selected_list_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.char_inventory.is_empty()
                && app.selected_list_index + 1 < app.char_inventory.len()
            {
                app.selected_list_index += 1;
            }
        }
        KeyCode::Char('K') => {
            // Shift+K — toggle item detail modal
            let idx = app.selected_list_index;
            if idx < app.char_inventory.len() {
                let inv_item = &app.char_inventory[idx];
                if app.inventory_item_detail_modal.is_some() {
                    app.inventory_item_detail_modal = None;
                } else if let Some(item) = app.all_items.iter().find(|i| i.id == inv_item.item_id) {
                    let name = item.name.clone();
                    let desc = get_item_description(item);
                    app.inventory_item_detail_modal = Some((name, desc));
                }
            }
        }
        _ => {}
    }
}

fn get_item_description(item: &crate::models::compendium::Item) -> String {
    let mut parts = Vec::new();

    let itype = item.item_type.as_deref().unwrap_or("misc");
    let rarity = item.rarity.as_deref().unwrap_or("none");
    let magic_tag = if item.is_magic.unwrap_or(false) { " ✦ Magic" } else { "" };
    parts.push(format!("Type: {}  |  Rarity: {}{}", itype, rarity, magic_tag));

    let weight_str = item
        .weight
        .as_deref()
        .map(|w| format!("{}lb", w))
        .unwrap_or_else(|| "—".to_string());
    let value_str = match item.value_cp {
        Some(cp) if cp >= 100 => format!("{} gp", cp / 100),
        Some(cp) if cp >= 10 => format!("{} sp", cp / 10),
        Some(cp) => format!("{} cp", cp),
        None => "—".to_string(),
    };
    parts.push(format!("Weight: {}  |  Value: {}", weight_str, value_str));

    // Properties with full descriptions
    if let Some(props) = &item.properties {
        if !props.is_empty() {
            parts.push(String::new());
            parts.push("Properties:".to_string());
            for prop in props {
                let code = crate::utils::weapon_properties::parse_property_code(prop);
                let name = crate::utils::weapon_properties::property_name(code);
                let desc = crate::utils::weapon_properties::property_description(code);
                parts.push(format!("{}. {}", name, desc));
            }
        }
    }

    // Mastery with full descriptions
    if let Some(masteries) = &item.mastery {
        if !masteries.is_empty() {
            parts.push(String::new());
            parts.push("Mastery:".to_string());
            for mastery in masteries {
                let code = crate::utils::weapon_properties::parse_property_code(mastery);
                let name = crate::utils::weapon_mastery::get_mastery_property(code);
                let desc = crate::utils::weapon_mastery::get_mastery_description(name);
                if name != "—" {
                    parts.push(format!("{}: {}", name, desc));
                }
            }
        }
    }

    if item.requires_attune.unwrap_or(false) {
        parts.push("⚠ Requires Attunement".to_string());
    }
    let needs_equip = matches!(itype, "HA" | "MA" | "LA" | "S" | "M" | "R" | "A" | "MNT");
    if needs_equip {
        parts.push("⚔ Must be equipped to use".to_string());
    }

    if let Some(entries) = &item.entries {
        if let Some(arr) = entries.as_array() {
            for entry in arr {
                if let Some(s) = entry.as_str() {
                    parts.push(s.to_string());
                }
            }
        }
    }

    parts.join("\n")
}

pub fn adjust_currency(app: &mut App, delta: i32) {
    let currency_idx = app.currency_selected;
    let currency_name = ["PP", "GP", "EP", "SP", "CP"][currency_idx];
    if let Some(ref mut ch) = app.active_character {
        let field = match currency_idx {
            0 => &mut ch.pp,
            1 => &mut ch.gp,
            2 => &mut ch.ep,
            3 => &mut ch.sp,
            4 => &mut ch.cp,
            _ => return,
        };
        *field = (*field + delta).max(0);
        let new_val = *field;

        // Persist via API
        let character = ch.clone();
        let req = UpdateCharacterRequest {
            pp: Some(character.pp),
            gp: Some(character.gp),
            ep: Some(character.ep),
            sp: Some(character.sp),
            cp: Some(character.cp),
            ..UpdateCharacterRequest::from_character(&character, app.active_class_id)
        };
        let rt = app.rt.clone();
        let id = character.id;
        match rt.block_on(app.client.update_character(id, &req)) {
            Ok(updated) => {
                app.active_character = Some(updated);
                app.status_msg = format!("{}: {}", currency_name, new_val);
            }
            Err(e) => {
                app.status_msg = format!("Failed to update currency: {e}");
            }
        }
    }
}
