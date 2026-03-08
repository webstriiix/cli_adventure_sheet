use crate::app::App;
use crate::models::{UpdateCharacterRequest, app_state::PickerMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_core_stats_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        // Inspiration toggle
        KeyCode::Char('i') => {
            if let Some(ref mut ch) = app.active_character {
                let new_val = !ch.inspiration;
                let character = ch.clone();
                let req = UpdateCharacterRequest {
                    inspiration: Some(new_val),
                    ..UpdateCharacterRequest::from_character(&character, app.active_class_id)
                };
                let rt = app.rt.clone();
                let id = character.id;
                if let Ok(updated) = rt.block_on(app.client.update_character(id, &req)) {
                    app.active_character = Some(updated);
                    app.status_msg = if new_val {
                        "Inspiration gained!".into()
                    } else {
                        "Inspiration spent.".into()
                    };
                }
            }
        }
        // Death save success: 's'
        KeyCode::Char('s') => {
            if app.death_saves_success < 3 {
                app.death_saves_success += 1;
            } else {
                app.death_saves_success = 0;
            }
            persist_death_saves(app);
            app.status_msg = format!("Death successes: {}", app.death_saves_success);
        }
        // Death save fail: 'd'
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if app.death_saves_fail < 3 {
                app.death_saves_fail += 1;
            } else {
                app.death_saves_fail = 0;
            }
            persist_death_saves(app);
            app.status_msg = format!("Death fails: {}", app.death_saves_fail);
        }
        // Hit dice usage (cycle sizes) - 'h'
        KeyCode::Char('h') => {
            let current_class_hd = app
                .classes
                .iter()
                .find(|c| c.id == app.active_class_id)
                .map(|c| c.hit_die)
                .unwrap_or(8);

            let idx = match current_class_hd {
                6 => 0,
                8 => 1,
                10 => 2,
                12 => 3,
                _ => 1,
            };

            let level = app
                .active_character
                .as_ref()
                .map(|c| crate::utils::level_from_xp(c.experience_pts))
                .unwrap_or(1);

            if (app.hit_dice_used[idx] as i32) < level {
                app.hit_dice_used[idx] += 1;
            } else {
                app.hit_dice_used[idx] = 0;
            }
            persist_hit_dice(app, current_class_hd, app.hit_dice_used[idx]);
        }
        // Short rest: Ctrl+S
        KeyCode::Char('S') => {
            do_short_rest(app);
        }
        // Long rest: Ctrl+L
        KeyCode::Char('L') => {
            do_long_rest(app);
        }
        // Condition picker
        KeyCode::Char('c') => {
            app.picker_mode = PickerMode::ConditionPicker;
            app.picker_selected = 0;
        }
        _ => {}
    }
}

pub fn persist_death_saves(app: &mut App) {
    let id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let rt = app.rt.clone();
    let s = app.death_saves_success as i32;
    let f = app.death_saves_fail as i32;
    if let Ok(updated) = rt.block_on(app.client.patch_death_saves(id, s, f)) {
        app.active_character = Some(updated);
    }
}

pub fn persist_hit_dice(app: &mut App, size: i32, expended: u8) {
    let id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let rt = app.rt.clone();

    let req_expended = expended as i32;

    // Fire-and-forget; ignore error to keep UI responsive
    let _ = rt.block_on(app.client.patch_hit_dice(id, size, req_expended));
}

pub fn do_long_rest(app: &mut App) {
    let id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let rt = app.rt.clone();
    match rt.block_on(app.client.long_rest(id)) {
        Ok(updated) => {
            app.death_saves_success = 0;
            app.death_saves_fail = 0;
            app.spell_slots_used = [0u8; 9];
            app.hit_dice_used = [0u8; 4];
            app.active_character = Some(updated);
            app.status_msg = "Long rest complete! HP and spell slots restored.".to_string();
        }
        Err(e) => app.status_msg = format!("Long rest failed: {e}"),
    }
}

pub fn do_short_rest(app: &mut App) {
    let id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let rt = app.rt.clone();
    match rt.block_on(app.client.short_rest(id)) {
        Ok(updated) => {
            app.active_character = Some(updated);
            app.status_msg = "Short rest complete! Short-rest resources restored.".to_string();
        }
        Err(e) => app.status_msg = format!("Short rest failed: {e}"),
    }
}
