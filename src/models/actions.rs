use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    pub name: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub range: Option<String>,
    pub hit_bonus: Option<String>,
    pub damage: Option<String>,
    pub max_uses: Option<i32>,
    pub current_uses: Option<i32>,
    pub reset_type: Option<String>,
    pub time: Option<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterActionsResponse {
    pub all: Vec<ActionEntry>,
    pub attack: Vec<ActionEntry>,
    pub action: Vec<ActionEntry>,
    pub bonus_action: Vec<ActionEntry>,
    pub reaction: Vec<ActionEntry>,
    pub other: Vec<ActionEntry>,
    pub limited_use: Vec<ActionEntry>,
}
