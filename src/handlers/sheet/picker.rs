use crate::app::App;
use crate::models::app_state::PickerMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::ALL_CONDITIONS;

pub fn handle_picker_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.picker_mode = PickerMode::None;
            app.show_item_detail = false;
            app.asi_feat_mode = false;
            app.status_msg.clear();
        }
        KeyCode::Enter => match app.picker_mode {
            PickerMode::ItemPicker => app.add_item_from_picker(),
            PickerMode::SpellPicker => app.add_spell_from_picker(),
            PickerMode::FeatPicker => {
                if app.asi_feat_mode {
                    app.confirm_feat_asi_choice();
                    app.asi_feat_mode = false;
                }
            }
            PickerMode::AsiFeatChoice => app.confirm_asi_choice(),
            PickerMode::SubclassPicker => app.confirm_subclass_pick(),
            PickerMode::ConditionPicker => {
                if let Some(&cond) = ALL_CONDITIONS.get(app.picker_selected) {
                    let cond = cond.to_string();
                    if let Some(pos) = app.conditions.iter().position(|c| c == &cond) {
                        app.conditions.remove(pos);
                        app.status_msg = format!("Removed: {cond}");
                    } else {
                        app.conditions.push(cond.clone());
                        app.status_msg = format!("Added: {cond}");
                    }
                }
            }
            PickerMode::WeaponMasteryPicker => {
                let weapons = app.filtered_mastery_weapons();
                if let Some(weapon) = weapons.get(app.picker_selected) {
                    let name = weapon.name.clone();
                    if let Some(pos) = app.char_weapon_masteries.iter().position(|w| w == &name) {
                        app.char_weapon_masteries.remove(pos);
                        app.status_msg = format!("Removed Weapon Mastery: {}", name);
                    } else if app.char_weapon_masteries.len() < 2 {
                        app.char_weapon_masteries.push(name.clone());
                        app.status_msg = format!("Added Weapon Mastery: {}", name);
                    } else {
                        app.status_msg = "Maximum 2 Weapon Masteries selected.".to_string();
                    }
                }
            }
            PickerMode::None => {}
        },
        // ASI-specific keys
        KeyCode::Tab if app.picker_mode == PickerMode::AsiFeatChoice => {
            // Cycle ability A forward
            app.asi_ability_a = (app.asi_ability_a + 1) % 6;
        }
        KeyCode::BackTab if app.picker_mode == PickerMode::AsiFeatChoice => {
            // Cycle ability B backward
            app.asi_ability_b = if app.asi_ability_b == 0 {
                5
            } else {
                app.asi_ability_b - 1
            };
        }
        KeyCode::Up if app.picker_mode == PickerMode::AsiFeatChoice => {
            app.asi_ability_a = if app.asi_ability_a == 0 {
                5
            } else {
                app.asi_ability_a - 1
            };
        }
        KeyCode::Down if app.picker_mode == PickerMode::AsiFeatChoice => {
            app.asi_ability_a = (app.asi_ability_a + 1) % 6;
        }
        KeyCode::Up => {
            if app.picker_selected > 0 {
                app.picker_selected -= 1;
            }
        }
        KeyCode::Down => {
            let max = match app.picker_mode {
                PickerMode::ItemPicker => app.filtered_items().len(),
                PickerMode::SpellPicker => app.filtered_spells().len(),
                PickerMode::FeatPicker => app.filtered_feats(None, None).len(),
                PickerMode::ConditionPicker => ALL_CONDITIONS.len(),
                PickerMode::SubclassPicker => app
                    .class_detail
                    .as_ref()
                    .map(|d| d.subclasses.len())
                    .unwrap_or(0),
                PickerMode::WeaponMasteryPicker => app.filtered_mastery_weapons().len(),
                _ => 0,
            };
            if max > 0 && app.picker_selected + 1 < max {
                app.picker_selected += 1;
            }
        }
        KeyCode::Backspace => {
            if app.picker_mode != PickerMode::AsiFeatChoice {
                app.picker_search.pop();
                app.picker_selected = 0;
            }
        }
        KeyCode::Char('a') if app.picker_mode == PickerMode::AsiFeatChoice => {
            app.asi_mode = crate::app::AsiMode::PlusOneTwo;
        }
        KeyCode::Char('s') if app.picker_mode == PickerMode::AsiFeatChoice => {
            app.asi_mode = crate::app::AsiMode::PlusOneThree;
        }
        KeyCode::Char('f') if app.picker_mode == PickerMode::AsiFeatChoice => {
            // Switch to feat picker (in ASI context — uses available-feats endpoint)
            app.asi_feat_mode = true;
            app.picker_mode = PickerMode::FeatPicker;
            app.picker_search.clear();
            app.picker_selected = 0;
            // Pre-fetch available feats from API and store in all_feats for filtering
            if let Some(char_id) = app.active_character.as_ref().map(|c| c.id) {
                let rt = app.rt.clone();
                if let Ok(feats) = rt.block_on(app.client.get_available_feats(char_id)) {
                    app.all_feats = feats;
                }
            }
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.picker_mode == PickerMode::ItemPicker {
                app.show_item_detail = !app.show_item_detail;
            }
        }
        KeyCode::Char(c) => {
            if app.picker_mode != PickerMode::AsiFeatChoice {
                app.picker_search.push(c);
                app.picker_selected = 0;
            }
        }
        _ => {}
    }
}
