use crate::App;
use crate::models::character::{AddCharacterClassRequest, CharacterClass, PatchCharacterClassRequest};
use crate::models::app_state::{MulticlassSection, PickerMode};

impl App {
    /// Add the selected class from the add-picker as a multiclass.
    /// POST /characters/{id}/classes — returns updated Character.
    pub fn add_multiclass(&mut self) {
        let character_id = match self.active_character.as_ref().map(|c| c.id) {
            Some(id) => id,
            None => return,
        };

        let picked = match self.classes.get(self.multiclass_add_index) {
            Some(c) => c.clone(),
            None => return,
        };

        // Prevent adding the primary class again or a duplicate multiclass
        if picked.id == self.active_class_id {
            self.status_msg = "Already your primary class.".to_string();
            return;
        }
        if self.char_classes.iter().any(|cc| cc.class_id == picked.id) {
            self.status_msg = "Already multiclassed into that class.".to_string();
            return;
        }

        let req = AddCharacterClassRequest { class_id: picked.id };
        let rt = self.rt.clone();
        match rt.block_on(self.client.add_character_class(character_id, &req)) {
            Ok(updated_char) => {
                // Record the new multiclass entry locally (level 1, no subclass yet)
                self.char_classes.push(CharacterClass {
                    id: 0, // server doesn't return the row id directly
                    character_id,
                    class_id: picked.id,
                    level: 1,
                    is_primary: false,
                    subclass_id: None,
                });
                self.active_character = Some(updated_char);
                self.multiclass_section = MulticlassSection::List;
                self.status_msg = format!("Added multiclass: {}", picked.name);
            }
            Err(e) => self.status_msg = format!("Failed to add multiclass: {e}"),
        }
    }

    /// Level up a multiclass entry (increase its level by 1).
    /// PATCH /characters/{id}/classes/{class_id}
    pub fn level_up_multiclass(&mut self) {
        let character_id = match self.active_character.as_ref().map(|c| c.id) {
            Some(id) => id,
            None => return,
        };

        let cc = match self.char_classes.get(self.multiclass_selected).cloned() {
            Some(cc) => cc,
            None => return,
        };

        let new_level = cc.level + 1;
        let req = PatchCharacterClassRequest {
            level: Some(new_level),
            subclass_id: None,
        };
        let rt = self.rt.clone();
        match rt.block_on(self.client.patch_character_class(character_id, cc.class_id, &req)) {
            Ok(updated_char) => {
                self.char_classes[self.multiclass_selected].level = new_level;
                self.active_character = Some(updated_char);
                self.status_msg = format!("Multiclass level updated to {new_level}.");
            }
            Err(e) => self.status_msg = format!("Failed to update level: {e}"),
        }
    }

    /// Set a subclass on a multiclass entry.
    /// PATCH /characters/{id}/classes/{class_id}
    pub fn set_multiclass_subclass(&mut self, class_id: i32, subclass_id: i32) {
        let character_id = match self.active_character.as_ref().map(|c| c.id) {
            Some(id) => id,
            None => return,
        };

        let req = PatchCharacterClassRequest {
            level: None,
            subclass_id: Some(subclass_id),
        };
        let rt = self.rt.clone();
        match rt.block_on(self.client.patch_character_class(character_id, class_id, &req)) {
            Ok(updated_char) => {
                if let Some(cc) = self.char_classes.iter_mut().find(|cc| cc.class_id == class_id) {
                    cc.subclass_id = Some(subclass_id);
                }
                self.active_character = Some(updated_char);
                self.status_msg = "Subclass set.".to_string();
            }
            Err(e) => self.status_msg = format!("Failed to set subclass: {e}"),
        }
    }

    /// Set level/subclass on the primary class.
    /// PATCH /characters/{id}/classes/{primary_class_id}
    pub fn patch_primary_class(&mut self, level: Option<i32>, subclass_id: Option<i32>) {
        let character_id = match self.active_character.as_ref().map(|c| c.id) {
            Some(id) => id,
            None => return,
        };
        let class_id = self.active_class_id;
        let req = PatchCharacterClassRequest { level, subclass_id };
        let rt = self.rt.clone();
        match rt.block_on(self.client.patch_character_class(character_id, class_id, &req)) {
            Ok(updated_char) => {
                self.active_character = Some(updated_char);
                self.status_msg = "Class updated.".to_string();
            }
            Err(e) => self.status_msg = format!("Failed to update class: {e}"),
        }
    }

    /// Build a display string of all classes (primary + multiclasses), e.g. "Wizard 5 / Fighter 2"
    pub fn all_classes_display(&self) -> String {
        let primary_level = self
            .active_character
            .as_ref()
            .map(|c| crate::utils::level_from_xp(c.experience_pts))
            .unwrap_or(1);

        if self.char_classes.is_empty() {
            return format!("{} {}", self.char_class_name, primary_level);
        }

        // Calculate primary effective level (Total level - sum of multiclass levels)
        let mc_total: i32 = self
            .char_classes
            .iter()
            .filter(|cc| !cc.is_primary && cc.class_id != self.active_class_id)
            .map(|cc| cc.level)
            .sum();
        let primary_effective = (primary_level - mc_total).max(1);

        let mut parts = vec![format!("{} {}", self.char_class_name, primary_effective)];
        
        for cc in &self.char_classes {
            // Only add if it's not the primary class (to avoid "Paladin 3 / Paladin 3")
            if cc.is_primary || cc.class_id == self.active_class_id {
                continue;
            }
            
            let name = self
                .classes
                .iter()
                .find(|c| c.id == cc.class_id)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            parts.push(format!("{} {}", name, cc.level));
        }
        parts.join(" / ")
    }

    /// Open the subclass picker for the primary class (class_id == 0 means primary).
    /// Fetches class detail if not already cached.
    pub fn open_subclass_picker(&mut self, class_id: i32) {
        // If class_id == 0, use the active primary class
        let target_class_id = if class_id == 0 { self.active_class_id } else { class_id };

        // Find class name + source to fetch detail
        let (class_name, source_slug) = match self.classes.iter().find(|c| c.id == target_class_id) {
            Some(c) => (c.name.clone(), c.source_slug.clone()),
            None => {
                self.status_msg = "Class not found.".to_string();
                return;
            }
        };

        // Fetch class detail if needed (or if wrong class is cached)
        let needs_fetch = self.class_detail.as_ref()
            .map(|d| d.class.id != target_class_id)
            .unwrap_or(true);

        if needs_fetch {
            let rt = self.rt.clone();
            match rt.block_on(self.client.get_class_detail(&class_name, &source_slug)) {
                Ok(detail) => {
                    self.class_detail = Some(detail);
                }
                Err(e) => {
                    self.status_msg = format!("Failed to load subclasses: {e}");
                    return;
                }
            }
        }

        let subclass_count = self.class_detail.as_ref().map(|d| d.subclasses.len()).unwrap_or(0);
        if subclass_count == 0 {
            self.status_msg = format!("No subclasses found for {}.", class_name);
            return;
        }

        self.subclass_picker_class_id = target_class_id;
        self.picker_mode = PickerMode::SubclassPicker;
        self.picker_selected = 0;
        self.status_msg = format!("Choose a subclass for {}:", class_name);
    }

    /// Confirm the selected subclass from the SubclassPicker overlay.
    pub fn confirm_subclass_pick(&mut self) {
        let detail = match &self.class_detail {
            Some(d) => d.clone(),
            None => return,
        };

        let swf = match detail.subclasses.get(self.picker_selected) {
            Some(s) => s.clone(),
            None => return,
        };

        let subclass_id = swf.subclass.id;
        let subclass_name = swf.subclass.name.clone();
        let class_id = self.subclass_picker_class_id;

        if class_id == self.active_class_id {
            self.patch_primary_class(None, Some(subclass_id));
        } else {
            self.set_multiclass_subclass(class_id, subclass_id);
        }

        // Update cached subclass name for primary class display
        if class_id == self.active_class_id {
            self.char_subclass_name = subclass_name.clone();
        }

        self.picker_mode = PickerMode::None;
        self.status_msg = format!("Subclass set: {}", subclass_name);
    }
}
