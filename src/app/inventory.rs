use crate::models::app_state::PickerMode;
use crate::models::character::{AddInventoryRequest, UpdateInventoryRequest};
use crate::models::compendium::Item;
use crate::App;

impl App {
    pub fn remove_selected_inventory_item(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_inventory.is_empty() {
            return;
        }

        let item = &self.char_inventory[self.selected_list_index];
        let inventory_id = item.id;

        let rt = self.rt.clone();
        match rt.block_on(
            self.client
                .remove_inventory_item(character_id, inventory_id),
        ) {
            Ok(()) => {
                self.char_inventory.remove(self.selected_list_index);
                if self.selected_list_index > 0
                    && self.selected_list_index >= self.char_inventory.len()
                {
                    self.selected_list_index -= 1;
                }
                self.status_msg = "Item removed".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Failed to remove item: {e}");
            }
        }
    }

    pub fn toggle_inventory_equipped(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_inventory.is_empty() {
            return;
        }

        let item = &self.char_inventory[self.selected_list_index];
        let inventory_id = item.id;
        let new_equipped = !item.is_equipped;

        let req = UpdateInventoryRequest {
            is_equipped: Some(new_equipped),
            ..Default::default()
        };

        let rt = self.rt.clone();
        match rt.block_on(
            self.client
                .update_inventory_item(character_id, inventory_id, &req),
        ) {
            Ok(updated) => {
                self.char_inventory[self.selected_list_index] = updated;
                self.status_msg = if new_equipped {
                    "Item equipped".to_string()
                } else {
                    "Item unequipped".to_string()
                };
            }
            Err(e) => {
                self.status_msg = format!("Failed to update item: {e}");
            }
        }
    }

    pub fn toggle_inventory_attuned(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_inventory.is_empty() {
            return;
        }

        let item = &self.char_inventory[self.selected_list_index];
        let inventory_id = item.id;
        let new_attuned = !item.is_attuned;

        let req = UpdateInventoryRequest {
            is_attuned: Some(new_attuned),
            ..Default::default()
        };

        let rt = self.rt.clone();
        match rt.block_on(
            self.client
                .update_inventory_item(character_id, inventory_id, &req),
        ) {
            Ok(updated) => {
                self.char_inventory[self.selected_list_index] = updated;
                self.status_msg = if new_attuned {
                    "Item attuned".to_string()
                } else {
                    "Item unattuned".to_string()
                };
            }
            Err(e) => {
                self.status_msg = format!("Failed to update item: {e}");
            }
        }
    }

    pub fn update_inventory_quantity(&mut self, delta: i32) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        if self.char_inventory.is_empty() {
            return;
        }

        let item = &self.char_inventory[self.selected_list_index];
        let inventory_id = item.id;
        let new_qty = item.quantity + delta;
        if new_qty < 0 {
            return;
        }

        let req = UpdateInventoryRequest {
            quantity: Some(new_qty),
            ..Default::default()
        };

        let rt = self.rt.clone();
        match rt.block_on(
            self.client
                .update_inventory_item(character_id, inventory_id, &req),
        ) {
            Ok(updated) => {
                self.char_inventory[self.selected_list_index] = updated;
                self.status_msg = format!("Quantity: {new_qty}");
            }
            Err(e) => {
                self.status_msg = format!("Failed to update quantity: {e}");
            }
        }
    }

    pub fn add_item_from_picker(&mut self) {
        let character_id = match &self.active_character {
            Some(c) => c.id,
            None => return,
        };

        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }

        let item_id = filtered[self.picker_selected].id;
        let req = AddInventoryRequest {
            item_id,
            quantity: Some(1),
            is_equipped: None,
            is_attuned: None,
            notes: None,
        };

        let rt = self.rt.clone();
        match rt.block_on(self.client.add_inventory_item(character_id, &req)) {
            Ok(inv_item) => {
                self.char_inventory.push(inv_item);
                self.status_msg = "Item added!".to_string();
                self.picker_mode = PickerMode::None;
                self.show_item_detail = false;
            }
            Err(e) => {
                self.status_msg = format!("Failed to add item: {e}");
            }
        }
    }

    pub fn filtered_items(&self) -> Vec<&Item> {
        let search = self.picker_search.to_lowercase();
        self.all_items
            .iter()
            .filter(|i| search.is_empty() || i.name.to_lowercase().contains(&search))
            .take(50)
            .collect()
    }
}
