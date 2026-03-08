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
        _ => {}
    }
}

pub fn adjust_currency(app: &mut App, delta: i32) {
    let names = ["PP", "GP", "EP", "SP", "CP"];

    if let Some(ref mut ch) = app.active_character {
        let field = match app.currency_selected {
            0 => &mut ch.pp,
            1 => &mut ch.gp,
            2 => &mut ch.ep,
            3 => &mut ch.sp,
            4 => &mut ch.cp,
            _ => return,
        };
        *field = (*field + delta).max(0);
        let new_val = *field;
        let currency_name = names[app.currency_selected];

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
