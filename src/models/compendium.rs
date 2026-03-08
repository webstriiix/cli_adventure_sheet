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
}

// ── Feat ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feat {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
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

// ── Monster ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub size: Vec<String>,
    #[serde(rename = "type")]
    pub monster_type: String,
    pub alignment: Vec<String>,
    pub ac: Vec<JsonValue>,
    pub hp_average: i32,
    pub hp_formula: String,
    pub speed: JsonValue,
    #[serde(rename = "str")]
    pub strength: i32,
    #[serde(rename = "dex")]
    pub dexterity: i32,
    #[serde(rename = "con")]
    pub constitution: i32,
    #[serde(rename = "int")]
    pub intelligence: i32,
    #[serde(rename = "wis")]
    pub wisdom: i32,
    #[serde(rename = "cha")]
    pub charisma: i32,
    pub skills: JsonValue,
    pub senses: Vec<String>,
    pub passive: i32,
    pub cr: String,
    pub traits: Vec<JsonValue>,
    pub actions: Vec<JsonValue>,
    pub reactions: Option<JsonValue>,
}

// ── Optional Feature ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalFeature {
    pub id: i32,
    pub name: String,
    pub source_id: i32,
    pub feature_type: String,
    pub prerequisite: Option<JsonValue>,
    pub entries: Vec<JsonValue>,
}

// ── Feature interpretation helpers ───────────────────────────────────────────

impl Feat {
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
        let text = match &self.entries {
            Some(arr) => json_array_to_text(arr),
            None => String::new(),
        };
        crate::models::features::interpret_feature(&text)
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
fn json_array_to_text(arr: &[JsonValue]) -> String {
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
