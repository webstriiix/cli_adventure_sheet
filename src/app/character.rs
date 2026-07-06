use ratatui::widgets::ListState;
use uuid::Uuid;

use crate::App;
use crate::models::app_state::{EditSection, Screen, SheetTab};
use crate::models::character::{Character, CharacterClass, CharacterClassResponse};
use crate::utils::storage::FullCharacterCache;

impl App {
    pub fn open_edit_character(&mut self, character: &Character, return_to_sheet: bool) {
        self.edit_character_id = Some(character.id);
        self.edit_return_to_sheet = return_to_sheet;
        self.edit_section = EditSection::Fields;
        self.edit_field_index = 0;

        self.edit_buffers[0] = character.name.clone();
        self.edit_buffers[1] = character.experience_pts.to_string();
        self.edit_buffers[2] = crate::models::rules::level_from_xp(character.experience_pts).to_string();
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
        let idx = self.edit_race_index;
        self.edit_race_state.select(Some(idx));

        self.edit_bg_index = if let Some(bg_id) = character.background_id {
            self.backgrounds
                .iter()
                .position(|b| b.id == bg_id)
                .unwrap_or(0)
        } else {
            0
        };
        self.edit_bg_state = ListState::default();
        let idx = self.edit_bg_index;
        self.edit_bg_state.select(Some(idx));

        // Authoritative fetch of classes for this character to ensure subclass info is accurate
        let rt = self.rt.clone();
        if let Ok(classes) = rt.block_on(self.client.get_character_classes(character.id)) {
            self.char_classes = classes
                .iter()
                .map(|ccr| CharacterClass {
                    id: 0,
                    character_id: character.id,
                    class_id: ccr.class_id,
                    level: ccr.level,
                    is_primary: ccr.is_primary,
                    subclass_id: ccr.subclass_id,
                })
                .collect();

            // Sync active_class_id and try to pre-load class detail if there's a subclass
            if let Some(primary) = classes.iter().find(|cc| cc.is_primary) {
                self.active_class_id = primary.class_id;
                let (name, source) = self
                    .classes
                    .iter()
                    .find(|cl| cl.id == primary.class_id)
                    .map(|cl| (cl.name.clone(), cl.source_slug.clone()))
                    .unwrap_or_else(|| ("".to_string(), "".to_string()));

                if !name.is_empty() {
                    if let Ok(detail) = rt.block_on(self.client.get_class_detail(&name, &source)) {
                        self.class_detail = Some(detail);
                    }
                }
            }
        }

        // Initialize class picker index
        let current_class_id = self.active_class_id;
        self.edit_class_index = self
            .classes
            .iter()
            .position(|c| c.id == current_class_id)
            .unwrap_or(0);
        self.edit_class_state = ListState::default();
        let idx = self.edit_class_index;
        self.edit_class_state.select(Some(idx));

        // Initialize subclass picker index from fetched data
        let current_subclass_id = self.char_classes.first().and_then(|cc| cc.subclass_id);
        self.edit_subclass_index = self
            .class_detail
            .as_ref()
            .and_then(|d| {
                d.subclasses
                    .iter()
                    .position(|swf| Some(swf.subclass.id) == current_subclass_id)
            })
            .unwrap_or(0);
        self.edit_subclass_state = ListState::default();
        let idx = self.edit_subclass_index;
        self.edit_subclass_state.select(Some(idx));

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
                let (class_name, class_source) = self
                    .classes
                    .iter()
                    .find(|cl| cl.id == first_class_id)
                    .map(|cl| (cl.name.clone(), cl.source_slug.clone()))
                    .unwrap_or_else(|| ("Unknown".into(), "PHB".into()));

                // Fetch character classes from API (includes authoritative subclass data)
                let char_classes_result = rt.block_on(self.client.get_character_classes(c.id));

                let level = crate::models::rules::level_from_xp(c.experience_pts);

                // Parallel fetch all related data
                let (feats, spells, inventory, slots, hit_dice, detail, actions, resources, profs) =
                    rt.block_on(async {
                        tokio::join!(
                            self.client.get_feats(c.id),
                            self.client.get_character_spells(c.id),
                            self.client.get_inventory(c.id),
                            self.client.get_spell_slots(c.id),
                            self.client.get_hit_dice(c.id),
                            self.client.get_class_detail(&class_name, &class_source),
                            self.client.get_character_actions(c.id),
                            self.client.get_class_resources(&class_name, &class_source, level),
                            self.client.get_proficiencies(c.id)
                        )
                    });

                // Build char_classes from API response (authoritative)
                let char_classes: Vec<CharacterClass> = char_classes_result
                    .as_ref()
                    .map(|ccs| ccs
                    .iter()
                    .map(|ccr| CharacterClass {
                        id: 0,
                        character_id: c.id,
                        class_id: ccr.class_id,
                        level: ccr.level,
                        is_primary: ccr.is_primary,
                        subclass_id: ccr.subclass_id,
                    })
                    .collect())
                    .unwrap_or_default();

                // Extract subclass_name from API response (primary class)
                let api_subclass_name: String = char_classes_result
                    .as_ref()
                    .ok()
                    .and_then(|ccs: &Vec<CharacterClassResponse>| ccs.iter().find(|cc| cc.is_primary))
                    .and_then(|cc| cc.subclass_name.clone())
                    .unwrap_or_default();

                // Preserve existing cache's spell_sources, use API data for subclass
                let filename = format!("char_{}.json", character_id);
                let existing_cache = self.storage.load_cache::<FullCharacterCache>(&filename);

                let cache = FullCharacterCache {
                    character: c.clone(),
                    feats: feats.unwrap_or_default(),
                    spells: spells.unwrap_or_default(),
                    inventory: inventory.unwrap_or_default(),
                    spell_slots: slots.unwrap_or_default(),
                    hit_dice: hit_dice.unwrap_or_default(),
                    proficiencies: profs.unwrap_or_default(),
                    class_detail: detail.ok(),
                    actions: actions.ok(),
                    resources: resources.ok(),
                    spell_sources: existing_cache
                        .as_ref()
                        .map(|ec| ec.spell_sources.clone())
                        .unwrap_or_default(),
                    subclass_name: api_subclass_name,
                    char_classes: if !char_classes.is_empty() {
                        char_classes
                    } else {
                        vec![CharacterClass {
                            id: 0,
                            character_id: c.id,
                            class_id: c.class_id.unwrap_or(0),
                            level: level,
                            is_primary: true,
                            subclass_id: None,
                        }]
                    },
                };

                // Save to local cache for offline use
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
        self.char_proficiencies = cache.proficiencies;
        self.char_actions = cache.actions;
        self.char_resources = cache.resources;
        self.class_detail = cache.class_detail;

        // Character level needed early for class initialization
        let char_level = crate::models::rules::level_from_xp(c.experience_pts);

        // UI State — restore char_classes from cache if available
        self.char_classes = if !cache.char_classes.is_empty() {
            cache.char_classes
        } else {
            vec![crate::models::character::CharacterClass {
                id: 0,
                character_id: c.id,
                class_id: c.class_id.unwrap_or(0),
                level: char_level,
                is_primary: true,
                subclass_id: None,
            }]
        };

        // Names
        self.char_race_name = self
            .races
            .iter()
            .find(|r| Some(r.id) == c.race_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| "Unknown".into());

        let active_class = self.classes.iter().find(|cl| cl.id == first_class_id);
        self.char_class_name = active_class
            .map(|cl| cl.name.clone())
            .unwrap_or_else(|| "Unknown".into());
        self.char_caster_progression = active_class
            .and_then(|cl| cl.caster_progression.clone())
            .unwrap_or_default();

        self.char_bg_name = self
            .backgrounds
            .iter()
            .find(|b| Some(b.id) == c.background_id)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "None".into());

        // Skills
        let mut skills = Vec::new();
        if let Some(bg) = self
            .backgrounds
            .iter()
            .find(|b| Some(b.id) == c.background_id)
        {
            if let Some(prof) = &bg.skill_proficiencies {
                for entry in prof {
                    if let Some(s) = entry.as_str() {
                        skills.push(s.to_lowercase());
                    } else if let Some(obj) = entry.as_object() {
                        for (key, val) in obj {
                            if val.as_bool().unwrap_or(false) {
                                skills.push(key.to_lowercase());
                            }
                        }
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
                        if !trimmed.is_empty() && !skills.contains(&trimmed) {
                            skills.push(trimmed);
                        }
                    }
                }
            }
        }
        self.char_chosen_skills = skills;

        // Expertise
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

        // Subclass — get subclass_id from char_classes, then look up by ID
        let subclass_id = self.char_classes.first().and_then(|cc| cc.subclass_id);
        self.char_subclass_name = if let Some(sid) = subclass_id {
            self.class_detail
                .as_ref()
                .and_then(|d| d.subclasses.iter().find(|swf| swf.subclass.id == sid))
                .map(|swf| swf.subclass.name.clone())
                .unwrap_or_default()
        } else {
            cache.subclass_name.clone()
        };

        // Features & Traits
        self.char_class_features = self
            .class_detail
            .as_ref()
            .map(|d| {
                d.features
                    .iter()
                    .filter(|f| f.level <= char_level && !f.is_subclass_gate)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        // Subclass features — match by subclass_id, not name
        self.char_subclass_features = self
            .class_detail
            .as_ref()
            .and_then(|d| {
                subclass_id
                    .and_then(|sid| d.subclasses.iter().find(|swf| swf.subclass.id == sid))
                    .map(|swf| {
                        swf.features
                            .iter()
                            .filter(|f| f.level <= char_level)
                            .cloned()
                            .collect()
                    })
            })
            .unwrap_or_default();

        self.char_race_traits =
            if let Some(race) = self.races.iter().find(|r| Some(r.id) == c.race_id) {
                let mut traits = Vec::new();
                if let Some(entries) = &race.entries {
                    for entry in entries {
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

        // Spell slots from API class definition
        self.char_spell_slots = self
            .class_detail
            .as_ref()
            .and_then(|d| d.class.spell_slots.as_ref())
            .map(|rows| {
                rows.iter()
                    .map(|row| row.iter().map(|&v| v as u8).collect())
                    .collect()
            })
            .unwrap_or_default();

        // Spell sources
        self.spell_sources = cache.spell_sources.into_iter().collect();

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

        // Mark spells dirty so sync runs on next render
        self.spells_dirty = true;

        // Merge derived actions into the stored char_actions
        let derived = self.derive_actions();
        if let Some(ref mut actions) = self.char_actions {
            for la in derived {
                // If it's a limited use action, merge into limited_use
                if la.max_uses.is_some() {
                    if !actions.limited_use.iter().any(|a| a.name == la.name) {
                        actions.limited_use.push(la.clone());
                    } else {
                        // Update max_uses in case level changed
                        if let Some(existing) = actions.limited_use.iter_mut().find(|a| a.name == la.name) {
                            existing.max_uses = la.max_uses;
                        }
                    }
                }
                // Also merge into 'all' if not present
                if !actions.all.iter().any(|a| a.name == la.name) {
                    actions.all.push(la.clone());
                } else {
                    if let Some(existing) = actions.all.iter_mut().find(|a| a.name == la.name) {
                        existing.max_uses = la.max_uses;
                    }
                }
                // Merge into 'attack' if it has hit_bonus/damage and not present
                if (la.hit_bonus.is_some() || la.damage.is_some())
                    && !actions.attack.iter().any(|a| a.name == la.name)
                {
                    actions.attack.push(la);
                }
            }
        } else {
            // If no char_actions from API, create it from derived
            let mut actions = crate::models::actions::CharacterActionsResponse {
                all: Vec::new(),
                attack: Vec::new(),
                action: Vec::new(),
                bonus_action: Vec::new(),
                reaction: Vec::new(),
                other: Vec::new(),
                limited_use: Vec::new(),
            };
            for la in derived {
                actions.all.push(la.clone());
                if la.max_uses.is_some() {
                    actions.limited_use.push(la.clone());
                }
                if la.hit_bonus.is_some() || la.damage.is_some() {
                    actions.attack.push(la);
                }
            }
            self.char_actions = Some(actions);
        }

        // Check if the character is missing a subclass despite meeting the level requirement
        let subclass_gate = self
            .class_detail
            .as_ref()
            .and_then(|d| d.features.iter().find(|f| f.is_subclass_gate).map(|f| f.level))
            .unwrap_or(3);

        let has_subclass = self.char_classes.first().and_then(|cc| cc.subclass_id).is_some();

        if char_level >= subclass_gate && !has_subclass {
            let class_name = self.char_class_name.clone();
            self.level_up_queue.push(crate::app::LevelUpPrompt::SubclassChoice {
                class_id: first_class_id,
                class_name,
            });
        }

        self.screen = Screen::CharacterSheet;
        self.sheet_tab = SheetTab::CoreStats;
        self.sheet_tab_index = 0;
        self.sidebar_focused = true;
        self.content_scroll = 0;
    }

    /// Save the current in-memory char_subclass_name, spell_sources, and char_classes to the local cache.
    pub fn persist_subclass_to_cache(&mut self) {
        if let Some(character) = &self.active_character {
            let filename = format!("char_{}.json", character.id);
            if let Some(mut cache) = self.storage.load_cache::<FullCharacterCache>(&filename) {
                cache.subclass_name = self.char_subclass_name.clone();
                cache.spell_sources = self
                    .spell_sources
                    .iter()
                    .map(|(id, src)| (*id, src.clone()))
                    .collect();
                cache.char_classes = self.char_classes.clone();
                self.storage.save_cache(&filename, &cache);
            }
        }
    }

    /// Re-calculate char_subclass_features from current state (used after subclass selection).
    pub fn refresh_subclass_features(&mut self) {
        let subclass_id = self.char_classes.first().and_then(|cc| cc.subclass_id);

        self.char_subclass_name = if let Some(sid) = subclass_id {
            self.class_detail
                .as_ref()
                .and_then(|d| d.subclasses.iter().find(|swf| swf.subclass.id == sid))
                .map(|swf| swf.subclass.name.clone())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let char_level = self.active_character
            .as_ref()
            .map(|c| crate::models::rules::level_from_xp(c.experience_pts))
            .unwrap_or(1);

        self.char_subclass_features = self
            .class_detail
            .as_ref()
            .and_then(|d| {
                subclass_id
                    .and_then(|sid| d.subclasses.iter().find(|swf| swf.subclass.id == sid))
                    .map(|swf| {
                        swf.features
                            .iter()
                            .filter(|f| f.level <= char_level)
                            .cloned()
                            .collect()
                    })
            })
            .unwrap_or_default();

        // Mark spells dirty since subclass changed (may affect always-prepared spells)
        self.spells_dirty = true;
    }

    pub fn save_notes(&mut self) {
        let (character_id, active_class_id, final_notes) = {
            let c = match &self.active_character {
                Some(c) => c,
                None => return,
            };

            let existing_notes = c.notes.as_deref().unwrap_or("");
            let skills_tag = if let Some(start) = existing_notes.find("[SKILLS:") {
                let end = existing_notes[start..]
                    .find(']')
                    .map(|e| start + e + 1)
                    .unwrap_or(existing_notes.len());
                Some(existing_notes[start..end].to_string())
            } else {
                None
            };

            let notes = if let Some(tag) = skills_tag {
                if self.notes_buffer.trim().is_empty() {
                    tag
                } else {
                    format!("{}\n{}", self.notes_buffer, tag)
                }
            } else {
                self.notes_buffer.clone()
            };

            (c.id, self.active_class_id, notes)
        };

        // Update local state immediately for responsiveness
        if let Some(active) = self.active_character.as_mut() {
            active.notes = Some(final_notes.clone());
        }

        let Some(character_ref) = self.active_character.as_ref() else {
            self.status_msg = "No active character to save notes".into();
            return;
        };
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
