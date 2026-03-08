mod actions;
mod common;
mod core_stats;
mod features;
mod inventory;
mod notes;
mod picker;
mod spells;

use crate::app::App;
use crate::models::app_state::{PickerMode, Screen, SheetTab};
use crossterm::event::{KeyCode, KeyEvent};

pub use actions::{change_limited_use, handle_actions_key, open_action_detail_modal};
pub use common::handle_default_content_key;
pub use features::handle_features_key;
pub use core_stats::{
    do_long_rest, do_short_rest, handle_core_stats_key, persist_death_saves, persist_hit_dice,
};
pub use inventory::{adjust_currency, handle_inventory_key};
pub use notes::{handle_notes_edit_key, handle_notes_key};
pub use picker::handle_picker_key;
pub use spells::{handle_spells_key, persist_spell_slot};

pub const ALL_CONDITIONS: &[&str] = &[
    "Blinded",
    "Charmed",
    "Deafened",
    "Exhaustion",
    "Frightened",
    "Grappled",
    "Incapacitated",
    "Invisible",
    "Paralyzed",
    "Petrified",
    "Poisoned",
    "Prone",
    "Restrained",
    "Stunned",
    "Unconscious",
];

pub fn handle_sheet_key(app: &mut App, key: KeyEvent) {
    // Handle notes editing mode
    if app.editing_notes {
        handle_notes_edit_key(app, key);
        return;
    }

    // Handle picker mode (item/spell/feat/subclass/ASI)
    if app.picker_mode != PickerMode::None {
        handle_picker_key(app, key);
        return;
    }

    // Drain any queued level-up prompts (subclass / ASI choice)
    if !app.level_up_queue.is_empty() {
        app.drain_level_up_queue();
        return; // Let the overlay render first before processing other keys
    }

    // CoreStats global keys — work regardless of sidebar focus
    if app.sheet_tab == SheetTab::CoreStats {
        match key.code {
            KeyCode::Char('i')
            | KeyCode::Char('s')
            | KeyCode::Char('S')
            | KeyCode::Char('d')
            | KeyCode::Char('D')
            | KeyCode::Char('c') => {
                handle_core_stats_key(app, key);
                return;
            }
            _ => {}
        }
    }

    if app.sidebar_focused {
        match key.code {
            KeyCode::Esc => {
                app.fetch_characters();
                app.screen = Screen::CharacterList;
            }
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(character) = app.active_character.clone() {
                    app.open_edit_character(&character, true);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.sheet_tab_index > 0 {
                    app.sheet_tab_index -= 1;
                    app.sheet_tab = SheetTab::ALL[app.sheet_tab_index];
                    app.content_scroll = 0;
                    app.selected_list_index = 0;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.sheet_tab_index + 1 < SheetTab::ALL.len() {
                    app.sheet_tab_index += 1;
                    app.sheet_tab = SheetTab::ALL[app.sheet_tab_index];
                    app.content_scroll = 0;
                    app.selected_list_index = 0;
                }
            }
            KeyCode::Enter | KeyCode::Right => {
                app.sidebar_focused = false;
                app.selected_list_index = 0;
            }
            _ => {}
        }
    } else {
        // Content panel focused — dispatch per tab
        match app.sheet_tab {
            SheetTab::Notes => handle_notes_key(app, key),
            SheetTab::Inventory => handle_inventory_key(app, key),
            SheetTab::Spells => handle_spells_key(app, key),
            SheetTab::Features => handle_features_key(app, key),
            SheetTab::CoreStats => handle_core_stats_key(app, key),
            SheetTab::Actions => handle_actions_key(app, key),
            _ => handle_default_content_key(app, key),
        }
    }
}
