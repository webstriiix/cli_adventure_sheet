use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::character::{Character, CharacterFeat, CharacterSpell, InventoryItem, CharacterSpellSlot, CharacterHitDice};
use crate::models::compendium::{Background, Class, ClassDetailResponse, Feat, Item, Race, Spell};
use crate::models::actions::CharacterActionsResponse;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FullCharacterCache {
    pub character: Character,
    pub feats: Vec<CharacterFeat>,
    pub spells: Vec<CharacterSpell>,
    pub inventory: Vec<InventoryItem>,
    pub spell_slots: Vec<CharacterSpellSlot>,
    pub hit_dice: Vec<CharacterHitDice>,
    pub class_detail: Option<ClassDetailResponse>,
    pub actions: Option<CharacterActionsResponse>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CompendiumCache {
    pub classes: Vec<Class>,
    pub races: Vec<Race>,
    pub backgrounds: Vec<Background>,
    pub spells: Vec<Spell>,
    pub items: Vec<Item>,
    pub feats: Vec<Feat>,
}

pub struct StorageManager {
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Option<Self> {
        let proj_dirs = ProjectDirs::from("com", "webstriix", "cli-adventure-sheet")?;
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let cache_dir = proj_dirs.cache_dir().to_path_buf();

        // Ensure directories exist
        let _ = fs::create_dir_all(&config_dir);
        let _ = fs::create_dir_all(&cache_dir);

        Some(Self {
            config_dir,
            cache_dir,
        })
    }

    // ── Session (Auth Token) ──

    fn session_path(&self) -> PathBuf {
        self.config_dir.join("session.json")
    }

    pub fn load_session(&self) -> Session {
        let path = self.session_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Session::default()
        }
    }

    pub fn save_session(&self, session: &Session) {
        let path = self.session_path();
        if let Ok(content) = serde_json::to_string_pretty(session) {
            let _ = fs::write(path, content);
        }
    }

    pub fn clear_session(&self) {
        let _ = fs::remove_file(self.session_path()).ok();
    }

    // ── Cache (Characters & Compendium) ──

    pub fn cache_path(&self, filename: &str) -> PathBuf {
        self.cache_dir.join(filename)
    }

    pub fn save_cache<T: Serialize>(&self, filename: &str, data: &T) {
        let path = self.cache_path(filename);
        if let Ok(content) = serde_json::to_string_pretty(data) {
            let _ = fs::write(path, content);
        }
    }

    pub fn load_cache<T: for<'de> Deserialize<'de>>(&self, filename: &str) -> Option<T> {
        let path = self.cache_path(filename);
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
}
