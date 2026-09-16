use crate::app::App;
use crate::app::LevelUpPrompt;
use crate::models::{
    UpdateCharacterRequest,
    app_state::{EditSection, MulticlassSection, Screen},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};


    pub fn handle_edit_character_key(app: &mut App, key: KeyEvent) {
        match app.edit_section {
            EditSection::Fields => handle_edit_fields_key(app, key),
            EditSection::Race => handle_edit_race_key(app, key),
            EditSection::Class => handle_edit_class_key(app, key),
            EditSection::Subclass => handle_edit_subclass_key(app, key),
            EditSection::Background => handle_edit_bg_key(app, key),
            EditSection::Multiclass => handle_edit_multiclass_key(app, key),
            EditSection::LevelUpChoice => handle_level_up_choice_key(app, key),
        }

    pub fn handle_edit_fields_key(app: &mut App, key: KeyEvent) {
        // Field indices: 0=name, 1=xp, 2=level, 3=max_hp, 4=cur_hp, 5=temp_hp,
        //                6=str, 7=dex, 8=con, 9=int, 10=wis, 11=cha, 12=inspiration
        // After field 12: Race(13), Class(14), Subclass(15), Background(16), Multiclass(17)
        const TOTAL_FIELDS: usize = 18; // 13 text fields + 4 list pickers + 1 multiclass manager

        match key.code {
            KeyCode::Esc => {
                app.screen = if app.edit_return_to_sheet {
                    Screen::CharacterSheet
                } else {
                    Screen::CharacterList
                };
            }
            KeyCode::Tab | KeyCode::Down => {
                app.edit_field_index = (app.edit_field_index + 1) % TOTAL_FIELDS;
                switch_edit_section(app);
            }
            KeyCode::BackTab | KeyCode::Up => {
                app.edit_field_index = if app.edit_field_index == 0 {
                    TOTAL_FIELDS - 1
                } else {
                    app.edit_field_index - 1
                };
                switch_edit_section(app);
            }
            KeyCode::Enter => {
                // Enter activates list pickers; on text fields moves to next field
                if app.edit_field_index >= 13 {
                    switch_edit_section(app);
                } else {
                    app.edit_field_index = (app.edit_field_index + 1) % TOTAL_FIELDS;
                    switch_edit_section(app);
                }
            }
            KeyCode::F(2) | KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                save_edit_character(app);
            }
            KeyCode::Backspace => {
                if app.edit_field_index < 13 {
                    let idx = app.edit_field_index;
                    app.edit_buffers[idx].pop();
                }
            }
            KeyCode::Char(c) => {
                if app.edit_field_index < 13 {
                    let idx = app.edit_field_index;
                    app.edit_buffers[idx].push(c);
                }
            }
            _ => {}
        }
    }

    pub fn switch_edit_section(app: &mut App,) {
        app.edit_section = match app.edit_field_index {
            13 => EditSection::Race,
            14 => EditSection::Class,
            15 => {
                ensure_subclasses_loaded(app);
                EditSection::Subclass
            }
            16 => EditSection::Background,
            17 => EditSection::Multiclass,
            _ => EditSection::Fields,
        };
    }

    fn ensure_subclasses_loaded(app: &mut App,) {
        let class_id = app.classes.get(app.edit_class_index).map(|c| c.id).unwrap_or(0);
        let needs_fetch = app.class_detail.as_ref().map(|d| d.class.id != class_id).unwrap_or(true);
        
        if needs_fetch && class_id != 0 {
            let (name, source) = app.classes.get(app.edit_class_index)
                .map(|c| (c.name.clone(), c.source_slug.clone()))
                .unwrap();
            let rt = app.rt.clone();
            if let Ok(detail) = rt.block_on(app.client.get_class_detail(&name, &source)) {
                // Update edit_subclass_index based on current character's subclass
                let current_subclass_id = app.char_classes.first().and_then(|cc| cc.subclass_id);
                app.edit_subclass_index = detail.subclasses.iter()
                    .position(|swf| Some(swf.subclass.id) == current_subclass_id)
                    .unwrap_or(0);
                let idx = app.edit_subclass_index;
                app.edit_subclass_state.select(Some(idx));
                // Only pre-select if the character actually has a subclass for this class.
                app.edit_subclass_selected = current_subclass_id.is_some();
                app.class_detail = Some(detail);
            }
        }
    }

    pub fn handle_edit_race_key(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.edit_section = EditSection::Fields;
            }
            KeyCode::Up => {
                if app.edit_race_index > 0 {
                    app.edit_race_index -= 1;
                    let idx = app.edit_race_index;
                    app.edit_race_state.select(Some(idx));
                }
            }
            KeyCode::Down => {
                if app.edit_race_index + 1 < app.races.len() {
                    app.edit_race_index += 1;
                    let idx = app.edit_race_index;
                    app.edit_race_state.select(Some(idx));
                }
            }
            KeyCode::Tab => {
                app.edit_field_index = 14;
                app.edit_section = EditSection::Class;
            }
            KeyCode::BackTab => {
                app.edit_field_index = 12;
                app.edit_section = EditSection::Fields;
            }
            _ => {}
        }
    }

    pub fn handle_edit_class_key(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.edit_section = EditSection::Fields;
            }
            KeyCode::Up => {
                if app.edit_class_index > 0 {
                    app.edit_class_index -= 1;
                    let idx = app.edit_class_index;
                    app.edit_class_state.select(Some(idx));
                    // Reset subclass when class changes
                    app.class_detail = None;
                    app.edit_subclass_index = 0;
                    app.edit_subclass_selected = false;
                    app.edit_subclass_state.select(Some(0));
                }
            }
            KeyCode::Down => {
                if app.edit_class_index + 1 < app.classes.len() {
                    app.edit_class_index += 1;
                    let idx = app.edit_class_index;
                    app.edit_class_state.select(Some(idx));
                    // Reset subclass when class changes
                    app.class_detail = None;
                    app.edit_subclass_index = 0;
                    app.edit_subclass_selected = false;
                    app.edit_subclass_state.select(Some(0));
                }
            }
            KeyCode::Tab => {
                app.edit_field_index = 15;
                switch_edit_section(app);
            }
            KeyCode::BackTab => {
                app.edit_field_index = 13;
                app.edit_section = EditSection::Race;
            }
            _ => {}
        }
    }

    pub fn handle_edit_subclass_key(app: &mut App, key: KeyEvent) {
        let count = app.class_detail.as_ref().map(|d| d.subclasses.len()).unwrap_or(0);

        match key.code {
            KeyCode::Esc => {
                app.edit_section = EditSection::Fields;
            }
            KeyCode::Up => {
                if app.edit_subclass_index > 0 {
                    app.edit_subclass_index -= 1;
                    let idx = app.edit_subclass_index;
                    app.edit_subclass_state.select(Some(idx));
                    app.edit_subclass_selected = true;
                }
            }
            KeyCode::Down => {
                if app.edit_subclass_index + 1 < count {
                    app.edit_subclass_index += 1;
                    let idx = app.edit_subclass_index;
                    app.edit_subclass_state.select(Some(idx));
                    app.edit_subclass_selected = true;
                }
            }
            KeyCode::Tab => {
                app.edit_field_index = 16;
                app.edit_section = EditSection::Background;
            }
            KeyCode::BackTab => {
                app.edit_field_index = 14;
                app.edit_section = EditSection::Class;
            }
            _ => {}
        }
    }

    pub fn handle_edit_bg_key(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.edit_section = EditSection::Fields;
            }
            KeyCode::Up => {
                if app.edit_bg_index > 0 {
                    app.edit_bg_index -= 1;
                    let idx = app.edit_bg_index;
                    app.edit_bg_state.select(Some(idx));
                }
            }
            KeyCode::Down => {
                if app.edit_bg_index + 1 < app.backgrounds.len() {
                    app.edit_bg_index += 1;
                    let idx = app.edit_bg_index;
                    app.edit_bg_state.select(Some(idx));
                }
            }
            KeyCode::Tab => {
                app.edit_field_index = 17;
                app.edit_section = EditSection::Multiclass;
            }
            KeyCode::BackTab => {
                app.edit_field_index = 15;
                app.edit_section = EditSection::Subclass;
            }
            _ => {}
        }
    }

    pub fn handle_edit_multiclass_key(app: &mut App, key: KeyEvent) {
        match app.multiclass_section {
            MulticlassSection::List => match key.code {
                KeyCode::Esc | KeyCode::BackTab => {
                    app.edit_field_index = 16;
                    app.edit_section = EditSection::Background;
                }
                KeyCode::Tab => {
                    app.edit_field_index = 0;
                    app.edit_section = EditSection::Fields;
                }
                KeyCode::Up => {
                    if app.multiclass_selected > 0 {
                        app.multiclass_selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if app.multiclass_selected + 1 < app.char_classes.len() {
                        app.multiclass_selected += 1;
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    app.multiclass_section = MulticlassSection::Add;
                    app.multiclass_add_index = 0;
                    app.multiclass_add_state.select(Some(0));
                }
                KeyCode::Char('+') | KeyCode::Char('l') => {
                    app.level_up_multiclass();
                }
                _ => {}
            },
            MulticlassSection::Add => match key.code {
                KeyCode::Esc => {
                    app.multiclass_section = MulticlassSection::List;
                }
                KeyCode::Up => {
                    if app.multiclass_add_index > 0 {
                        app.multiclass_add_index -= 1;
                        let idx = app.multiclass_add_index;
                        app.multiclass_add_state
                            .select(Some(idx));
                    }
                }
                KeyCode::Down => {
                    if app.multiclass_add_index + 1 < app.classes.len() {
                        app.multiclass_add_index += 1;
                        let idx = app.multiclass_add_index;
                        app.multiclass_add_state
                            .select(Some(idx));
                    }
                }
                KeyCode::Enter => {
                    app.add_multiclass();
                    app.multiclass_section = MulticlassSection::List;
                }
                _ => {}
            },
        }
    }

    pub fn save_edit_character(app: &mut App,) {
        let id = match app.edit_character_id {
            Some(id) => id,
            None => {
                app.status_msg = "No character to save".to_string();
                return;
            }
        };

        let parse_i32 = |s: &str| s.trim().parse::<i32>().ok();

        let class_id = app
            .classes
            .get(app.edit_class_index)
            .map(|c| c.id)
            .unwrap_or(0);

        // If Level was edited directly, update new_xp
        let parsed_xp = parse_i32(&app.edit_buffers[1]).unwrap_or(0);
        let edited_lvl = parse_i32(&app.edit_buffers[2]).unwrap_or(1);

        let old_level = app
            .active_character
            .as_ref()
            .map(|c| crate::models::rules::level_from_xp(c.experience_pts))
            .unwrap_or(edited_lvl);

        let new_xp = if edited_lvl != old_level && edited_lvl != crate::models::rules::level_from_xp(parsed_xp)
        {
            crate::models::rules::xp_from_level(edited_lvl)
        } else {
            parsed_xp
        };

        let new_level = crate::models::rules::level_from_xp(new_xp);

        let auto_max_hp = if new_level != old_level {
            let con = parse_i32(&app.edit_buffers[8]).unwrap_or(10);
            let con_mod = (con - 10).div_euclid(2);
            let hit_die = app
                .classes
                .iter()
                .find(|c| c.id == class_id)
                .map(|c| c.hit_die)
                .unwrap_or(8);
            // Add or remove (hit_die/2 + 1 + con_mod) per level delta
            let per_level = hit_die / 2 + 1 + con_mod;
            let current_max_hp = parse_i32(&app.edit_buffers[3]).unwrap_or_default();
            let level_delta = new_level - old_level;
            Some((current_max_hp + per_level * level_delta).max(1))
        } else {
            None
        };
        let computed_max_hp =
            auto_max_hp.unwrap_or_else(|| parse_i32(&app.edit_buffers[3]).unwrap_or_default());

        let name = app.edit_buffers[0].trim().to_string();

        // Determine the subclass gate level from class data (default 3 for all 5.5e classes).
        let subclass_gate = app.class_detail.as_ref()
            .and_then(|d| d.features.iter().find(|f| f.is_subclass_gate).map(|f| f.level))
            .unwrap_or(3);

        // Only include subclass_id if the user has an explicit selection AND the character
        // meets the gate level. If they're too low, force None and tell them why.
        let mut subclass_level_warning: Option<String> = None;
        let subclass_id: Option<i32> = if app.edit_subclass_selected {
            if new_level >= subclass_gate {
                app.class_detail.as_ref().and_then(|d| {
                    d.subclasses.get(app.edit_subclass_index).map(|swf| swf.subclass.id)
                })
            } else {
                // Below the gate — block the selection silently and warn on save.
                app.edit_subclass_selected = false;
                subclass_level_warning = Some(format!(
                    "Subclass not applied: {} subclass unlocks at level {} (you are level {}).",
                    app.char_class_name, subclass_gate, new_level
                ));
                None
            }
        } else {
            None
        };

        let req = UpdateCharacterRequest {
            name,
            class_id,
            subclass_id,
            experience_pts: Some(new_xp),
            max_hp: computed_max_hp,
            current_hp: parse_i32(&app.edit_buffers[4]),
            temp_hp: parse_i32(&app.edit_buffers[5]),
            strength: parse_i32(&app.edit_buffers[6]).unwrap_or_default(),
            dexterity: parse_i32(&app.edit_buffers[7]).unwrap_or_default(),
            constitution: parse_i32(&app.edit_buffers[8]).unwrap_or_default(),
            intelligence: parse_i32(&app.edit_buffers[9]).unwrap_or_default(),
            wisdom: parse_i32(&app.edit_buffers[10]).unwrap_or_default(),
            charisma: parse_i32(&app.edit_buffers[11]).unwrap_or_default(),
            inspiration: match app.edit_buffers[12].trim().to_lowercase().as_str() {
                "yes" | "y" | "true" | "1" => Some(true),
                "no" | "n" | "false" | "0" => Some(false),
                _ => None,
            },
            race_id: app.races.get(app.edit_race_index).map(|r| r.id),
            subrace_id: None,
            background_id: app.backgrounds.get(app.edit_bg_index).map(|b| b.id),
            notes: None,
            death_saves_successes: None,
            death_saves_failures: None,
            cp: None,
            sp: None,
            ep: None,
            gp: None,
            pp: None,
        };

        let rt = app.rt.clone();
        // Log update payload for debugging level-up saves
        if let Ok(payload) = serde_json::to_string_pretty(&req) {
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("draft_payload.log")
                .and_then(|mut f| {
                    use std::io::Write;
                    writeln!(f, "UPDATE /characters/{} => {}\n", id, payload)
                });
        }

        match rt.block_on(app.client.update_character(id, &req)) {
            Ok(updated) => {
                // Update active character if editing the currently viewed one
                if app.active_character.as_ref().map(|c| c.id) == Some(updated.id) {
                    app.char_race_name = updated
                        .race_id
                        .and_then(|rid| app.races.iter().find(|r| r.id == rid))
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    app.char_bg_name = updated
                        .background_id
                        .and_then(|bid| app.backgrounds.iter().find(|b| b.id == bid))
                        .map(|b| b.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    if let Some(cls) = app.classes.get(app.edit_class_index) {
                        app.char_class_name = cls.name.clone();
                        app.active_class_id = cls.id;
                    }
                    // Sync max_hp buffer if it was auto-calculated
                    if auto_max_hp.is_some() {
                        app.edit_buffers[3] = updated.max_hp.to_string();
                    }

                    // Update local subclass state immediately
                    if let Some(cc) = app.char_classes.first_mut() {
                        cc.subclass_id = subclass_id;
                    }
                    app.refresh_subclass_features();
                    app.persist_subclass_to_cache();

                    app.active_character = Some(updated);
                    app.refresh_derived_actions();

                    // Queue level-up prompts if level changed
                    if new_level != old_level {
                        app.check_level_up_prompts(old_level, new_level);
                    }
                }

                // Patch the class record when the level changed so the backend
                // keeps its class-level in sync with the character's XP-derived level.
                if new_level != old_level {
                    let patch_req = crate::models::character::PatchCharacterClassRequest {
                        level: Some(new_level),
                        subclass_id: None, // subclass already handled in the PUT above
                    };
                    if let Err(e) = rt.block_on(app.client.patch_character_class(id, class_id, &patch_req)) {
                        app.status_msg = format!("Save failed: {e}");
                        return;
                    }
                    // Sync local class level
                    if let Some(cc) = app.char_classes.first_mut() {
                        cc.level = new_level;
                    }
                }

                app.fetch_characters();

                // If there are level-up prompts, stay in EditCharacter and show the first one
                if !app.level_up_queue.is_empty() {
                    advance_level_up_prompt(app);
                    return;
                }

                app.status_msg = if let Some(warn) = subclass_level_warning {
                    format!("Saved! ⚠ {}", warn)
                } else {
                    "Saved!".to_string()
                };
                app.screen = if app.edit_return_to_sheet {
                    Screen::CharacterSheet
                } else {
                    Screen::CharacterList
                };
            }
            Err(e) => {
                app.status_msg = format!("Save failed: {e}");
            }
        }
    }

    /// Pop the next queued level-up prompt and show it in the edit screen.
    pub fn advance_level_up_prompt(app: &mut App,) {
        if app.level_up_queue.is_empty() {
            // All done — leave edit screen now
            app.level_up_current = None;
            app.edit_section = EditSection::Fields;
            app.status_msg = "Saved! All level-up choices done.".to_string();
            app.screen = if app.edit_return_to_sheet {
                Screen::CharacterSheet
            } else {
                Screen::CharacterList
            };
            return;
        }

        let prompt = app.level_up_queue.remove(0);

        // For SubclassChoice: ensure class_detail is loaded
        if let LevelUpPrompt::SubclassChoice {
            ref class_id,
            ref class_name,
        } = prompt
        {
            let cid = *class_id;
            let cname = class_name.clone();
            let needs_fetch = app
                .class_detail
                .as_ref()
                .map(|d| d.class.id != cid)
                .unwrap_or(true);
            if needs_fetch {
                let source_slug = app
                    .classes
                    .iter()
                    .find(|c| c.id == cid)
                    .map(|c| c.source_slug.clone())
                    .unwrap_or_default();
                let rt = app.rt.clone();
                match rt.block_on(app.client.get_class_detail(&cname, &source_slug)) {
                    Ok(detail) => app.class_detail = Some(detail),
                    Err(e) => {
                        app.status_msg = format!("Could not load subclasses: {e}");
                        app.level_up_queue.clear();
                        app.level_up_current = None;
                        app.screen = if app.edit_return_to_sheet {
                            Screen::CharacterSheet
                        } else {
                            Screen::CharacterList
                        };
                        return;
                    }
                }
            }
            app.subclass_picker_class_id = cid;
            app.status_msg = format!("Level up! Choose a subclass for {}.", cname);
        } else if let LevelUpPrompt::AsiOrFeat { ref class_name } = prompt {
            app.status_msg = format!("Level up! {} gets an ASI or Feat.", class_name);
            // Reset ASI state
            app.asi_choice_index = 0;
            app.asi_mode = crate::app::AsiMode::PlusOneTwo;
            app.asi_ability_a = 0;
            app.asi_ability_b = 1;
            app.asi_ability_c = 2;
            app.picker_search.clear();
            app.picker_selected = 0;
        }

        app.level_up_current = Some(prompt);
        app.edit_section = EditSection::LevelUpChoice;
    }

    /// Key handler for the level-up overlay shown inside the edit screen.
    pub fn handle_level_up_choice_key(app: &mut App, key: KeyEvent) {
        use crate::models::rules::ABILITY_NAMES;

        let prompt = match app.level_up_current.clone() {
            Some(p) => p,
            None => {
                advance_level_up_prompt(app);
                return;
            }
        };

        match &prompt {
            LevelUpPrompt::SubclassChoice { class_id, .. } => {
                let cid = *class_id;
                let count = app
                    .class_detail
                    .as_ref()
                    .map(|d| d.subclasses.len())
                    .unwrap_or(0);
                match key.code {
                    KeyCode::Up => {
                        if app.picker_selected > 0 {
                            app.picker_selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.picker_selected + 1 < count {
                            app.picker_selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Confirm subclass
                        let detail = match app.class_detail.clone() {
                            Some(d) => d,
                            None => return,
                        };
                        if let Some(swf) = detail.subclasses.get(app.picker_selected) {
                            let subclass_id = swf.subclass.id;
                            let subclass_name = swf.subclass.name.clone();
                            if cid == app.active_class_id {
                                // Use actual character level from XP, not stale char_classes level
                                let char_level = app.active_character
                                    .as_ref()
                                    .map(|c| crate::models::rules::level_from_xp(c.experience_pts));
                                app.patch_primary_class(char_level, Some(subclass_id));
                                app.char_subclass_name = subclass_name.clone();
                                // Refresh subclass features so they appear immediately
                                app.refresh_subclass_features();
                            } else {
                                app.set_multiclass_subclass(cid, subclass_id);
                            }
                            app.persist_subclass_to_cache();
                            app.status_msg = format!("Subclass set: {}", subclass_name);
                        }
                        app.picker_selected = 0;
                        advance_level_up_prompt(app);
                    }
                    KeyCode::Esc => {
                        // Skip this subclass choice
                        app.picker_selected = 0;
                        advance_level_up_prompt(app);
                    }
                    _ => {}
                }
            }
            LevelUpPrompt::AsiOrFeat { .. } => {
                // Sub-mode: are we picking feat or ASI?
                let feat_mode = app.asi_feat_mode;
                if feat_mode {
                    // Feat picker mode
                    match key.code {
                        KeyCode::Up => {
                            if app.picker_selected > 0 {
                                app.picker_selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let max = app.filtered_feats(app.active_character.clone(), None).len();
                            if app.picker_selected + 1 < max {
                                app.picker_selected += 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            app.picker_search.push(c);
                            app.picker_selected = 0;
                        }
                        KeyCode::Backspace => {
                            app.picker_search.pop();
                        }
                        KeyCode::Enter => {
                            app.confirm_feat_asi_choice();
                            app.asi_feat_mode = false;
                            advance_level_up_prompt(app);
                        }
                        KeyCode::Esc => {
                            // Back to ASI view
                            app.asi_feat_mode = false;
                            app.picker_search.clear();
                            app.picker_selected = 0;
                        }
                        _ => {}
                    }
                } else {
                    // ASI selection mode (+1/+1 only)
                    match key.code {
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            // Switch to feat picker
                            app.picker_search.clear();
                            app.picker_selected = 0;
                            // Fetch available feats if not loaded
                            if app.all_feats.is_empty() {
                                if let Some(char_id) = app.active_character.as_ref().map(|c| c.id) {
                                    let rt = app.rt.clone();
                                    if let Ok(feats) =
                                        rt.block_on(app.client.get_available_feats(char_id))
                                    {
                                        app.all_feats = feats;
                                    }
                                }
                            }
                            if app.all_feats.is_empty() {
                                app.status_msg =
                                    "No feats available or failed to load feats.".to_string();
                            } else {
                                app.asi_feat_mode = true;
                            }
                        }
                        KeyCode::Up => {
                            let max_idx = ABILITY_NAMES.len();
                            match app.asi_choice_index {
                                0 => {
                                    if app.asi_ability_a > 0 {
                                        app.asi_ability_a -= 1;
                                    } else {
                                        app.asi_ability_a = max_idx - 1;
                                    }
                                }
                                1 => {
                                    if app.asi_ability_b > 0 {
                                        app.asi_ability_b -= 1;
                                    } else {
                                        app.asi_ability_b = max_idx - 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Down => {
                            let max_idx = ABILITY_NAMES.len();
                            match app.asi_choice_index {
                                0 => app.asi_ability_a = (app.asi_ability_a + 1) % max_idx,
                                1 => app.asi_ability_b = (app.asi_ability_b + 1) % max_idx,
                                _ => {}
                            }
                        }
                        KeyCode::Tab | KeyCode::Right => {
                            app.asi_choice_index = (app.asi_choice_index + 1) % 2;
                        }
                        KeyCode::BackTab | KeyCode::Left => {
                            if app.asi_choice_index > 0 {
                                app.asi_choice_index -= 1;
                            } else {
                                app.asi_choice_index = 1;
                            }
                        }
                        KeyCode::Enter => {
                            app.confirm_asi_choice();
                            advance_level_up_prompt(app);
                        }
                        KeyCode::Esc => {
                            // Skip this ASI choice
                            advance_level_up_prompt(app);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

}
