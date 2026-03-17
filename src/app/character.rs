use ratatui::widgets::ListState;
use uuid::Uuid;

use crate::App;
use crate::models::app_state::{EditSection, Screen, SheetTab};
use crate::models::character::Character;
use crate::utils::storage::FullCharacterCache;

impl App {
    pub fn open_edit_character(&mut self, character: &Character, return_to_sheet: bool) {
        self.edit_character_id = Some(character.id);
        self.edit_return_to_sheet = return_to_sheet;
        self.edit_section = EditSection::Fields;
        self.edit_field_index = 0;

        self.edit_buffers[0] = character.name.clone();
        self.edit_buffers[1] = character.experience_pts.to_string();
        self.edit_buffers[2] = crate::utils::level_from_xp(character.experience_pts).to_string();
        self.edit_buffers[3] = character.max_hp.to_string();
        self.edit_buffers[4] = character.current_hp.to_string();
        self.edit_buffers[5] = character.temp_hp.to_string();
        self.edit_buffers[6] = character.strength.to_string();
        self.edit_buffers[7] = character.dexterity.to_string();
        self.edit_buffers[8] = character.constitution.to_string();
        self.edit_buffers[9] = character.intelligence.to_string();
        self.edit_buffers[10] = character.wisdom.to_string();
        self.edit_buffers[11] = character.charisma.to_string();
        self.edit_buffers[12] = if character.inspiration {
            "Yes".to_string()
        } else {
            "No".to_string()
        };

        self.edit_race_index = self
            .races
            .iter()
            .position(|r| Some(r.id) == character.race_id)
            .unwrap_or(0);
        self.edit_race_state = ListState::default();
        self.edit_race_state.select(Some(self.edit_race_index));

        let current_class_id = self.active_class_id;

        self.edit_class_index = self
            .classes
            .iter()
            .position(|c| c.id == current_class_id)
            .unwrap_or(0);
        self.edit_class_state = ListState::default();
        self.edit_class_state.select(Some(self.edit_class_index));

        self.edit_bg_index = if let Some(bg_id) = character.background_id {
            self.backgrounds
                .iter()
                .position(|b| b.id == bg_id)
                .unwrap_or(0)
        } else {
            0
        };
        self.edit_bg_state = ListState::default();
        self.edit_bg_state.select(Some(self.edit_bg_index));

        self.multiclass_selected = 0;
        self.multiclass_section = crate::models::app_state::MulticlassSection::List;
        self.screen = Screen::EditCharacter;
        self.status_msg = "Editing character".to_string();
    }

    pub fn load_character_sheet(&mut self, character_id: Uuid) {
        let rt = self.rt.clone();
        self.status_msg = "Loading character sheet...".into();

        // 1. Try to fetch from API
        match rt.block_on(self.client.get_character(character_id)) {
            Ok(c) => {
                // Find class source slug for detail fetch
                let first_class_id = c.class_id.unwrap_or(1);
                let (class_name, class_source) = self.classes.iter()
                    .find(|cl| cl.id == first_class_id)
                    .map(|cl| (cl.name.clone(), cl.source_slug.clone()))
                    .unwrap_or_else(|| ("Unknown".into(), "PHB".into()));

                // Parallel fetch all related data
                let (feats, spells, inventory, slots, hit_dice, detail, actions) = rt.block_on(async {
                    tokio::join!(
                        self.client.get_feats(c.id),
                        self.client.get_character_spells(c.id),
                        self.client.get_inventory(c.id),
                        self.client.get_spell_slots(c.id),
                        self.client.get_hit_dice(c.id),
                        self.client.get_class_detail(&class_name, &class_source),
                        self.client.get_character_actions(c.id)
                    )
                });

                let cache = FullCharacterCache {
                    character: c.clone(),
                    feats: feats.unwrap_or_default(),
                    spells: spells.unwrap_or_default(),
                    inventory: inventory.unwrap_or_default(),
                    spell_slots: slots.unwrap_or_default(),
                    hit_dice: hit_dice.unwrap_or_default(),
                    class_detail: detail.ok(),
                    actions: actions.ok(),
                };

                // Save to local cache for offline use
                let filename = format!("char_{}.json", character_id);
                self.storage.save_cache(&filename, &cache);

                self.apply_character_data(cache);
                self.is_offline = false;
                self.status_msg = "Character sheet loaded.".into();
            }
            Err(e) => {
                // 2. Fallback to local cache
                let filename = format!("char_{}.json", character_id);
                if let Some(cache) = self.storage.load_cache::<FullCharacterCache>(&filename) {
                    self.apply_character_data(cache);
                    self.is_offline = true;
                    self.status_msg = "Loaded from local cache (Offline).".into();
                } else {
                    self.status_msg = format!("Failed to load character: {}", e);
                }
            }
        }
    }

    /// Internal helper to populate the App state with a character's data.
    fn apply_character_data(&mut self, cache: FullCharacterCache) {
        let c = cache.character;
        let first_class_id = c.class_id.unwrap_or(1);

        self.active_character = Some(c.clone());
        self.active_class_id = first_class_id;
        self.char_feats = cache.feats;
        self.char_spells = cache.spells;
        self.char_inventory = cache.inventory;
        self.char_actions = cache.actions;
        self.class_detail = cache.class_detail;

        // Names
        self.char_race_name = self.races.iter()
            .find(|r| Some(r.id) == c.race_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "Unknown".into());

        let active_class = self.classes.iter().find(|cl| cl.id == first_class_id);
        self.char_class_name = active_class.map(|cl| cl.name.clone()).unwrap_or_else(|| "Unknown".into());
        self.char_caster_progression = active_class.and_then(|cl| cl.caster_progression.clone()).unwrap_or_default();

        self.char_bg_name = self.backgrounds.iter()
            .find(|b| Some(b.id) == c.background_id)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "None".into());

        // Skills
        let mut skills = Vec::new();
        if let Some(bg) = self.backgrounds.iter().find(|b| Some(b.id) == c.background_id) {
            if let Some(prof) = &bg.skill_proficiencies {
                for entry in prof {
                    if let Some(s) = entry.as_str() { skills.push(s.to_lowercase()); }
                    else if let Some(obj) = entry.as_object() {
                        for (key, val) in obj { if val.as_bool().unwrap_or(false) { skills.push(key.to_lowercase()); } }
                    }
                }
            }
        }
        // Notes-based skills
        if let Some(ref notes) = c.notes {
            if let Some(start) = notes.find("[SKILLS:") {
                let after = &notes[start + 8..];
                if let Some(end) = after.find(']') {
                    for s in after[..end].split(',') {
                        let trimmed = s.trim().to_lowercase();
                        if !trimmed.is_empty() && !skills.contains(&trimmed) { skills.push(trimmed); }
                    }
                }
            }
        }
        self.char_chosen_skills = skills;

        // Expertise
        self.char_expertise_skills = self.char_feats.iter()
            .filter(|cf| self.all_feats.iter().find(|f| f.id == cf.feat_id).map(|f| f.name.to_lowercase().contains("expertise")).unwrap_or(false))
            .filter_map(|cf| cf.chosen_ability.as_ref())
            .flat_map(|s| s.split(',').map(|p| p.trim().to_lowercase().to_string()))
            .filter(|s| !s.is_empty())
            .collect();

        // Subclass
        self.char_subclass_name = if let Some(detail) = &self.class_detail {
            let feat_names: Vec<String> = self.char_feats.iter()
                .filter(|cf| cf.source_type.to_lowercase().contains("subclass"))
                .filter_map(|cf| self.all_feats.iter().find(|f| f.id == cf.feat_id).map(|f| f.name.clone()))
                .collect();

            detail.subclasses.iter().find(|swf| {
                swf.features.iter().any(|sf| {
                    feat_names.iter().any(|fn_| fn_.to_lowercase().contains(&sf.name.to_lowercase()) || sf.name.to_lowercase().contains(&fn_.to_lowercase()))
                })
            }).map(|swf| swf.subclass.name.clone()).unwrap_or_default()
        } else {
            String::new()
        };

        // Features & Traits
        let char_level = crate::utils::level_from_xp(c.experience_pts);
        self.char_class_features = self.class_detail.as_ref()
            .map(|d| d.features.iter().filter(|f| f.level <= char_level && !f.is_subclass_gate).cloned().collect())
            .unwrap_or_default();

        self.char_race_traits = if let Some(race) = self.races.iter().find(|r| Some(r.id) == c.race_id) {
            let mut traits = Vec::new();
            if let Some(entries) = &race.entries {
                for entry in entries {
                    if let Some(obj) = entry.as_object() {
                        let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let desc = obj.get("entries").and_then(|v| v.as_array()).map(|arr| {
                            arr.iter().filter_map(|e| e.as_str().map(str::to_string)).collect::<Vec<_>>().join("\n")
                        }).unwrap_or_default();
                        if !name.is_empty() { traits.push((name, desc)); }
                    } else if let Some(s) = entry.as_str() { traits.push((s.to_string(), String::new())); }
                }
            }
            traits
        } else { Vec::new() };

        // Resources
        self.spell_slots_used = [0; 9];
        for slot in cache.spell_slots {
            if slot.slot_level >= 1 && slot.slot_level <= 9 {
                self.spell_slots_used[(slot.slot_level - 1) as usize] = slot.expended as u8;
            }
        }

        self.hit_dice_used = [0; 4];
        for hd in cache.hit_dice {
            match hd.die_size {
                6 => self.hit_dice_used[0] = hd.expended as u8,
                8 => self.hit_dice_used[1] = hd.expended as u8,
                10 => self.hit_dice_used[2] = hd.expended as u8,
                12 => self.hit_dice_used[3] = hd.expended as u8,
                _ => {}
            }
        }

        self.death_saves_success = c.death_saves_successes as u8;
        self.death_saves_fail = c.death_saves_failures as u8;

        // UI State
        self.char_classes = vec![crate::models::character::CharacterClass {
            id: 0,
            character_id: c.id,
            class_id: c.class_id.unwrap_or(0),
            level: char_level,
            is_primary: true,
            subclass_id: None,
        }];
        self.screen = Screen::CharacterSheet;
        self.sheet_tab = SheetTab::CoreStats;
        self.sheet_tab_index = 0;
        self.sidebar_focused = true;
        self.content_scroll = 0;
    }

    pub fn save_notes(&mut self) {
        let (character_id, active_class_id, final_notes) = {
            let c = match &self.active_character {
                Some(c) => c,
                None => return,
            };
            
            let existing_notes = c.notes.as_deref().unwrap_or("");
            let skills_tag = if let Some(start) = existing_notes.find("[SKILLS:") {
                let end = existing_notes[start..].find(']').map(|e| start + e + 1).unwrap_or(existing_notes.len());
                Some(existing_notes[start..end].to_string())
            } else {
                None
            };
            
            let notes = if let Some(tag) = skills_tag {
                if self.notes_buffer.trim().is_empty() { tag } else { format!("{}\n{}", self.notes_buffer, tag) }
            } else {
                self.notes_buffer.clone()
            };
            
            (c.id, self.active_class_id, notes)
        };

        // Update local state immediately for responsiveness
        if let Some(active) = self.active_character.as_mut() {
            active.notes = Some(final_notes.clone());
        }

        let character_ref = self.active_character.as_ref().unwrap();
        let update = crate::models::character::UpdateCharacterRequest {
            notes: Some(final_notes),
            ..crate::models::character::UpdateCharacterRequest::from_character(
                character_ref,
                active_class_id,
            )
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.update_character(character_id, &update)) {
            Ok(updated) => {
                self.active_character = Some(updated.clone());
                // Cache updated version
                let filename = format!("char_{}.json", character_id);
                if let Some(mut cache) = self.storage.load_cache::<FullCharacterCache>(&filename) {
                    cache.character = updated;
                    self.storage.save_cache(&filename, &cache);
                }
                self.status_msg = "Notes saved.".into();
            }
            Err(e) => {
                self.status_msg = format!("Offline: Saved notes locally. (Error: {e})");
                // In offline mode, we still save to disk
                let filename = format!("char_{}.json", character_id);
                if let Some(mut cache) = self.storage.load_cache::<FullCharacterCache>(&filename) {
                    if let Some(active) = &self.active_character {
                        cache.character = active.clone();
                        self.storage.save_cache(&filename, &cache);
                    }
                }
            }
        }
    }
}
