use crate::App;
use crate::models::character::AddInventoryRequest;

impl App {
    /// Extract (item_name, quantity) pairs from a `defaultData` starting_equipment value.
    /// Only reads option "A" (index 0) — the standard package.
    pub fn parse_starting_equipment_items(eq: &serde_json::Value) -> Vec<(String, i32)> {
        let mut out = Vec::new();
        let default_data = match eq.get("defaultData").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return out,
        };
        for choice in default_data {
            // XPHB classes use "A", backgrounds use "a"
            let option_a = choice
                .get("A")
                .or_else(|| choice.get("a"))
                .and_then(|v| v.as_array());
            if let Some(entries) = option_a {
                for entry in entries {
                    // Class format: {"item": "dagger|xphb", "quantity": 1}
                    // Background format: "dagger|xphb" (plain string)
                    let raw = if let Some(s) = entry.as_str() {
                        Some((s, 1i32))
                    } else if let Some(item_str) = entry.get("item").and_then(|v| v.as_str()) {
                        let qty =
                            entry.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                        Some((item_str, qty))
                    } else {
                        None // {"value": ...} gold entry — skip
                    };

                    if let Some((item_str, qty)) = raw {
                        // Strip source suffix: "dagger|xphb" → "dagger"
                        let name = item_str
                            .split('|')
                            .next()
                            .unwrap_or(item_str)
                            .trim()
                            .to_string();
                        out.push((name, qty));
                    }
                }
            }
        }
        out
    }

    pub fn add_starting_items(
        &self,
        rt: &tokio::runtime::Handle,
        character_id: uuid::Uuid,
        items: &[(String, i32)],
    ) {
        for (name, qty) in items {
            // Case-insensitive exact match
            let item_id = self
                .all_items
                .iter()
                .find(|i| i.name.eq_ignore_ascii_case(name))
                .map(|i| i.id);

            if let Some(id) = item_id {
                let req = AddInventoryRequest {
                    item_id: id,
                    quantity: Some(*qty),
                    is_equipped: None,
                    is_attuned: None,
                    notes: None,
                };
                let _ = rt.block_on(self.client.add_inventory_item(character_id, &req));
            }
        }
    }
}
