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
            app.screen = Screen::CharacterBuilder;
        }
        KeyCode::Up => {
            if app.selected_char > 0 {
                app.selected_char -= 1;
                app.char_list_state.select(Some(app.selected_char));
            }
        }
        KeyCode::Down => {
            if !app.characters.is_empty() && app.selected_char + 1 < app.characters.len() {
                app.selected_char += 1;
                app.char_list_state.select(Some(app.selected_char));
            }
        }
        KeyCode::Enter => {
            if !app.characters.is_empty() {
                let character = app.characters[app.selected_char].clone();
                app.load_character_sheet(character.id);
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
            app.char_list_state.select(Some(app.selected_char));
            app.status_msg = "Character deleted.".to_string();
        }
        Err(e) => {
            app.status_msg = format!("Delete failed: {e}");
        }
    }
}
