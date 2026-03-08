use crate::app::App;
use crate::models::{CreateCharacterRequest, UpdateCharacterRequest, app_state::BuilderState};
use crossterm::event::KeyEvent;

pub fn handle_builder_key(app: &mut App, key: KeyEvent) {
    crate::ui::builder::handle_key(app, key);
}

pub fn submit_character_from_builder(app: &mut App) {
    let b = &app.builder;

    let class_id = match b.class_id {
        Some(id) => id,
        None => {
            app.status_msg = "No class selected!".to_string();
            return;
        }
    };

    // Calculate max HP: hit die + CON modifier
    let hit_die = app
        .classes
        .iter()
        .find(|c| c.id == class_id)
        .map(|c| c.hit_die)
        .unwrap_or(8);
    let con_mod = (b.abilities[2] + b.bg_ability_bonuses[2] - 10) / 2;
    let max_hp = (hit_die + con_mod).max(1);

    // Pack lore into notes since the API has no dedicated fields for it
    let notes = {
        let mut parts = Vec::new();
        if !b.age.is_empty() {
            parts.push(format!("Age: {}", b.age));
        }
        if !b.height.is_empty() {
            parts.push(format!("Height: {}", b.height));
        }
        if !b.weight.is_empty() {
            parts.push(format!("Weight: {}", b.weight));
        }
        if !b.appearance.is_empty() {
            parts.push(format!("Appearance: {}", b.appearance));
        }
        if !b.alignment.is_empty() {
            parts.push(format!("Alignment: {}", b.alignment));
        }
        if !b.trait_text.is_empty() {
            parts.push(format!("Personality: {}", b.trait_text));
        }
        if !b.ideal.is_empty() {
            parts.push(format!("Ideal: {}", b.ideal));
        }
        if !b.bond.is_empty() {
            parts.push(format!("Bond: {}", b.bond));
        }
        if !b.flaw.is_empty() {
            parts.push(format!("Flaw: {}", b.flaw));
        }
        // Collect all chosen skill proficiencies (class + race + feat)
        let mut all_skills: Vec<String> = Vec::new();
        all_skills.extend(b.skill_choices.iter().cloned());
        if let Some(ref race_skill) = b.race_skill_choice {
            all_skills.push(race_skill.clone());
        }
        all_skills.extend(b.feat_skill_choices.iter().cloned());
        if !all_skills.is_empty() {
            parts.push(format!("[SKILLS:{}]", all_skills.join(",")));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    };

    let req = CreateCharacterRequest {
        name: b.name.trim().to_string(),
        class_id,
        race_id: b.race_id,
        subrace_id: None,
        background_id: b.bg_id,
        strength: b.abilities[0] + b.bg_ability_bonuses[0],
        dexterity: b.abilities[1] + b.bg_ability_bonuses[1],
        constitution: b.abilities[2] + b.bg_ability_bonuses[2],
        intelligence: b.abilities[3] + b.bg_ability_bonuses[3],
        wisdom: b.abilities[4] + b.bg_ability_bonuses[4],
        charisma: b.abilities[5] + b.bg_ability_bonuses[5],
        max_hp,
        bonus_feat_id: b.bonus_feat_id,
        background_feat_id: b.background_feat_id,
    };

    let rt = app.rt.clone();
    match rt.block_on(app.client.create_character(&req)) {
        Ok(character) => {
            let id = character.id;
            app.active_class_id = class_id;
            // If lore fields collected, do a follow-up update for notes
            if let Some(notes_text) = notes {
                let update = UpdateCharacterRequest {
                    notes: Some(notes_text),
                    ..UpdateCharacterRequest::from_character(&character, class_id)
                };
                let _ = rt.block_on(app.client.update_character(id, &update));
            }
            // Add starting equipment (option A = standard package)
            if app.builder.equipment_option == Some(0) {
                let mut items_to_add: Vec<(String, i32)> = Vec::new();

                // From class
                if let Some(class) = app.classes.iter().find(|c| c.id == class_id) {
                    let mut class_items =
                        App::parse_starting_equipment_items(&class.starting_equipment);
                    items_to_add.append(&mut class_items);
                }

                // From background
                if let Some(bg_id) = app.builder.bg_id {
                    if let Some(bg) = app.backgrounds.iter().find(|b| b.id == bg_id) {
                        if let Some(eq) = &bg.starting_equipment {
                            // Background format is an array of choice objects
                            let wrapped = serde_json::json!({ "defaultData": eq });
                            let mut bg_items = App::parse_starting_equipment_items(&wrapped);
                            items_to_add.append(&mut bg_items);
                        }
                    }
                }

                if !items_to_add.is_empty() {
                    app.add_starting_items(&rt, id, &items_to_add);
                }
            }

            app.builder = BuilderState::default();
            app.fetch_characters();
            app.load_character_sheet(id);
        }
        Err(e) => {
            app.status_msg = format!("Failed to create character: {e}");
        }
    }
}
