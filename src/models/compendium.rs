use serde::{Deserialize, Serialize};

use super::error::JsonValue;

// ── Class ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub hit_die: i32,
    pub proficiency_saves: Option<Vec<String>>,
    pub spellcasting_ability: Option<String>,
    pub caster_progression: Option<String>,
    pub weapon_proficiencies: Option<Vec<String>>,
    pub armor_proficiencies: Option<Vec<String>>,
    pub skill_choices: JsonValue,
    pub starting_equipment: JsonValue,
    pub multiclass_requirements: Option<JsonValue>,
    pub class_table: Option<Vec<JsonValue>>,
    /// 2D array of spell slots: spell_slots[character_level - 1][spell_level_index] = max slots.
    #[serde(default)]
    pub spell_slots: Option<Vec<Vec<i32>>>,
    /// Always-prepared/known/innate spells granted by class features.
    pub additional_spells: Option<JsonValue>,
    pub subclass_title: Option<String>,
    pub edition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassFeature {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub class_name: String,
    pub level: i32,
    pub entries: Option<Vec<JsonValue>>,
    pub is_subclass_gate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subclass {
    pub id: i32,
    pub name: String,
    pub short_name: String,
    pub source_slug: String,
    pub class_name: String,
    pub class_source: String,
    pub unlock_level: i32,
    pub fluff_text: Option<String>,
    pub fluff_image_url: Option<String>,
    /// Always-prepared/known/innate spells granted by this subclass (oath/domain/circle spells).
    #[serde(default)]
    pub additional_spells: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassFeature {
    pub id: i32,
    pub name: String,
    pub source_slug: String,
    pub subclass_short_name: String,
    pub subclass_source: String,
    pub class_name: String,
    pub level: i32,
    pub header: Option<crate::models::error::JsonValue>,
    pub entries: Option<Vec<crate::models::error::JsonValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassWithFeatures {
    pub subclass: Subclass,
    pub features: Vec<SubclassFeature>,
}

impl SubclassFeature {
    /// Convert to `ClassFeature`, combining `header` and `entries` so that
    /// description_text() works correctly in the Ctrl+K detail modal.
    pub fn to_class_feature(&self) -> ClassFeature {
        ClassFeature {
            id: self.id,
            name: self.name.clone(),
            source_slug: self.source_slug.clone(),
            class_name: self.class_name.clone(),
            level: self.level,
            entries: {
                let mut combined = self.entries.clone().unwrap_or_default();
                if let Some(hdr) = &self.header {
                    combined.insert(0, hdr.clone());
                }
                Some(combined)
            },
            is_subclass_gate: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDetailResponse {
    pub class: Class,
    pub features: Vec<ClassFeature>,
    pub subclasses: Vec<SubclassWithFeatures>,
}

// ── Race ──

/// Maps a race/background/spell source_id to a short human-readable label.
/// source_id=1 → PHB (2014), source_id=2 → XPHB (2024), etc.
pub fn source_id_label(source_id: i32) -> &'static str {
    match source_id {
        // ── Core rulebooks ──
        1 => "PHB",  // Player's Handbook 2014
        2 => "XPHB", // Player's Handbook 2024
        7 => "XPHB", // PHB 2024 (backgrounds use this id)
        28 => "PHB", // PHB 2014 backgrounds
        // ── Major supplements ──
        9 => "AAG",     // Astral Adventurer's Guide
        11 => "AKR",    // Amonkhet Reborn
        13 => "SOM",    // Shadows over Innistrad
        14 => "ERLW",   // Eberron: Rising from the Last War
        18 => "GGR",    // Guildmasters' Guide to Ravnica
        19 => "MOT",    // Mythic Odysseys of Theros
        21 => "ERLW",   // Eberron (extra backgrounds)
        26 => "IXALAN", // Rivals of Ixalan
        27 => "COS",    // Curse of Strahd
        45 => "WBtW",   // The Wild Beyond the Witchlight
        85 => "DSotDQ", // Dragonlance: Shadow of the Dragon Queen
        91 => "DSotDQ",
        118 => "EGW", // Explorer's Guide to Wildemount
        124 => "SCC", // Strixhaven: Curriculum of Chaos
        152 => "AI",  // Acquisitions Incorporated
        172 => "AI",
        196 => "SCAG",  // Sword Coast Adventurer's Guide
        259 => "ToA",   // Tomb of Annihilation
        267 => "BGDIA", // Baldur's Gate: Descent into Avernus
        280 => "CotN",  // Confrontation at Ogre Bridge (AL)
        282 => "CotN",
        287 => "CotN",
        292 => "CotN",
        311 => "GoS",   // Ghosts of Saltmarsh
        316 => "MPP",   // Morte's Planar Parade
        318 => "BGG",   // Bigby Presents: Glory of the Giants
        386 => "BMT",   // Book of Many Things
        703 => "MM",    // Monster Manual
        1198 => "MPMM", // Mordenkainen Presents: Monsters of the Multiverse
        1199 => "MPMM",
        1202 => "GGR",
        1204 => "PSK", // Plane Shift: Kaladesh
        1205 => "PSZ", // Plane Shift: Zendikar
        1207 => "ERLW",
        1208 => "EGW",
        1209 => "MOT",
        1210 => "TCE", // Tasha's Cauldron of Everything
        1214 => "FTD", // Fizban's Treasury of Dragons
        1215 => "SCC",
        1217 => "EEPC", // Elemental Evil Player's Companion
        1222 => "PLANESCAPE",
        1223 => "MPP",
        1226 => "BMT",
        1238 => "PSX", // Plane Shift: Ixalan
        1241 => "PSI", // Plane Shift: Innistrad
        1248 => "UA",  // Unearthed Arcana
        1261 => "BGDIA",
        1266 => "TDCSR", // Tal'Dorei Campaign Setting Reborn
        1281 => "PSI",
        1282 => "PSX",
        1287 => "IDRotF", // Icewind Dale: Rime of the Frostmaiden
        1303 => "SatO",   // Sigil and the Outlands
        1314 => "ToFW",   // Turn of Fortune's Wheel
        1320 => "AAG",
        1340 => "BAM", // Boo's Astral Menagerie
        1348 => "LoX", // Light of Xaryxis
        6715 => "Homebrew",
        _ => "Other",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub size: Vec<String>,
    pub speed: JsonValue,
    pub ability_bonuses: Vec<JsonValue>,
    pub age_description: Option<String>,
    pub alignment_description: Option<String>,
    pub skill_proficiencies: Option<JsonValue>,
    pub language_proficiencies: Option<Vec<JsonValue>>,
    pub trait_tags: Vec<String>,
    pub entries: Option<Vec<JsonValue>>,
    /// True if this race grants a free Origin feat at character creation (e.g. Human XPHB).
    #[serde(default)]
    pub grants_bonus_feat: bool,
}

// ── Background ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Background {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub skill_proficiencies: Option<Vec<JsonValue>>,
    pub tool_proficiencies: Option<Vec<JsonValue>>,
    pub language_count: Option<i32>,
    pub starting_equipment: Option<JsonValue>,
    pub entries: Option<Vec<JsonValue>>,
    /// True if this background grants a fixed bonus feat at character creation (all XPHB backgrounds).
    #[serde(default)]
    pub grants_bonus_feat: bool,
    #[serde(default)]
    pub ability_bonuses: Option<Vec<JsonValue>>,
}

// ── Spell ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spell {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub level: i32,
    pub school: String,
    pub casting_time: Option<Vec<JsonValue>>,
    pub range: Option<JsonValue>,
    pub components: Option<JsonValue>,
    pub duration: Option<Vec<JsonValue>>,
    pub entries: Option<Vec<JsonValue>>,
    pub entries_higher_lvl: Option<JsonValue>,
    pub ritual: Option<bool>,
    pub concentration: Option<bool>,
    /// List of classes that can cast this spell (e.g. [{"name": "Paladin", ...}])
    pub classes: Option<Vec<JsonValue>>,
}

// ── Feat ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feat {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    #[serde(default)]
    pub source_slug: Option<String>,
    pub prerequisite: Option<JsonValue>,
    pub ability: Option<JsonValue>,
    pub entries: JsonValue,
    pub has_uses: bool,
}

// ── Item ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub rarity: Option<String>,
    pub weight: Option<String>,
    pub value_cp: Option<i32>,
    pub damage: Option<JsonValue>,
    pub armor_class: Option<JsonValue>,
    pub properties: Option<Vec<String>>,
    pub requires_attune: Option<bool>,
    pub mastery: Option<Vec<String>>,
    pub entries: Option<JsonValue>,
    pub is_magic: Option<bool>,
}

// ── Class Resources ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassChoice {
    pub subclass_feature_id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubclassOption {
    pub gate_feature_id: i32,
    pub gate_feature_name: String,
    pub choices: Vec<SubclassChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassResourceResponse {
    pub class_name: String,
    pub source: String,
    pub level: i32,
    pub lay_on_hands_pool: Option<i32>,
    pub channel_divinity_uses: Option<i32>,
    pub subclass_options: Option<Vec<SubclassOption>>,
}

// ── Class display helpers ────────────────────────────────────────────────────

const _KNOWN_SKILLS: &[&str] = &[
    "Acrobatics",
    "Animal Handling",
    "Arcana",
    "Athletics",
    "Deception",
    "History",
    "Insight",
    "Intimidation",
    "Investigation",
    "Medicine",
    "Nature",
    "Perception",
    "Performance",
    "Persuasion",
    "Religion",
    "Sleight of Hand",
    "Stealth",
    "Survival",
];

impl Class {
    /// Parses `skill_choices` (5e-tools JSON) into a human-readable summary.
    /// Supports both 2014 (PHB) and 2024 (XPHB) schema variations.
    /// Example output: "Choose 2 from: Arcana, History, …"
    pub fn skill_choices_summary(&self) -> String {
        let arr = match self.skill_choices.as_array() {
            Some(a) if !a.is_empty() => a,
            _ => return String::new(),
        };
        let mut parts: Vec<String> = Vec::new();
        for entry in arr {
            let choose_count = if let Some(n) = entry.get("choose").and_then(|v| v.as_i64()) {
                n as usize
            } else if let Some(choose_obj) = entry.get("choose").and_then(|v| v.as_object()) {
                choose_obj
                    .get("count")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as usize
            } else if let Some(n) = entry.get("count").and_then(|v| v.as_i64()) {
                n as usize
            } else {
                0
            };

            if choose_count > 0 {
                let from_array = entry.get("from").and_then(|v| v.as_array()).or_else(|| {
                    entry
                        .get("choose")
                        .and_then(|v| v.as_object())
                        .and_then(|obj| obj.get("from"))
                        .and_then(|v| v.as_array())
                });

                let from_labels: Vec<String> = from_array
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                v.as_str()
                                    .or_else(|| v.get("name").and_then(|n| n.as_str()))
                                    .map(|s| {
                                        s.split_whitespace()
                                            .map(|w| {
                                                let mut c = w.chars();
                                                match c.next() {
                                                    None => String::new(),
                                                    Some(f) => {
                                                        f.to_uppercase().collect::<String>()
                                                            + c.as_str()
                                                    }
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                    })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                if from_labels.is_empty() {
                    parts.push(format!("Choose {}", choose_count));
                } else {
                    parts.push(format!(
                        "Choose {} from: {}",
                        choose_count,
                        from_labels.join(", ")
                    ));
                }
            }
        }
        parts.join("; ")
    }

    /// Formats `starting_equipment` JSON into a compact summary line.
    pub fn starting_equipment_summary(&self) -> String {
        let val = &self.starting_equipment;

        // 5e-tools "defaultData" wrapper
        let data = val.get("defaultData").or_else(|| {
            // Also try direct array
            if val.is_array() { Some(val) } else { None }
        });

        let arr = match data.and_then(|v| v.as_array()) {
            Some(a) if !a.is_empty() => a,
            _ => return String::new(),
        };

        let mut items: Vec<String> = Vec::new();
        for entry in arr {
            if let Some(item_val) = entry.get("item") {
                let name = item_val
                    .as_str()
                    .unwrap_or("")
                    .split('|')
                    .next()
                    .unwrap_or("")
                    .trim();
                let qty = entry.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1);
                if !name.is_empty() {
                    if qty > 1 {
                        items.push(format!("{} ×{}", name, qty));
                    } else {
                        items.push(name.to_string());
                    }
                }
            }
        }
        if items.is_empty() {
            String::new()
        } else {
            items.join(", ")
        }
    }
}

// ── Feature interpretation helpers ───────────────────────────────────────────

impl ClassFeature {
    /// Returns `true` if `current_level` meets or exceeds this feature's required level.
    pub fn is_unlocked(&self, current_level: i32) -> bool {
        current_level >= self.level
    }

    /// Render all entries into a flat plain-text description suitable for the
    /// Ctrl+K detail modal.
    pub fn description_text(&self) -> String {
        match &self.entries {
            Some(arr) => json_array_to_text(arr),
            None => String::new(),
        }
    }

    /// How many interactive choice slots this feature exposes at character creation.
    /// Returns 0 for static features that need no player input.
    pub fn slot_count(&self) -> usize {
        match self.interpret() {
            crate::models::features::Feature::WeaponMastery { choose } => choose as usize,
            crate::models::features::Feature::Asi { .. } => 0, // handled by abilities step
            crate::models::features::Feature::SkillChoice { choose } => choose as usize,
            crate::models::features::Feature::GrantsOriginFeat { choose } => choose as usize,
            crate::models::features::Feature::Choice { choose, .. } => choose as usize,
            crate::models::features::Feature::Spells { choose, .. } => choose as usize,
            _ => 0,
        }
    }

    /// A short label for what kind of thing gets picked in each slot.
    /// Used as the modal title prefix.
    pub fn choice_kind(&self) -> &'static str {
        match self.interpret() {
            crate::models::features::Feature::WeaponMastery { .. } => "Weapon Mastery",
            crate::models::features::Feature::SkillChoice { .. } => "Skill",
            crate::models::features::Feature::GrantsOriginFeat { .. } => "Origin Feat",
            crate::models::features::Feature::Choice { .. } => "Choice",
            crate::models::features::Feature::Spells { .. } => "Spell",
            _ => "Option",
        }
    }
}

impl Feat {
    /// Mengekstrak minimum level yang dibutuhkan feat dari kolom JSON prerequisite.
    /// Jika tidak ada syarat level, defaultnya adalah Level 1.
    pub fn min_level(&self) -> i32 {
        if let Some(reqs) = &self.prerequisite {
            if let Some(arr) = reqs.as_array() {
                for req in arr {
                    // Check schema 2024 (langsung "level": 4) didalam array object
                    if let Some(lvl) = req.get("level").and_then(|v| v.as_i64()) {
                        return lvl as i32;
                    }
                    // Check schema bersarang {"level": {"level": 19}}
                    if let Some(obj) = req.get("level").and_then(|v| v.as_object()) {
                        if let Some(lvl) = obj.get("level").and_then(|v| v.as_i64()) {
                            return lvl as i32;
                        }
                    }
                }
            } else if let Some(obj) = reqs.as_object() {
                if let Some(lvl) = obj.get("level").and_then(|v| v.as_i64()) {
                    return lvl as i32;
                }
                if let Some(inner) = obj.get("level").and_then(|v| v.as_object()) {
                    if let Some(lvl) = inner.get("level").and_then(|v| v.as_i64()) {
                        return lvl as i32;
                    }
                }
            }
        }
        1 // Default: Origin feat (Bisa diambil kapan saja)
    }

    pub fn source_slug_str(&self) -> &str {
        if let Some(ref slug) = self.source_slug {
            if !slug.is_empty() {
                return slug.as_str();
            }
        }
        source_id_label(self.source_id)
    }

    /// Interpret this feat's description text into a structured [`Feature`].
    ///
    /// Extracts plain strings from the `entries` JSON array, concatenates them,
    /// and delegates to the interpreter heuristics.
    pub fn interpret(&self) -> crate::models::features::Feature {
        let text = json_entries_to_text(&self.entries);
        crate::models::features::interpret_feature(&text)
    }
}

impl ClassFeature {
    /// Interpret this class feature's description text into a structured [`Feature`].
    pub fn interpret(&self) -> crate::models::features::Feature {
        let body = match &self.entries {
            Some(arr) => json_array_to_text(arr),
            None => String::new(),
        };
        // Prepend name so the interpreter sees "Weapon Mastery. Your training..."
        // This ensures features named "Weapon Mastery" are detected even if the text varies.
        let combined = format!("{}. {}", self.name, body);
        crate::models::features::interpret_feature(&combined)
    }
}

/// Flatten a `serde_json::Value` (expected to be an array) into a single text blob.
fn json_entries_to_text(val: &JsonValue) -> String {
    match val.as_array() {
        Some(arr) => json_array_to_text(arr),
        None => val.as_str().unwrap_or("").to_string(),
    }
}

/// Recursively pull readable strings out of a JSON array (5e-tools schema).
pub fn json_array_to_text(arr: &[JsonValue]) -> String {
    let mut parts = Vec::new();
    for entry in arr {
        collect_text(entry, &mut parts);
    }
    parts.join(" ")
}

fn collect_text(val: &JsonValue, out: &mut Vec<String>) {
    match val {
        JsonValue::String(s) => {
            if !s.is_empty() {
                out.push(s.clone());
            }
        }
        JsonValue::Object(o) => {
            // Try "entries" sub-array, then "name" string
            if let Some(arr) = o.get("entries").and_then(|v| v.as_array()) {
                for e in arr {
                    collect_text(e, out);
                }
            }
            if let Some(arr) = o.get("items").and_then(|v| v.as_array()) {
                for e in arr {
                    collect_text(e, out);
                }
            }
            if let Some(s) = o.get("name").and_then(|v| v.as_str()) {
                out.push(s.to_string());
            }
        }
        JsonValue::Array(arr) => {
            for e in arr {
                collect_text(e, out);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Subrace {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub race_id: i32,
    pub speed: Option<JsonValue>,
    pub ability_bonuses: Option<JsonValue>,
    pub entries: Option<JsonValue>,
}

// ── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_feat_min_level_default_origin() {
        let feat: Feat = serde_json::from_value(json!({
            "id": 1,
            "name": "Alert",
            "source_id": 2,
            "prerequisite": null,
            "ability": null,
            "entries": [],
            "has_uses": false
        }))
        .unwrap();
        assert_eq!(feat.min_level(), 1);
        assert_eq!(feat.source_slug_str(), "XPHB");
    }

    #[test]
    fn test_feat_min_level_direct_level_4() {
        let feat: Feat = serde_json::from_value(json!({
            "id": 2,
            "name": "Actor",
            "source_id": 1,
            "source_slug": "phb",
            "prerequisite": [{"level": 4}],
            "ability": null,
            "entries": [],
            "has_uses": false
        }))
        .unwrap();
        assert_eq!(feat.min_level(), 4);
        assert_eq!(feat.source_slug_str(), "phb");
    }

    #[test]
    fn test_feat_min_level_nested_epic_boon() {
        let feat: Feat = serde_json::from_value(json!({
            "id": 3,
            "name": "Boon of Combat Prowess",
            "source_id": 2,
            "source_slug": "xphb",
            "prerequisite": [{"level": {"level": 19}}],
            "ability": null,
            "entries": [],
            "has_uses": false
        }))
        .unwrap();
        assert_eq!(feat.min_level(), 19);
        assert_eq!(feat.source_slug_str(), "xphb");
    }
}
