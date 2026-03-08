use ratatui::widgets::ListState;
use uuid::Uuid;

use crate::App;
use crate::models::app_state::{EditSection, Screen, SheetTab};
use crate::models::character::Character;

impl App {
    pub fn fetch_characters(&mut self) {
        let rt = self.rt.clone();
        match rt.block_on(self.client.get_characters()) {
            Ok(chars) => {
                self.characters = chars;
                if self.selected_char >= self.characters.len() {
                    self.selected_char = self.characters.len().saturating_sub(1);
                }
                self.char_list_state.select(Some(self.selected_char));
            }
            Err(e) => {
                self.status_msg = format!("Error parsing characters: {}", e);
                self.characters = vec![];
                self.selected_char = 0;
                self.char_list_state.select(Some(0));
            }
        }
    }

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

        match rt.block_on(self.client.get_character(character_id)) {
            Ok(c) => {
                let first_class_id = c
                    .class_id
                    .unwrap_or_else(|| self.classes.first().map(|cl| cl.id).unwrap_or(1));

                self.active_class_id = first_class_id;
                self.char_race_name = self
                    .races
                    .iter()
                    .find(|r| Some(r.id) == c.race_id)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| "Unknown".into());

                let active_class = self
                    .classes
                    .iter()
                    .find(|cl| cl.id == first_class_id);

                self.char_class_name = active_class
                    .map(|cl| cl.name.clone())
                    .unwrap_or_else(|| "Unknown".into());

                self.char_caster_progression = active_class
                    .and_then(|cl| cl.caster_progression.clone())
                    .unwrap_or_default();

                self.char_bg_name = c
                    .background_id
                    .and_then(|_id| {
                        self.backgrounds
                            .iter()
                            .find(|b| Some(b.id) == c.background_id)
                    })
                    .map(|b| b.name.clone())
                    .unwrap_or_else(|| "None".into());

                // Build skill proficiencies array from background.
                // skill_proficiencies is Vec<JsonValue> where each element is either:
                //   - a string: "history"
                //   - an object: {"history": true, "intimidation": true}
                let mut skills = Vec::new();
                if let Some(bg_id) = c.background_id {
                    if let Some(bg) = self.backgrounds.iter().find(|b| b.id == bg_id) {
                        if let Some(prof) = &bg.skill_proficiencies {
                            for entry in prof {
                                if let Some(s) = entry.as_str() {
                                    // plain string entry
                                    skills.push(s.to_lowercase());
                                } else if let Some(obj) = entry.as_object() {
                                    // object entry: keys are skill names, values are true
                                    for (key, val) in obj {
                                        if val.as_bool().unwrap_or(false) {
                                            skills.push(key.to_lowercase());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Also parse skill proficiencies stored in notes as [SKILLS:skill1,skill2,...]
                if let Some(ref notes) = c.notes {
                    if let Some(start) = notes.find("[SKILLS:") {
                        let after = &notes[start + 8..];
                        if let Some(end) = after.find(']') {
                            for s in after[..end].split(',') {
                                let trimmed = s.trim().to_lowercase();
                                if !trimmed.is_empty() && !skills.contains(&trimmed) {
                                    skills.push(trimmed);
                                }
                            }
                        }
                    }
                }
                self.char_chosen_skills = skills;

                // Find the class source slug for the detail fetch
                let class_name_for_detail = self.char_class_name.clone();
                let class_source_for_detail = self
                    .classes
                    .iter()
                    .find(|cl| cl.id == first_class_id)
                    .map(|cl| cl.source_slug.clone())
                    .unwrap_or_default();

                // Fire off parallel requests for nested things:
                let feats_future = async { self.client.get_feats(c.id).await.unwrap_or_default() };
                let spells_future = async {
                    self.client
                        .get_character_spells(c.id)
                        .await
                        .unwrap_or_default()
                };
                let inventory_future =
                    async { self.client.get_inventory(c.id).await.unwrap_or_default() };
                let spell_slots_future = async {
                    match self.client.get_spell_slots(c.id).await {
                        Ok(slots) => Some(slots),
                        Err(e) => {
                            use std::io::Write;
                            if let Ok(mut f) = std::fs::OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("api_debug.log")
                            {
                                let _ = writeln!(f, "[SPELL_SLOTS_ERR] {e}\n");
                            }
                            None
                        }
                    }
                };
                let hit_dice_future = async { self.client.get_hit_dice(c.id).await.ok() };
                let class_detail_future = async {
                    self.client
                        .get_class_detail(&class_name_for_detail, &class_source_for_detail)
                        .await
                        .ok()
                };
                let actions_future = async { self.client.get_character_actions(c.id).await };
                let (
                    feats_res,
                    spells_res,
                    inventory_res,
                    spell_slots_res,
                    hit_dice_res,
                    class_detail_res,
                    actions_res,
                ) = rt.block_on(async {
                    tokio::join!(
                        feats_future,
                        spells_future,
                        inventory_future,
                        spell_slots_future,
                        hit_dice_future,
                        class_detail_future,
                        actions_future
                    )
                });

                self.char_feats = feats_res;
                match actions_res {
                    Ok(actions) => self.char_actions = Some(actions),
                    Err(e) => {
                        self.char_actions = None;
                        self.status_msg = format!("Actions error: {e:?}");
                    }
                }
                self.char_spells = spells_res;
                self.char_inventory = inventory_res;
                // char_classes is session-only (no GET endpoint); reset on sheet load
                self.char_classes = Vec::new();
                self.multiclass_selected = 0;
                self.conditions = Vec::new();
                self.concentrating_on = None;

                // Derive expertise from feats named "Expertise" — chosen_ability holds
                // a comma-separated list of skill names (e.g. "Stealth,Perception")
                self.char_expertise_skills = self
                    .char_feats
                    .iter()
                    .filter(|cf| {
                        self.all_feats
                            .iter()
                            .find(|f| f.id == cf.feat_id)
                            .map(|f| f.name.to_lowercase().contains("expertise"))
                            .unwrap_or(false)
                    })
                    .filter_map(|cf| cf.chosen_ability.as_ref())
                    .flat_map(|s| s.split(',').map(|p| p.trim().to_lowercase().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect();

                // Derive subclass name from class detail — look for any subclass whose
                // features appear in char_feats (matched by source_type == "subclass_feature")
                // or fall back to checking unlock_level vs character level.
                self.char_subclass_name = if let Some(detail) = &class_detail_res {
                    let char_level = crate::utils::level_from_xp(c.experience_pts);
                    // Try matching by subclass feat names present in char_feats
                    let feat_names: Vec<String> = self
                        .char_feats
                        .iter()
                        .filter(|cf| cf.source_type.to_lowercase().contains("subclass"))
                        .filter_map(|cf| {
                            self.all_feats
                                .iter()
                                .find(|f| f.id == cf.feat_id)
                                .map(|f| f.name.clone())
                        })
                        .collect();

                    let matched = detail.subclasses.iter().find(|swf| {
                        // Check if any subclass feature name matches a character feat name
                        swf.features.iter().any(|sf| {
                            feat_names.iter().any(|fn_| {
                                fn_.to_lowercase().contains(&sf.name.to_lowercase())
                                    || sf.name.to_lowercase().contains(&fn_.to_lowercase())
                            })
                        })
                    });

                    if let Some(swf) = matched {
                        swf.subclass.name.clone()
                    } else if char_level
                        >= detail
                            .subclasses
                            .iter()
                            .map(|s| s.subclass.unlock_level)
                            .min()
                            .unwrap_or(99)
                    {
                        // Level-gated: character is high enough for a subclass but none matched
                        String::new()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                // Populate class features (up to current level)
                let char_level = crate::utils::level_from_xp(c.experience_pts);
                self.char_class_features = if let Some(detail) = &class_detail_res {
                    detail
                        .features
                        .iter()
                        .filter(|f| f.level <= char_level && !f.is_subclass_gate)
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                };

                // Populate race traits from race.entries
                self.char_race_traits =
                    if let Some(race) = self.races.iter().find(|r| Some(r.id) == c.race_id) {
                        let mut traits = Vec::new();
                        if let Some(entries) = &race.entries {
                            for entry in entries {
                                // Each entry can be a string or an object {"type":"entries","name":"...","entries":[...]}
                                if let Some(obj) = entry.as_object() {
                                    let name = obj
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string();
                                    let desc = obj
                                        .get("entries")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|e| e.as_str().map(str::to_string))
                                                .collect::<Vec<_>>()
                                                .join("\n")
                                        })
                                        .unwrap_or_default();
                                    if !name.is_empty() {
                                        traits.push((name, desc));
                                    }
                                } else if let Some(s) = entry.as_str() {
                                    traits.push((s.to_string(), String::new()));
                                }
                            }
                        }
                        traits
                    } else {
                        Vec::new()
                    };

                // Load resource states
                self.spell_slots_used = [0; 9];
                if let Some(slots) = spell_slots_res {
                    for slot in slots {
                        if slot.slot_level >= 1 && slot.slot_level <= 9 {
                            self.spell_slots_used[(slot.slot_level - 1) as usize] =
                                slot.expended as u8;
                        }
                    }
                }

                self.hit_dice_used = [0; 4];
                if let Some(hit_dice) = hit_dice_res {
                    for hd in hit_dice {
                        match hd.die_size {
                            6 => self.hit_dice_used[0] = hd.expended as u8,
                            8 => self.hit_dice_used[1] = hd.expended as u8,
                            10 => self.hit_dice_used[2] = hd.expended as u8,
                            12 => self.hit_dice_used[3] = hd.expended as u8,
                            _ => {}
                        }
                    }
                }

                self.death_saves_success = c.death_saves_successes as u8;
                self.death_saves_fail = c.death_saves_failures as u8;

                self.active_character = Some(c);
                self.screen = Screen::CharacterSheet;
                self.sheet_tab = SheetTab::CoreStats;
                self.sheet_tab_index = 0;
                self.sidebar_focused = true;
                self.content_scroll = 0;
                self.status_msg = "Character sheet loaded.".into();
            }
            Err(e) => self.status_msg = format!("Failed to load character: {e}"),
        }
    }

    pub fn save_notes(&mut self) {
        let c = match &self.active_character {
            Some(c) => c,
            None => return,
        };
        let rt = self.rt.clone();
        // Preserve internal [SKILLS:...] tag when saving user-edited notes
        let existing_notes = c.notes.as_deref().unwrap_or("");
        let skills_tag = if let Some(start) = existing_notes.find("[SKILLS:") {
            let end = existing_notes[start..].find(']').map(|e| start + e + 1).unwrap_or(existing_notes.len());
            Some(existing_notes[start..end].to_string())
        } else {
            None
        };
        let final_notes = if let Some(tag) = skills_tag {
            if self.notes_buffer.trim().is_empty() {
                tag
            } else {
                format!("{}\n{}", self.notes_buffer, tag)
            }
        } else {
            self.notes_buffer.clone()
        };
        let update = crate::models::character::UpdateCharacterRequest {
            notes: Some(final_notes),
            ..crate::models::character::UpdateCharacterRequest::from_character(
                c,
                self.active_class_id,
            )
        };
        match rt.block_on(self.client.update_character(c.id, &update)) {
            Ok(updated) => {
                self.active_character = Some(updated);
                self.status_msg = "Notes saved.".into();
            }
            Err(e) => self.status_msg = format!("Failed to save notes: {e}"),
        }
    }
}
