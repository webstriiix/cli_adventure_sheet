use crate::App;
use crate::app::LevelUpPrompt;
use crate::models::app_state::PickerMode;

impl App {
    /// Call this after XP is saved and a level change is detected.
    /// Queues SubclassChoice and/or AsiOrFeat prompts for every level gained.
    ///
    /// `old_level` = level before XP change, `new_level` = level after.
    /// `class_id` and `class_name` refer to the primary class; multiclasses
    /// would need their own tracking (future work).
    pub fn check_level_up_prompts(&mut self, old_level: i32, new_level: i32) {
        if new_level <= old_level {
            return; // No level-up (or de-level — don't prompt)
        }

        let class_id = self.active_class_id;
        let class_name = self.char_class_name.clone();

        // Find the subclass gate level from class detail (default 3 for most classes)
        let subclass_gate = self
            .class_detail
            .as_ref()
            .and_then(|d| {
                d.features
                    .iter()
                    .find(|f| f.is_subclass_gate)
                    .map(|f| f.level)
            })
            .unwrap_or(3);

        let asi_levels = Self::asi_levels_for_class(&class_name);

        // Check each level between old+1 and new_level (inclusive)
        for lvl in (old_level + 1)..=(new_level) {
            // Subclass gate — only if subclass not already set
            if lvl == subclass_gate && self.char_subclass_name.is_empty() {
                self.level_up_queue.push(LevelUpPrompt::SubclassChoice {
                    class_id,
                    class_name: class_name.clone(),
                });
            }

            // ASI/Feat milestone
            if asi_levels.contains(&lvl) {
                self.level_up_queue.push(LevelUpPrompt::AsiOrFeat {
                    class_name: class_name.clone(),
                });
            }
        }
    }

    /// Called once per frame (from the sheet key handler) when picker_mode == None.
    /// If there are queued level-up prompts, opens the appropriate picker/overlay.
    pub fn drain_level_up_queue(&mut self) {
        if self.picker_mode != PickerMode::None || self.level_up_queue.is_empty() {
            return;
        }

        let prompt = self.level_up_queue.remove(0);
        match prompt {
            LevelUpPrompt::SubclassChoice { class_id, class_name } => {
                self.status_msg = format!("Level up! Choose a subclass for {}.", class_name);
                // Fetch class detail if not cached for this class
                let needs_fetch = self.class_detail.as_ref()
                    .map(|d| d.class.id != class_id)
                    .unwrap_or(true);

                if needs_fetch {
                    let source_slug = self.classes.iter()
                        .find(|c| c.id == class_id)
                        .map(|c| c.source_slug.clone())
                        .unwrap_or_default();
                    let rt = self.rt.clone();
                    match rt.block_on(self.client.get_class_detail(&class_name, &source_slug)) {
                        Ok(detail) => self.class_detail = Some(detail),
                        Err(e) => {
                            self.status_msg = format!("Could not load subclasses: {e}");
                            return;
                        }
                    }
                }

                self.subclass_picker_class_id = class_id;
                self.picker_mode = PickerMode::SubclassPicker;
                self.picker_selected = 0;
            }
            LevelUpPrompt::AsiOrFeat { class_name } => {
                self.status_msg = format!("Level up! {} gets an ASI or Feat.", class_name);
                self.open_asi_choice();
            }
        }
    }
}
