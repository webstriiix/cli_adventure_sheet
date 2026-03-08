use crate::App;
use crate::models::app_state::PickerMode;
use crate::models::character::{AddSpellRequest, UpdateSpellRequest};
use crate::models::compendium::Spell;

impl App {
    pub fn remove_selected_spell(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_spells.is_empty() {
            return;
        }

        let spell = &self.char_spells[self.selected_list_index];
        let spell_id = spell.spell_id;

        let rt = self.rt.clone();
        match rt.block_on(self.client.remove_spell(character_id, spell_id)) {
            Ok(()) => {
                self.char_spells.remove(self.selected_list_index);
                if self.selected_list_index > 0
                    && self.selected_list_index >= self.char_spells.len()
                {
                    self.selected_list_index -= 1;
                }
                self.status_msg = "Spell removed".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Failed to remove spell: {e}");
            }
        }
    }

    pub fn toggle_spell_prepared(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_spells.is_empty() {
            return;
        }

        let spell = &self.char_spells[self.selected_list_index];
        let spell_id = spell.spell_id;
        let new_prepared = !spell.is_prepared;

        let req = UpdateSpellRequest {
            is_prepared: new_prepared,
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.update_spell(character_id, spell_id, &req)) {
            Ok(updated) => {
                self.char_spells[self.selected_list_index] = updated;
                self.status_msg = if new_prepared {
                    "Spell prepared".to_string()
                } else {
                    "Spell unprepared".to_string()
                };
            }
            Err(e) => {
                self.status_msg = format!("Failed to update spell: {e}");
            }
        }
    }

    pub fn add_spell_from_picker(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        let filtered = self.filtered_spells();
        if filtered.is_empty() {
            return;
        }

        let spell_id = filtered[self.picker_selected].id;
        let req = AddSpellRequest {
            spell_id,
            is_prepared: Some(false),
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.add_spell(character_id, &req)) {
            Ok(char_spell) => {
                self.char_spells.push(char_spell);
                self.status_msg = "Spell added!".to_string();
                self.picker_mode = PickerMode::None;
            }
            Err(e) => {
                self.status_msg = format!("Failed to add spell: {e}");
            }
        }
    }

    pub fn filtered_spells(&self) -> Vec<&Spell> {
        let search = self.picker_search.to_lowercase();
        self.all_spells
            .iter()
            .filter(|s| search.is_empty() || s.name.to_lowercase().contains(&search))
            .take(50)
            .collect()
    }
}
