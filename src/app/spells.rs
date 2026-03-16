use crate::App;
use crate::models::app_state::PickerMode;
use crate::models::character::{AddSpellRequest, CharacterSpell, UpdateSpellRequest};
use crate::models::compendium::Spell;

impl App {
    /// Returns the character's spells filtered by the current level filter and sorted by level then name.
    pub fn char_spells_filtered(&self) -> Vec<&CharacterSpell> {
        let filter = self.spell_level_filter;
        let mut entries: Vec<(i32, String, &CharacterSpell)> = self
            .char_spells
            .iter()
            .filter_map(|cs| {
                let spell = self.all_spells.iter().find(|s| s.id == cs.spell_id)?;
                if let Some(lvl) = filter {
                    if spell.level != lvl {
                        return None;
                    }
                }
                Some((spell.level, spell.name.clone(), cs))
            })
            .collect();

        // Sort by level (ascending), then name (ascending)
        entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        entries.into_iter().map(|(_, _, cs)| cs).collect()
    }

    pub fn remove_selected_spell(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        let filtered = self.char_spells_filtered();
        if filtered.is_empty() || self.selected_list_index >= filtered.len() {
            return;
        }

        let spell_id = filtered[self.selected_list_index].spell_id;

        let rt = self.rt.clone();
        match rt.block_on(self.client.remove_spell(character_id, spell_id)) {
            Ok(()) => {
                // Find and remove from the main list
                if let Some(pos) = self.char_spells.iter().position(|s| s.spell_id == spell_id) {
                    self.char_spells.remove(pos);
                }

                // Adjust selection index if needed
                let new_filtered_len = self.char_spells_filtered().len();
                if self.selected_list_index > 0 && self.selected_list_index >= new_filtered_len {
                    self.selected_list_index = new_filtered_len.saturating_sub(1);
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

        let filtered = self.char_spells_filtered();
        if filtered.is_empty() || self.selected_list_index >= filtered.len() {
            return;
        }

        let spell_id = filtered[self.selected_list_index].spell_id;
        let is_prepared = filtered[self.selected_list_index].is_prepared;
        let new_prepared = !is_prepared;

        let req = UpdateSpellRequest {
            is_prepared: new_prepared,
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.update_spell(character_id, spell_id, &req)) {
            Ok(updated) => {
                // Update in the main list
                if let Some(pos) = self.char_spells.iter().position(|s| s.spell_id == spell_id) {
                    self.char_spells[pos] = updated;
                }

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
        let always_prepared = self.always_prepared_spell_ids();
        
        self.all_spells
            .iter()
            .filter(|s| {
                // Exclude if already always prepared by a feature
                if always_prepared.contains(&s.id) {
                    return false;
                }
                search.is_empty() || s.name.to_lowercase().contains(&search)
            })
            .take(50)
            .collect()
    }
}
