use crate::app::App;
use crate::models::app_state::Screen;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_char_list_key(app: &mut App, key: KeyEvent) {
    // Handle delete confirmation popup
    if app.delete_confirm {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.delete_confirm = false;
                delete_selected_character(app);
            }
            _ => {
                app.delete_confirm = false;
                app.status_msg = "Delete cancelled".to_string();
            }
        }
        return;
    }

    match key.code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.status_msg.clear();
            app.builder = crate::models::app_state::BuilderState::default();
            // Ensure compendium data (including weapons) is loaded for builder
            if app.all_items.is_empty() {
                app.fetch_compendium_data();
            }
            app.screen = Screen::CharacterBuilder;
        }
        KeyCode::Up => {
            if app.selected_char > 0 {
                app.selected_char -= 1;
                let idx = app.selected_char;
                app.char_list_state.select(Some(idx));
            }
        }
        KeyCode::Down => {
            if !app.characters.is_empty() && app.selected_char + 1 < app.characters.len() {
                app.selected_char += 1;
                let idx = app.selected_char;
                app.char_list_state.select(Some(idx));
            }
        }
        KeyCode::Enter => {
            if !app.characters.is_empty() {
                let character = app.characters[app.selected_char].clone();
                if character.name.starts_with("[DRAFT]") {
                    // Check active connection before resuming a draft
                    if !app.is_online() {
                        app.status_msg = "Offline! Cannot resume draft without an active connection.".to_string();
                        return;
                    }

                    let mut loaded_draft = None;
                    if let Some(ref notes) = character.notes {
                        if let Ok(d) = serde_json::from_str::<crate::models::CharacterDraft>(notes) {
                            loaded_draft = Some(d);
                        }
                    }

                    app.builder = crate::models::app_state::BuilderState::default();
                    app.builder.draft_id = Some(character.id);

                    if let Some(d) = loaded_draft {
                        app.builder.step = match d.current_step {
                            1 => crate::models::app_state::CharacterCreationStep::Class,
                            2 => crate::models::app_state::CharacterCreationStep::Background,
                            3 => crate::models::app_state::CharacterCreationStep::Species,
                            4 => crate::models::app_state::CharacterCreationStep::Abilities,
                            5 => crate::models::app_state::CharacterCreationStep::Equipment,
                            _ => crate::models::app_state::CharacterCreationStep::Class,
                        };
                        app.builder.class_id = d.class_id;
                        app.builder.level = d.level;
                        app.builder.subclass_id = d.subclass_id;
                        app.builder.name = d.name;
                        app.builder.trait_text = d.personality;
                        app.builder.bg_id = d.background_id;
                        app.builder.background_feat_id = d.background_feat_id;
                        app.builder.race_id = d.species_id;
                        app.builder.lineage_id = d.lineage_id;
                        app.builder.abilities = d.abilities;
                        app.builder.equipment_option = d.equipment_option;
                    }

                    if app.all_items.is_empty() {
                        app.fetch_compendium_data();
                    }
                    app.screen = Screen::CharacterBuilder;
                    app.status_msg = "Draft resumed.".to_string();
                } else {
                    app.load_character_sheet(character.id);
                }
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if !app.characters.is_empty() {
                let character = app.characters[app.selected_char].clone();
                app.open_edit_character(&character, false);
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if !app.characters.is_empty() {
                app.delete_confirm = true;
                app.status_msg.clear();
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.fetch_characters();
            app.status_msg = "Refreshed".to_string();
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.logout();
        }
        _ => {}
    }
}

pub fn delete_selected_character(app: &mut App) {
    if app.characters.is_empty() {
        return;
    }
    let id = app.characters[app.selected_char].id;
    let rt = app.rt.clone();
    match rt.block_on(app.client.delete_character(id)) {
        Ok(()) => {
            app.characters.remove(app.selected_char);
            if app.selected_char > 0 && app.selected_char >= app.characters.len() {
                app.selected_char -= 1;
            }
            let idx = app.selected_char;
            app.char_list_state.select(Some(idx));
            app.status_msg = "Character deleted.".to_string();
        }
        Err(e) => {
            app.status_msg = format!("Delete failed: {e}");
        }
    }
}
