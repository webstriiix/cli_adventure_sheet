use crate::App;
use crate::models::app_state::PickerMode;
use crate::models::character::{AsiChoiceRequest, Character};
use crate::models::compendium::Feat;

impl App {
    pub fn expend_selected_feat(&mut self, change: i32) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_feats.is_empty() {
            return;
        }

        let mut feat = self.char_feats[self.selected_list_index].clone();
        let max = feat.uses_max.unwrap_or(0);
        if max == 0 {
            self.status_msg = "This feature has no limited uses".to_string();
            return;
        }

        let rem = feat.uses_remaining.unwrap_or(0);
        let new_uses = (rem - change).clamp(0, max);
        if new_uses == rem {
            return;
        }

        feat.uses_remaining = Some(new_uses);
        let compendium_feat_id = feat.feat_id; // Feat ID (Compendium)

        // Optimistically update UI
        let idx = self.selected_list_index;
        self.char_feats[idx] = feat.clone();
        self.status_msg = if change > 0 {
            "Feature used".to_string()
        } else {
            "Feature uses recovered".to_string()
        };

        // Persist to API
        let rt = self.rt.clone();
        let _ = rt.block_on(self.client.patch_feature_uses(
            character_id,
            compendium_feat_id,
            new_uses,
        ));
    }

    pub fn remove_selected_feat(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_feats.is_empty() {
            return;
        }

        let feat = &self.char_feats[self.selected_list_index];
        let compendium_feat_id = feat.feat_id;

        let rt = self.rt.clone();
        match rt.block_on(self.client.remove_feat(character_id, compendium_feat_id)) {
            Ok(()) => {
                self.char_feats.remove(self.selected_list_index);
                if self.selected_list_index > 0 && self.selected_list_index >= self.char_feats.len()
                {
                    self.selected_list_index -= 1;
                }
                self.status_msg = "Feat removed".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Failed to remove feat: {e}");
            }
        }
    }

    pub fn confirm_asi_choice(&mut self) {
        let character = match &self.active_character {
            Some(c) => c.clone(),
            None => return,
        };

        let ability_keys = ["str", "dex", "con", "int", "wis", "cha"];
        let mut increases: std::collections::HashMap<String, i32> =
            std::collections::HashMap::new();

        // Only mode: +1/+1 to two abilities
        *increases
            .entry(ability_keys[self.asi_ability_a].to_string())
            .or_insert(0) += 1;
        *increases
            .entry(ability_keys[self.asi_ability_b].to_string())
            .or_insert(0) += 1;

        let req = AsiChoiceRequest {
            bump_str: increases.get("str").copied(),
            bump_dex: increases.get("dex").copied(),
            bump_con: increases.get("con").copied(),
            bump_int: increases.get("int").copied(),
            bump_wis: increases.get("wis").copied(),
            bump_cha: increases.get("cha").copied(),
            feat_id: None,
            source_type: None,
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.post_asi_choice(character.id, &req)) {
            Ok(updated_char) => {
                let label = format!(
                    "+1 {} and +1 {}",
                    crate::models::rules::ABILITY_NAMES[self.asi_ability_a],
                    crate::models::rules::ABILITY_NAMES[self.asi_ability_b],
                );
                self.active_character = Some(updated_char);
                self.status_msg = format!("ASI applied: {}", label);
                self.picker_mode = PickerMode::None;
            }
            Err(e) => {
                self.status_msg = format!("Failed to apply ASI: {e}");
            }
        }
    }

    /// Confirm a feat pick from the ASI/Feat choice overlay.
    pub fn confirm_feat_asi_choice(&mut self) {
        let character = match &self.active_character {
            Some(c) => c.clone(),
            None => return,
        };

        let filtered = self.filtered_feats(Some(character.clone()), None);
        if filtered.is_empty() {
            return;
        }
        let feat_id = filtered[self.picker_selected].id;

        let req = AsiChoiceRequest {
            bump_str: None,
            bump_dex: None,
            bump_con: None,
            bump_int: None,
            bump_wis: None,
            bump_cha: None,
            feat_id: Some(feat_id),
            source_type: Some(self.char_class_name.clone()),
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.post_asi_choice(character.id, &req)) {
            Ok(updated_char) => {
                let name = self.feat_name(feat_id);
                self.active_character = Some(updated_char);
                // Refresh feats list
                let char_id = character.id;
                if let Ok(feats) = rt.block_on(self.client.get_feats(char_id)) {
                    self.char_feats = feats;
                }
                self.status_msg = format!("Feat chosen: {name}");
                self.picker_mode = PickerMode::None;
            }
            Err(e) => {
                self.status_msg = format!("Failed to apply feat choice: {e}");
            }
        }
    }

    /// Returns ASI/Feat levels for the given class name (D&D 5e rules).
    pub fn asi_levels_for_class(class_name: &str) -> &'static [i32] {
        match class_name.to_lowercase().as_str() {
            "fighter" => &[4, 6, 8, 12, 14, 16, 19],
            "rogue" => &[4, 8, 10, 12, 16, 18],
            _ => &[4, 8, 12, 16, 19],
        }
    }

    /// Filter feats by search string and optionally by character prerequisites.
    pub fn filtered_feats(
        &self,
        character: Option<Character>,
        _class_id: Option<i32>,
    ) -> Vec<&Feat> {
        let search = self.picker_search.to_lowercase();
        self.all_feats
            .iter()
            .filter(|f| search.is_empty() || f.name.to_lowercase().contains(&search))
            .filter(|f| {
                let Some(ref ch) = character else {
                    return true;
                };
                Self::feat_prereqs_met(f, ch)
            })
            .take(100)
            .collect()
    }

    /// Check if character meets feat prerequisites.
    pub fn feat_prereqs_met(feat: &Feat, ch: &Character) -> bool {
        let prereqs = match &feat.prerequisite {
            Some(p) => p,
            None => return true, // no prereqs — always available
        };

        let arr = match prereqs.as_array() {
            Some(a) => a,
            None => return true,
        };

        // All prerequisite objects must be satisfied (AND logic across items in the array)
        for prereq in arr {
            // Ability score minimum: {"ability": [{"str": 13}]}
            if let Some(ability_arr) = prereq.get("ability").and_then(|v| v.as_array()) {
                for ab in ability_arr {
                    for key in ["str", "dex", "con", "int", "wis", "cha"] {
                        if let Some(req_val) = ab.get(key).and_then(|v| v.as_i64()) {
                            if crate::models::rules::ch_ability_score(ch, key) < req_val as i32 {
                                return false;
                            }
                        }
                    }
                }
            }

            // Level prerequisite: {"level": 4}
            if let Some(req_level) = prereq.get("level").and_then(|v| v.as_i64()) {
                let char_level = crate::models::rules::level_from_xp(ch.experience_pts);
                if char_level < req_level as i32 {
                    return false;
                }
            }
            // Other prereq types (campaign, class, race) are not filtered out —
            // we don't have enough context so we show the feat and let the player decide.
        }
        true
    }

    /// Open the ASI/Feat choice overlay (call when level-up is detected).
    pub fn open_asi_choice(&mut self) {
        self.picker_mode = PickerMode::AsiFeatChoice;
        self.asi_choice_index = 0;
        self.asi_mode = crate::app::AsiMode::PlusOneTwo;
        self.asi_ability_a = 0;
        self.asi_ability_b = 1;
        self.asi_ability_c = 2;
        self.picker_search.clear();
        self.picker_selected = 0;
        self.status_msg = "Level-up: choose ASI or Feat!".to_string();
    }
}
