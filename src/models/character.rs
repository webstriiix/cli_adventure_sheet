use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Character ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Character {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub experience_pts: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
    pub class_id: Option<i32>,
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
    pub max_hp: i32,
    pub current_hp: i32,
    pub temp_hp: i32,
    pub inspiration: bool,
    pub notes: Option<String>,
    pub death_saves_successes: i32,
    pub death_saves_failures: i32,
    pub cp: i32,
    pub sp: i32,
    pub ep: i32,
    pub gp: i32,
    pub pp: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterSpellSlot {
    pub character_id: Uuid,
    pub slot_level: i32,
    pub expended: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterHitDice {
    pub character_id: Uuid,
    pub die_size: i32,
    pub expended: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCharacterRequest {
    pub name: String,
    pub class_id: i32,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
    pub background_id: Option<i32>,
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
    pub max_hp: i32,
    /// Feat granted by race/species at creation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonus_feat_id: Option<i32>,
    /// Feat granted by background at creation (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_feat_id: Option<i32>,
}

// ── ASI Choice ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsiChoiceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_str: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_dex: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_con: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_int: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_wis: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bump_cha: Option<i32>,
    /// Feat to grant instead of ability score increase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feat_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
}

/// PUT /characters/{id} expects the same shape as POST (required fields must be present).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCharacterRequest {
    // Required by the API (same as create)
    pub name: String,
    pub class_id: i32,
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
    pub max_hp: i32,
    // Optional extras
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subrace_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_hp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inspiration: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experience_pts: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_saves_successes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_saves_failures: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ep: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pp: Option<i32>,
}

impl UpdateCharacterRequest {
    /// Build a request pre-filled from an existing Character, with a given class_id.
    pub fn from_character(c: &Character, class_id: i32) -> Self {
        Self {
            name: c.name.clone(),
            class_id,
            strength: c.strength,
            dexterity: c.dexterity,
            constitution: c.constitution,
            intelligence: c.intelligence,
            wisdom: c.wisdom,
            charisma: c.charisma,
            max_hp: c.max_hp,
            race_id: c.race_id,
            subrace_id: c.subrace_id,
            background_id: c.background_id,
            current_hp: Some(c.current_hp),
            temp_hp: Some(c.temp_hp),
            inspiration: Some(c.inspiration),
            notes: c.notes.clone(),
            experience_pts: Some(c.experience_pts),
            death_saves_successes: Some(c.death_saves_successes),
            death_saves_failures: Some(c.death_saves_failures),
            cp: Some(c.cp),
            sp: Some(c.sp),
            ep: Some(c.ep),
            gp: Some(c.gp),
            pp: Some(c.pp),
        }
    }
}

// ── Multiclass ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterClass {
    pub id: i32,
    pub character_id: Uuid,
    pub class_id: i32,
    pub level: i32,
    #[serde(default)]
    pub is_primary: bool,
    pub subclass_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCharacterClassRequest {
    pub class_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCharacterClassRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subclass_id: Option<i32>,
}

// ── Character Feats ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterFeat {
    pub id: i32,
    pub character_id: Uuid,
    pub feat_id: i32,
    pub chosen_ability: Option<String>,
    pub uses_remaining: Option<i32>,
    pub uses_max: Option<i32>,
    pub recharge_on: Option<String>,
    #[serde(default)]
    pub source_type: String,
    pub gained_at_level: Option<i32>,
}

// ── Character Spells ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterSpell {
    pub character_id: Uuid,
    pub spell_id: i32,
    pub is_prepared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSpellRequest {
    pub spell_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_prepared: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSpellRequest {
    pub is_prepared: bool,
}

// ── Character Inventory ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: i32,
    pub character_id: Uuid,
    pub item_id: i32,
    pub quantity: i32,
    pub is_equipped: bool,
    pub is_attuned: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInventoryRequest {
    pub item_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_equipped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_attuned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateInventoryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_equipped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_attuned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}
