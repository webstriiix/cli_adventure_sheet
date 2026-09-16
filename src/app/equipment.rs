use crate::App;
use crate::models::character::AddInventoryRequest;

impl App {
    // ─────────────────────────────────────────────────────────────────────────
    // Equipment step population
    // ─────────────────────────────────────────────────────────────────────────

    /// Build the display strings for Step 5 and store them in
    /// `builder.equipment_options`.  Also calculates `builder.starting_gold`
    /// from the class (and optionally background) Option B gold entries.
    ///
    /// Call this once when transitioning from Step 4 → Step 5.
    pub fn populate_equipment_options(&mut self) {
        self.builder.equipment_options.clear();
        self.builder.starting_gold = None;

        let class_id = match self.builder.class_id {
            Some(id) => id,
            None => return,
        };
        let class = match self.classes.iter().find(|c| c.id == class_id).cloned() {
            Some(c) => c,
            None => return,
        };

        let eq = &class.starting_equipment;

        // Does this class want background equipment included in Option A?
        let additional_from_bg = eq
            .get("additionalFromBackground")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // ── Option A: item list ──────────────────────────────────────────────
        // Class Option A items
        let class_items = Self::extract_option_a_labels(eq);

        // Background Option A items (if additionalFromBackground = true)
        let bg_items: Vec<String> = if additional_from_bg {
            self.builder
                .bg_id
                .and_then(|id| self.backgrounds.iter().find(|b| b.id == id))
                .and_then(|b| b.starting_equipment.as_ref())
                .map(|bg_eq| {
                    // Background eq is a raw array, so wrap it the same way
                    // parse_starting_equipment_items expects
                    let wrapped = serde_json::json!({ "defaultData": bg_eq });
                    Self::extract_option_a_labels(&wrapped)
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Merge into display strings
        let mut option_a_lines: Vec<String> = Vec::new();
        if !class_items.is_empty() {
            option_a_lines.push(format!(
                "── {} equipment ──",
                self.builder
                    .class_id
                    .and_then(|id| self.classes.iter().find(|c| c.id == id))
                    .map(|c| c.name.as_str())
                    .unwrap_or("Class")
            ));
            option_a_lines.extend(class_items);
        }
        if !bg_items.is_empty() {
            option_a_lines.push(String::new()); // blank separator
            option_a_lines.push(format!(
                "── {} equipment ──",
                self.builder
                    .bg_id
                    .and_then(|id| self.backgrounds.iter().find(|b| b.id == id))
                    .map(|b| b.name.as_str())
                    .unwrap_or("Background")
            ));
            option_a_lines.extend(bg_items);
        }

        if option_a_lines.is_empty() {
            option_a_lines.push("(no equipment data available)".to_string());
        }

        self.builder.equipment_options = option_a_lines;

        // ── Option B: gold ───────────────────────────────────────────────────
        let class_gold = Self::extract_option_b_gold(eq);
        let bg_gold: i32 = if additional_from_bg {
            self.builder
                .bg_id
                .and_then(|id| self.backgrounds.iter().find(|b| b.id == id))
                .and_then(|b| b.starting_equipment.as_ref())
                .map(|bg_eq| {
                    let wrapped = serde_json::json!({ "defaultData": bg_eq });
                    Self::extract_option_b_gold(&wrapped)
                })
                .unwrap_or(0)
        } else {
            0
        };

        let total_gold = class_gold + bg_gold;
        if total_gold > 0 {
            self.builder.starting_gold = Some(total_gold);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // JSON extraction helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract human-readable item labels from Option "A" of a starting_equipment blob.
    /// Returns one string per item, e.g. "  • 2x Dagger", "  • Spellbook".
    fn extract_option_a_labels(eq: &serde_json::Value) -> Vec<String> {
        let default_data = match eq.get("defaultData").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Vec::new(),
        };

        let mut lines = Vec::new();
        for choice in default_data {
            let option_a = choice
                .get("A")
                .or_else(|| choice.get("a"))
                .and_then(|v| v.as_array());

            if let Some(entries) = option_a {
                for entry in entries {
                    if let Some(label) = Self::entry_to_label(entry) {
                        lines.push(format!("  • {}", label));
                    }
                }
            }
        }
        lines
    }

    /// Public wrapper around `extract_option_b_gold` for use in UI rendering.
    pub fn extract_option_b_gold_pub(eq: &serde_json::Value) -> i32 {
        Self::extract_option_b_gold(eq)
    }

    /// Extract the total Option B gold (in GP) from a starting_equipment blob.
    /// Looks for `{"value": N}` entries (value is in CP; divide by 100 for GP).
    fn extract_option_b_gold(eq: &serde_json::Value) -> i32 {
        let default_data = match eq.get("defaultData").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return 0,
        };

        let mut total_gp = 0i32;
        for choice in default_data {
            // Option B gold is stored under key "B" or "b"
            let option_b = choice
                .get("B")
                .or_else(|| choice.get("b"))
                .and_then(|v| v.as_array());

            if let Some(entries) = option_b {
                for entry in entries {
                    // Format: {"value": 5500}  (value in CP)
                    if let Some(cp) = entry.get("value").and_then(|v| v.as_i64()) {
                        total_gp += (cp / 100) as i32;
                    }
                }
            }
        }
        total_gp
    }

    /// Convert a single starting_equipment entry to a display label.
    fn entry_to_label(entry: &serde_json::Value) -> Option<String> {
        // Plain string: "dagger|xphb"
        if let Some(s) = entry.as_str() {
            let name = Self::strip_source_suffix(s);
            return Some(Self::capitalize(&name));
        }

        // Object with "item" key: {"item": "dagger|xphb", "quantity": 2}
        if let Some(item_str) = entry.get("item").and_then(|v| v.as_str()) {
            let name = Self::strip_source_suffix(item_str);
            let qty = entry.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1);
            let label = if qty > 1 {
                format!("{}x {}", qty, Self::capitalize(&name))
            } else {
                Self::capitalize(&name)
            };
            return Some(label);
        }

        // Object with "value" key: gold — skip in Option A
        if entry.get("value").is_some() {
            let cp = entry.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
            let gp = cp / 100;
            if gp > 0 {
                return Some(format!("{} GP", gp));
            }
        }

        // Nested choice object with "choose" key — show as a generic label
        if let Some(choose) = entry.get("choose") {
            if let Some(from_arr) = choose.get("from").and_then(|v| v.as_array()) {
                let choices: Vec<String> = from_arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| Self::capitalize(&Self::strip_source_suffix(s)))
                    .collect();
                if !choices.is_empty() {
                    let count = choose.get("count").and_then(|v| v.as_i64()).unwrap_or(1);
                    return Some(format!("Choose {} of: {}", count, choices.join(", ")));
                }
            }
        }

        None
    }

    /// Strip the `|source` suffix from 5e-tools item strings.
    /// "dagger|xphb" → "dagger", "quarterstaff" → "quarterstaff"
    fn strip_source_suffix(s: &str) -> String {
        s.split('|').next().unwrap_or(s).trim().replace('-', " ")
    }

    /// Capitalise the first letter of a string.
    fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Starting-item submission helpers (used in builder.rs on finalise)
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract (item_name, quantity) pairs from a `defaultData` starting_equipment value.
    /// Only reads Option "A" (index 0) — the standard package.
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
                    let raw = if let Some(s) = entry.as_str() {
                        Some((s.to_string(), 1i32))
                    } else if let Some(item_str) = entry.get("item").and_then(|v| v.as_str()) {
                        let qty =
                            entry.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                        Some((item_str.to_string(), qty))
                    } else {
                        None // {"value": ...} gold entry — skip
                    };

                    if let Some((item_str, qty)) = raw {
                        // Strip source suffix: "dagger|xphb" → "dagger"
                        let name = item_str
                            .split('|')
                            .next()
                            .unwrap_or(&item_str)
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
