use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use std::collections::HashMap;

use crate::client::ApiClient;
use crate::models::{
    app_state::{BuilderState, CharacterCreationStep, Screen},
    character::{Character, CharacterClass, CharacterFeat, CharacterSpell, InventoryItem},
    compendium::{
        Background, Class, ClassFeature, Feat, Item, Race, Spell, SubclassFeature, Subrace,
    },
};
use crate::ui;
use crate::utils::storage::StorageManager;

pub mod ui_state;
pub use ui_state::UiState;

pub mod character;
pub mod equipment;
pub mod events;
pub mod feats;
pub mod inventory;
pub mod levelup;
pub mod multiclass;
pub mod spells;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeaturesSubTab {
    All,
    ClassFeatures,
    SpeciesTraits,
    Feats,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsiMode {
    PlusOneTwo, // +1/+1 to two abilities
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelUpPrompt {
    SubclassChoice { class_id: i32, class_name: String },
    AsiOrFeat { class_name: String },
}

/// Application state.
///
/// Domain / infrastructure fields live directly on `App`.  All TUI
/// navigation, widget, and modal state lives in `app.ui` (`UiState`).
///
/// `App` implements `Deref<Target = UiState>` so that existing call-sites
/// (`app.screen`, `app.sidebar_focused`, …) continue to compile during the
/// transition to explicit `app.ui.*` access.
pub struct App {
    // ── Infrastructure ────────────────────────────────────────────────────────
    pub client: ApiClient,
    pub rt: tokio::runtime::Handle,
    pub storage: StorageManager,
    pub is_offline: bool,

    // ── TUI / navigation state (all fields in here) ───────────────────────────
    pub ui: UiState,

    // ── Character builder ─────────────────────────────────────────────────────
    pub builder: BuilderState,

    // ── Compendium reference data ─────────────────────────────────────────────
    pub classes: Vec<Class>,
    pub races: Vec<Race>,
    pub backgrounds: Vec<Background>,
    pub subraces: Vec<Subrace>,

    // ── Character list ────────────────────────────────────────────────────────
    pub characters: Vec<Character>,

    // ── Active character domain data ──────────────────────────────────────────
    pub active_character: Option<Character>,
    pub char_feats: Vec<CharacterFeat>,
    pub char_weapon_masteries: Vec<String>,
    pub char_spells: Vec<CharacterSpell>,
    /// spell_id → source label ("Spellbook", "Paladin", "Oath of Devotion", …)
    pub spell_sources: HashMap<i32, String>,
    pub char_inventory: Vec<InventoryItem>,
    pub char_proficiencies: Vec<crate::models::CharacterProficiency>,
    pub char_classes: Vec<CharacterClass>,
    pub char_race_name: String,
    pub char_class_name: String,
    /// Caster progression from class data ("full", "1/2", "1/3", …).
    pub char_caster_progression: String,
    pub char_bg_name: String,
    pub active_class_id: i32,
    /// Skill proficiencies from background + class choices.
    pub char_chosen_skills: Vec<String>,
    /// Skills with double proficiency (expertise).
    pub char_expertise_skills: Vec<String>,
    /// Subclass display name (empty if none/unknown).
    pub char_subclass_name: String,
    /// Class features up to the character's current level.
    pub char_class_features: Vec<ClassFeature>,
    /// Subclass features up to the character's current level.
    pub char_subclass_features: Vec<SubclassFeature>,
    /// Race traits as `(name, description)` pairs.
    pub char_race_traits: Vec<(String, String)>,
    /// Aggregated combat actions.
    pub char_actions: Option<crate::models::actions::CharacterActionsResponse>,
    /// Class resources (LOH, Channel Divinity, …).
    pub char_resources: Option<crate::models::ClassResourceResponse>,

    // ── Combat state ──────────────────────────────────────────────────────────
    /// Active conditions (Poisoned, Blinded, …).
    pub conditions: Vec<String>,
    /// spell_id of active concentration spell, `None` if not concentrating.
    pub concentrating_on: Option<i32>,

    // ── Spell slots and hit dice ──────────────────────────────────────────────
    /// Max slots per level from the API, indexed [level-1][slot_idx].
    pub char_spell_slots: Vec<Vec<u8>>,
    pub spell_slots_used: [u8; 9],
    pub hit_dice_used: [u8; 4],

    // ── Compendium data for pickers ───────────────────────────────────────────
    pub all_spells: Vec<Spell>,
    pub all_items: Vec<Item>,
    pub all_feats: Vec<Feat>,

    // ── Derived / cached data ─────────────────────────────────────────────────
    pub cached_actions: Option<Vec<crate::models::actions::ActionEntry>>,
    pub spells_dirty: bool,

    // ── Death saves (synced with API) ─────────────────────────────────────────
    pub death_saves_success: u8,
    pub death_saves_fail: u8,
}

// ── Deref shims (Step 1 transition — remove when call-sites are updated) ─────

impl std::ops::Deref for App {
    type Target = UiState;
    #[inline]
    fn deref(&self) -> &UiState {
        &self.ui
    }
}

impl std::ops::DerefMut for App {
    #[inline]
    fn deref_mut(&mut self) -> &mut UiState {
        &mut self.ui
    }
}

impl App {
    pub fn new(client: ApiClient, rt: tokio::runtime::Handle) -> Self {
        let storage_opt = StorageManager::new();
        let is_offline = storage_opt.is_none();
        let storage = storage_opt.expect("Failed to initialize storage");

        let mut app = Self {
            client,
            rt,
            storage,
            is_offline,

            ui: UiState::default(),

            builder: BuilderState::default(),

            classes: Vec::new(),
            races: Vec::new(),
            backgrounds: Vec::new(),
            subraces: Vec::new(),

            characters: Vec::new(),

            active_character: None,
            char_feats: Vec::new(),
            char_weapon_masteries: Vec::new(),
            char_spells: Vec::new(),
            spell_sources: HashMap::new(),
            char_inventory: Vec::new(),
            char_proficiencies: Vec::new(),
            char_classes: Vec::new(),
            char_race_name: String::new(),
            char_class_name: String::new(),
            char_caster_progression: String::new(),
            char_bg_name: String::new(),
            active_class_id: 0,
            char_chosen_skills: Vec::new(),
            char_expertise_skills: Vec::new(),
            char_subclass_name: String::new(),
            char_class_features: Vec::new(),
            char_subclass_features: Vec::new(),
            char_race_traits: Vec::new(),
            char_actions: None,
            char_resources: None,
            conditions: Vec::new(),
            concentrating_on: None,
            char_spell_slots: Vec::new(),
            spell_slots_used: [0u8; 9],
            hit_dice_used: [0u8; 4],

            all_spells: Vec::new(),
            all_items: Vec::new(),
            all_feats: Vec::new(),

            cached_actions: None,
            spells_dirty: true,

            death_saves_success: 0,
            death_saves_fail: 0,
        };

        if !app.is_offline {
            app.check_saved_session();
        }

        app
    }

    pub fn check_saved_session(&mut self) {
        let session = self.storage.load_session();
        if let Some(token) = session.token {
            self.client.set_token(token);
            // Try to fetch initial data
            self.fetch_compendium_data();
            self.fetch_characters();
            if !self.characters.is_empty() || !self.classes.is_empty() {
                self.screen = Screen::CharacterList;
                self.status_msg = "Session restored.".into();
            }
        }
    }

    pub fn logout(&mut self) {
        self.storage.clear_session();
        self.client.clear_token();
        self.screen = Screen::Login;
        self.status_msg = "Logged out.".into();
    }

    pub fn fetch_compendium_data(&mut self) {
        let rt = self.rt.clone();
        let core = rt.block_on(async {
            tokio::join!(
                self.client.get_classes(None, None),
                self.client.get_races(None, None),
                self.client.get_backgrounds(None, None),
                self.client.get_spells(None, None),
                self.client.get_items(None, None),
                self.client.get_compendium_feats(None),
                self.client.get_subraces(),
            )
        });

        match core {
            (
                Ok(classes),
                Ok(races),
                Ok(backgrounds),
                Ok(spells),
                Ok(items),
                Ok(feats),
                Ok(subraces),
            ) => {
                self.classes = classes.clone();
                self.races = races.clone();
                self.backgrounds = backgrounds.clone();
                self.all_spells = spells.clone();
                self.all_items = items.clone();
                self.all_feats = feats.clone();
                self.subraces = subraces.clone();

                // Save to cache
                let cache = crate::utils::storage::CompendiumCache {
                    classes,
                    races,
                    backgrounds,
                    spells,
                    items,
                    feats,
                    subraces,
                };
                self.storage.save_cache("compendium.json", &cache);
                self.is_offline = false;
            }
            _ => {
                // Fallback to cache
                if let Some(cache) = self
                    .storage
                    .load_cache::<crate::utils::storage::CompendiumCache>("compendium.json")
                {
                    self.classes = cache.classes;
                    self.races = cache.races;
                    self.backgrounds = cache.backgrounds;
                    self.all_spells = cache.spells;
                    self.all_items = cache.items;
                    self.all_feats = cache.feats;
                    self.subraces = cache.subraces;
                    self.status_msg = "Loaded compendium from cache (Offline).".into();
                    self.is_offline = true;
                }
            }
        }
    }

    pub fn fetch_characters(&mut self) {
        let rt = self.rt.clone();
        match rt.block_on(self.client.get_characters()) {
            Ok(chars) => {
                self.characters = chars.clone();
                self.storage.save_cache("characters.json", &chars);
                self.is_offline = false;
                if self.selected_char >= self.characters.len() {
                    self.selected_char = self.characters.len().saturating_sub(1);
                }
                let idx = self.selected_char;
                self.char_list_state.select(Some(idx));
            }
            Err(_) => {
                // Fallback to cache
                if let Some(chars) = self.storage.load_cache::<Vec<Character>>("characters.json") {
                    self.characters = chars;
                    self.is_offline = true;
                    self.status_msg = "Loaded characters from cache (Offline).".into();
                    if self.selected_char >= self.characters.len() {
                        self.selected_char = self.characters.len().saturating_sub(1);
                    }
                    let idx = self.selected_char;
                    self.char_list_state.select(Some(idx));
                }
            }
        }
    }

    // ── Render dispatch ──

    pub fn render(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Login => ui::login::render(self, frame),
            Screen::CharacterList => ui::char_list::render(self, frame),
            Screen::CharacterBuilder => ui::builder::render(self, frame),
            Screen::CharacterSheet => ui::sheet::render(self, frame),
            Screen::EditCharacter => ui::edit_character::render(self, frame),
        }
    }

    // ── Event dispatch ──

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            tracing::debug!("Input event: {key:?}");
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.should_quit = true;
                return;
            }

            match self.screen {
                Screen::Login => crate::app::events::auth::handle_login_key(self, key),
                Screen::CharacterList => {
                    crate::app::events::character_list::handle_char_list_key(self, key)
                }
                Screen::CharacterBuilder => crate::ui::builder::handle_key(self, key),
                Screen::CharacterSheet => crate::app::events::sheet::handle_sheet_key(self, key),
                Screen::EditCharacter => {
                    crate::app::events::edit::handle_edit_character_key(self, key)
                }
            }
        }
    }

    /// Calculate AC from equipped armor + shield in inventory.
    /// Falls back to 10 + DEX mod (unarmored).
    pub fn calc_ac(&self, dex_mod: i32) -> i32 {
        let mut base = 10 + dex_mod; // unarmored default
        let mut shield_bonus = 0i32;

        for inv in self.char_inventory.iter().filter(|i| i.is_equipped) {
            if let Some(item) = self.all_items.iter().find(|i| i.id == inv.item_id) {
                let itype = item
                    .item_type
                    .as_deref()
                    .unwrap_or("")
                    .split('|')
                    .next()
                    .unwrap_or("");

                // Helper to extract AC value from JSON
                let extract_ac = |val: &serde_json::Value| -> i32 {
                    if let Some(n) = val.as_i64() {
                        return n as i32;
                    }
                    if let Some(obj) = val.as_object() {
                        if let Some(ac) = obj.get("ac").and_then(|v| v.as_i64()) {
                            return ac as i32;
                        }
                    }
                    0
                };

                match itype {
                    "LA" => {
                        let ac = item.armor_class.as_ref().map(extract_ac).unwrap_or(11);
                        base = ac + dex_mod;
                    }
                    "MA" => {
                        let ac = item.armor_class.as_ref().map(extract_ac).unwrap_or(13);
                        base = ac + dex_mod.min(2);
                    }
                    "HA" => {
                        let ac = item.armor_class.as_ref().map(extract_ac).unwrap_or(16);
                        base = ac;
                    }
                    "S" => {
                        shield_bonus = 2;
                    }
                    _ => {}
                }
            }
        }
        base + shield_bonus
    }

    /// Derives combat actions from inventory (weapons) and class features.
    /// This ensures weapon attacks are visible even if the API fails or is offline.
    pub fn derive_actions(&self) -> Vec<crate::models::actions::ActionEntry> {
        let mut derived = Vec::new();
        let character = match &self.active_character {
            Some(c) => c,
            None => return derived,
        };

        let str_mod = crate::models::rules::ability_modifier(character.strength);
        let dex_mod = crate::models::rules::ability_modifier(character.dexterity);
        let level = crate::models::rules::level_from_xp(character.experience_pts);
        let prof = crate::models::rules::proficiency_bonus(level);

        // Scan inventory for weapons
        for inv in self.char_inventory.iter().filter(|i| i.is_equipped) {
            if let Some(item) = self.all_items.iter().find(|i| i.id == inv.item_id) {
                let itype_full = item.item_type.as_deref().unwrap_or("");
                let itype = itype_full.split('|').next().unwrap_or("");
                if itype == "M" || itype == "R" {
                    // 'M'elee Weapon, 'R'anged Weapon
                    let is_finesse = item
                        .properties
                        .as_ref()
                        .map(|p| p.iter().any(|s| s.to_lowercase() == "finesse"))
                        .unwrap_or(false);
                    let is_ranged = itype == "R";

                    let ability_mod = if is_ranged || (is_finesse && dex_mod > str_mod) {
                        dex_mod
                    } else {
                        str_mod
                    };

                    let hit_bonus = prof + ability_mod;
                    let damage_val = item
                        .damage
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("1d4");

                    let mut desc = format!(
                        "Proficiency with a {} allows you to add your proficiency bonus to the attack roll for any attack you make with it.",
                        item.name
                    );

                    if let Some(props) = &item.properties {
                        if !props.is_empty() {
                            desc.push_str("\n\nProperties:\n");
                            for prop in props {
                                let code =
                                    crate::utils::weapon_properties::parse_property_code(prop);
                                let prop_name =
                                    crate::utils::weapon_properties::property_name(code);
                                let prop_desc =
                                    crate::utils::weapon_properties::property_description(code);
                                desc.push_str(&format!("{}. {}\n", prop_name, prop_desc));
                            }
                        }
                    }

                    // Always check for mastery info from backend
                    let mut mastery_details = Vec::new();
                    if let Some(masteries) = &item.mastery {
                        for mastery in masteries {
                            let code =
                                crate::utils::weapon_properties::parse_property_code(mastery);
                            let mut name = crate::utils::weapon_mastery::get_mastery_property(code);
                            let mut desc_text =
                                crate::utils::weapon_mastery::get_mastery_description(name);
                            if name == "—" {
                                let desc_direct =
                                    crate::utils::weapon_mastery::get_mastery_description(code);
                                if desc_direct != "No description available." {
                                    name = code;
                                    desc_text = desc_direct;
                                }
                            }
                            if name != "—" {
                                mastery_details.push((name, desc_text));
                            }
                        }
                    }
                    if mastery_details.is_empty() {
                        let name = crate::utils::weapon_mastery::get_mastery_property(&item.name);
                        let desc_text = crate::utils::weapon_mastery::get_mastery_description(name);
                        if name != "—" {
                            mastery_details.push((name, desc_text));
                        }
                    }
                    if !mastery_details.is_empty() {
                        desc.push_str("\nMastery:\n");
                        for (name, d) in mastery_details {
                            desc.push_str(&format!("{}: {}\n", name, d));
                        }
                    }

                    derived.push(crate::models::actions::ActionEntry {
                        name: item.name.clone(),
                        source: Some("Inventory".into()),
                        description: Some(desc),
                        range: Some(if is_ranged {
                            "80/320".into()
                        } else {
                            "5 ft".into()
                        }),
                        hit_bonus: Some(format!("{:+}", hit_bonus)),
                        damage: Some(format!("{} {:+}", damage_val, ability_mod)),
                        max_uses: None,
                        current_uses: None,
                        reset_type: None,
                        time: None,
                    });
                }
            }
        }

        // Add default Unarmed Strike
        let unarmed_desc = "Instead of using a weapon to make a melee attack, you can use a punch, kick, head-butt, or similar forceful blow.\n\n\
            Damage. You make an attack roll against the target. On a hit, the target takes Bludgeoning damage equal to 1 plus your Strength modifier.\n\n\
            Grapple. The target must succeed on a Strength or Dexterity saving throw (it chooses which), or it has the Grappled condition. The DC equals 8 + your Strength modifier and Proficiency Bonus.\n\n\
            Shove. The target must succeed on a Strength or Dexterity saving throw (it chooses which), or you either push it 5 feet away or cause it to have the Prone condition. The DC equals 8 + your Strength modifier and Proficiency Bonus.";

        derived.push(crate::models::actions::ActionEntry {
            name: "Unarmed Strike".into(),
            source: Some("Rules".into()),
            description: Some(unarmed_desc.into()),
            range: Some("5 ft".into()),
            hit_bonus: Some(format!("{:+}", prof + str_mod)),
            damage: Some(format!("{}", 1 + str_mod)),
            max_uses: None,
            current_uses: None,
            reset_type: None,
            time: None,
        });

        // Add Paladin resources (Lay on Hands, Channel Divinity)
        if self.char_class_name.eq_ignore_ascii_case("paladin") {
            // Lay on Hands
            let loh_max = self
                .char_resources
                .as_ref()
                .and_then(|r| r.lay_on_hands_pool)
                .unwrap_or_else(|| level * 5);

            let loh_desc = self
                .char_class_features
                .iter()
                .find(|f| f.name.eq_ignore_ascii_case("lay on hands"))
                .and_then(|f| f.entries.as_ref())
                .map(|e| crate::models::compendium::json_array_to_text(e))
                .unwrap_or_else(|| "As an action, you can touch a creature and draw power from the pool to restore a number of hit points to that creature, up to the maximum amount remaining in your pool.".into());

            derived.push(crate::models::actions::ActionEntry {
                name: "Lay on Hands".into(),
                source: Some("Paladin".into()),
                description: Some(loh_desc),
                range: Some("Touch".into()),
                hit_bonus: None,
                damage: None,
                max_uses: Some(loh_max),
                current_uses: None,
                reset_type: Some("Long Rest".into()),
                time: Some(serde_json::json!([{"number": 1, "unit": "action"}])),
            });

            // Channel Divinity
            let cd_max = if let Some(r) = &self.char_resources {
                r.channel_divinity_uses
            } else {
                // Fallback: level-based logic (class_table extraction is unreliable with 5etools format)
                Some(if level >= 18 {
                    3
                } else if level >= 7 {
                    2
                } else {
                    1
                })
            };

            if let Some(max) = cd_max {
                let cd_desc = self
                    .char_class_features
                    .iter()
                    .find(|f| f.name.eq_ignore_ascii_case("channel divinity"))
                    .and_then(|f| f.entries.as_ref())
                    .map(|e| crate::models::compendium::json_array_to_text(e))
                    .unwrap_or_else(|| {
                        "You can use your Channel Divinity to create various effects.".into()
                    });

                derived.push(crate::models::actions::ActionEntry {
                    name: "Channel Divinity".into(),
                    source: Some("Paladin".into()),
                    description: Some(cd_desc),
                    range: None,
                    hit_bonus: None,
                    damage: None,
                    max_uses: Some(max),
                    current_uses: None,
                    reset_type: Some("Short or Long Rest".into()),
                    time: None,
                });
            }
        }

        derived
    }

    /// Returns the max spell slots for a given slot index (0=1st level), preferring
    /// the API-provided `char_spell_slots` over the hardcoded table.
    pub fn spell_slots_max_for_slot(&self, slot_idx: usize) -> u8 {
        let level = self
            .active_character
            .as_ref()
            .map(|c| crate::models::rules::level_from_xp(c.experience_pts))
            .unwrap_or(1);

        if !self.char_spell_slots.is_empty() {
            self.char_spell_slots
                .get((level as usize).saturating_sub(1))
                .and_then(|row| row.get(slot_idx))
                .copied()
                .unwrap_or(0)
        } else {
            crate::models::rules::spell_slots_max(&self.char_caster_progression, level, slot_idx)
        }
    }

    /// The ability that governs spellcasting for the character's class.
    pub fn spellcasting_ability(&self) -> Option<&'static str> {
        // First try to look up from the actual class data
        if let Some(class) = self.classes.iter().find(|cl| cl.id == self.active_class_id) {
            if let Some(ability) = &class.spellcasting_ability {
                return Some(match ability.to_lowercase().as_str() {
                    "strength" => "str",
                    "dexterity" => "dex",
                    "constitution" => "con",
                    "intelligence" => "int",
                    "wisdom" => "wis",
                    "charisma" => "cha",
                    _ => "cha",
                });
            }
        }
        // Fallback based on class name
        match self.char_class_name.to_lowercase().as_str() {
            "wizard" => Some("int"),
            "sorcerer" | "bard" | "warlock" | "paladin" => Some("cha"),
            "cleric" | "druid" | "ranger" => Some("wis"),
            _ => None,
        }
    }

    pub fn spell_save_dc(&self) -> Option<i32> {
        let ability = self.spellcasting_ability()?;
        let character = self.active_character.as_ref()?;
        let score = crate::models::rules::ch_ability_score(character, ability);
        let modifier = crate::models::rules::ability_modifier(score);
        let level = crate::models::rules::level_from_xp(character.experience_pts);
        let prof = crate::models::rules::proficiency_bonus(level);
        Some(8 + prof + modifier)
    }

    pub fn spell_attack_bonus(&self) -> Option<i32> {
        let ability = self.spellcasting_ability()?;
        let character = self.active_character.as_ref()?;
        let score = crate::models::rules::ch_ability_score(character, ability);
        let modifier = crate::models::rules::ability_modifier(score);
        let level = crate::models::rules::level_from_xp(character.experience_pts);
        let prof = crate::models::rules::proficiency_bonus(level);
        Some(prof + modifier)
    }

    /// Returns a list of (Class Name, Ability Mod, Attack Bonus, Save DC) for all classes.
    pub fn multiclass_spell_stats(&self) -> Vec<(String, i32, i32, i32)> {
        let character = match &self.active_character {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        // For each class entry in char_classes
        for cc in &self.char_classes {
            // Find class name and spellcasting ability from compendium
            if let Some(class_data) = self.classes.iter().find(|c| c.id == cc.class_id) {
                if let Some(ability_name) = &class_data.spellcasting_ability {
                    let ability_key = match ability_name.to_lowercase().as_str() {
                        "strength" => "str",
                        "dexterity" => "dex",
                        "constitution" => "con",
                        "intelligence" => "int",
                        "wisdom" => "wis",
                        "charisma" => "cha",
                        _ => "cha",
                    };
                    let score = crate::models::rules::ch_ability_score(character, ability_key);
                    let modifier = crate::models::rules::ability_modifier(score);
                    let char_level = crate::models::rules::level_from_xp(character.experience_pts);
                    let prof = crate::models::rules::proficiency_bonus(char_level);

                    results.push((
                        class_data.name.clone(),
                        modifier,
                        prof + modifier,
                        8 + prof + modifier,
                    ));
                }
            }
        }

        // If empty, fall back to single-class method
        if results.is_empty() {
            if let (Some(atk), Some(dc)) = (self.spell_attack_bonus(), self.spell_save_dc()) {
                let ability = self.spellcasting_ability().unwrap_or("");
                let score = crate::models::rules::ch_ability_score(character, ability);
                let modifier = crate::models::rules::ability_modifier(score);
                results.push((self.char_class_name.clone(), modifier, atk, dc));
            }
        }

        results
    }

    /// Extract walk speed from race's `speed` JsonValue.
    pub fn race_speed(&self) -> i32 {
        let race = self
            .active_character
            .as_ref()
            .and_then(|c| c.race_id)
            .and_then(|rid| self.races.iter().find(|r| r.id == rid));

        if let Some(r) = race {
            // Speed can be a plain number or {"walk": 30, "fly": 50, ...}
            if let Some(n) = r.speed.as_i64() {
                return n as i32;
            }
            if let Some(obj) = r.speed.as_object() {
                if let Some(walk) = obj.get("walk").and_then(|v| v.as_i64()) {
                    return walk as i32;
                }
            }
        }
        30 // fallback
    }

    /// Helper to resolve feat / item / spell name
    pub fn feat_name(&self, feat_id: i32) -> String {
        self.all_feats
            .iter()
            .find(|f| f.id == feat_id)
            .map(|f| f.name.clone())
            .unwrap_or_else(|| format!("Feat #{feat_id}"))
    }

    pub fn item_name(&self, item_id: i32) -> String {
        self.all_items
            .iter()
            .find(|i| i.id == item_id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| format!("Item #{item_id}"))
    }

    pub fn spell_name(&self, spell_id: i32) -> String {
        self.all_spells
            .iter()
            .find(|s| s.id == spell_id)
            .map(|s| {
                let source = crate::models::compendium::source_id_label(s.source_id);
                if source == "Other" {
                    s.name.clone()
                } else {
                    format!("{} {}", s.name, source)
                }
            })
            .unwrap_or_else(|| format!("Spell #{spell_id}"))
    }

    /// Ensures any "always prepared" spells from class/subclass additional_spells (API data)
    /// are present in the character's spell list and marked as prepared.
    /// Falls back to interpreting class features text for older editions.
    pub fn sync_always_prepared_spells(&mut self) {
        let char_level = self
            .active_character
            .as_ref()
            .map(|c| crate::models::rules::level_from_xp(c.experience_pts))
            .unwrap_or(1);

        // Collect (spell_id, source_name) pairs
        let mut to_add: Vec<(i32, String)> = Vec::new();

        // Parse class and subclass additional_spells from the API response
        if let Some(detail) = &self.class_detail.clone() {
            // Class-level additional_spells (e.g. XPHB Paladin: Divine Smite at 2, Find Steed at 5)
            if let Some(additional) = &detail.class.additional_spells {
                Self::collect_additional_spells(
                    additional,
                    char_level,
                    &self.all_spells,
                    &detail.class.name,
                    &mut to_add,
                );
            }

            // Subclass-level additional_spells (oath/domain/circle spells)
            // Only include spells from the character's chosen subclass
            let subclass_name_lower = self.char_subclass_name.to_lowercase();
            for swf in &detail.subclasses {
                let matches = !subclass_name_lower.is_empty()
                    && (swf.subclass.name.to_lowercase() == subclass_name_lower
                        || swf.subclass.short_name.to_lowercase() == subclass_name_lower);
                if matches {
                    // PHB format: subclass.additional_spells JSON
                    if let Some(additional) = &swf.subclass.additional_spells {
                        Self::collect_additional_spells(
                            additional,
                            char_level,
                            &self.all_spells,
                            &swf.subclass.name,
                            &mut to_add,
                        );
                    }
                    // XPHB format: subclass features with spell tables (e.g. "Oath of Glory Spells")
                    Self::collect_spells_from_subclass_features(
                        &swf.features,
                        char_level,
                        &self.all_spells,
                        &swf.subclass.name,
                        &mut to_add,
                    );
                }
            }
        }

        // Also check class features text for "always have the X spell prepared" (legacy fallback)
        for feature in &self.char_class_features {
            if let crate::models::features::Feature::GrantsSpell { spell_name } =
                feature.interpret()
            {
                let target_name = spell_name.to_lowercase();
                if let Some(spell) = self.all_spells.iter().find(|s| {
                    let s_name = s.name.to_lowercase();
                    s_name == target_name || s_name.starts_with(&format!("{} ", target_name))
                }) {
                    if !to_add.iter().any(|(id, _)| *id == spell.id) {
                        to_add.push((spell.id, self.char_class_name.clone()));
                    }
                }
            }
        }

        // Apply: add missing spells and mark existing as prepared
        for (spell_id, source_name) in &to_add {
            if !self.char_spells.iter().any(|cs| cs.spell_id == *spell_id) {
                let character_id = self.active_character.as_ref().map(|c| c.id);
                if let Some(cid) = character_id {
                    let rt = self.rt.clone();
                    let req = crate::models::character::AddSpellRequest {
                        spell_id: *spell_id,
                        is_prepared: Some(true),
                    };
                    match rt.block_on(self.client.add_spell(cid, &req)) {
                        Ok(cs) => self.char_spells.push(cs),
                        Err(_) => {
                            self.char_spells
                                .push(crate::models::character::CharacterSpell {
                                    character_id: cid,
                                    spell_id: *spell_id,
                                    is_prepared: true,
                                });
                        }
                    }
                }
                self.spell_sources.insert(*spell_id, source_name.clone());
            } else if let Some(cs) = self
                .char_spells
                .iter_mut()
                .find(|cs| cs.spell_id == *spell_id)
            {
                cs.is_prepared = true;
                self.spell_sources.insert(*spell_id, source_name.clone());
            }
        }

        // Persist subclass name to cache if we have one
        if !self.char_subclass_name.is_empty() {
            self.persist_subclass_to_cache();
        }
    }

    /// Parse additional_spells JSON (from class or subclass) and collect (spell_id, source_name)
    /// pairs of spells that should be always prepared/known at the given character level.
    ///
    /// additional_spells format:
    /// ```json
    /// [{"prepared": {"2": ["divine smite|xphb"], "5": ["find steed|xphb"]}}]
    /// ```
    fn collect_additional_spells(
        additional: &serde_json::Value,
        char_level: i32,
        all_spells: &[crate::models::compendium::Spell],
        source_name: &str,
        out: &mut Vec<(i32, String)>,
    ) {
        let arr = match additional.as_array() {
            Some(a) => a,
            None => return,
        };
        for entry in arr {
            let obj = match entry.as_object() {
                Some(o) => o,
                None => continue,
            };
            for (_key, level_map) in obj {
                let level_map_obj = match level_map.as_object() {
                    Some(o) => o,
                    None => continue,
                };
                for (level_str, spell_names) in level_map_obj {
                    let required_level: i32 = match level_str.parse::<i32>() {
                        Ok(l) => l,
                        Err(_) => continue,
                    };
                    if required_level > char_level {
                        continue;
                    }
                    let names_arr = match spell_names.as_array() {
                        Some(a) => a,
                        None => continue,
                    };
                    for name_val in names_arr {
                        let raw_name = match name_val.as_str() {
                            Some(s) => s,
                            None => continue,
                        };
                        let spell_name = raw_name.split('|').next().unwrap_or(raw_name).trim();
                        if let Some(spell) = all_spells
                            .iter()
                            .find(|s| s.name.eq_ignore_ascii_case(spell_name))
                        {
                            if !out.iter().any(|(id, _)| *id == spell.id) {
                                out.push((spell.id, source_name.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Parse subclass features for spell-granting entries (XPHB 2024 format).
    /// Looks for features named like "{Subclass} Spells" that contain a table of spells.
    /// Handles nested entry structures and both `,` and `•` (U+2022) as spell separators.
    fn collect_spells_from_subclass_features(
        features: &[crate::models::compendium::SubclassFeature],
        char_level: i32,
        all_spells: &[crate::models::compendium::Spell],
        source_name: &str,
        out: &mut Vec<(i32, String)>,
    ) {
        for feature in features {
            let name_lower = feature.name.to_lowercase();

            // Skip features that don't grant spells (e.g. flavor text, abilities)
            if !name_lower.contains("spells")
                && !name_lower.contains("domain")
                && !name_lower.contains("circle")
            {
                continue;
            }

            let entries = match &feature.entries {
                Some(e) => e,
                None => continue,
            };

            // Flatten all nested entries to find tables at any depth
            let mut tables: Vec<Vec<(String, String)>> = Vec::new();
            Self::collect_tables_from_entries(entries, &mut tables);

            for rows in &tables {
                for (level_str, spells_str) in rows {
                    let level_num = level_str
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<i32>()
                        .unwrap_or(0);
                    if level_num == 0 || level_num > char_level {
                        continue;
                    }
                    Self::add_spell_names(spells_str, all_spells, source_name, out);
                }
            }
        }
    }

    /// Recursively walk JSON entries and collect all (level, spells) pairs from tables.
    fn collect_tables_from_entries(
        entries: &[serde_json::Value],
        out: &mut Vec<Vec<(String, String)>>,
    ) {
        for entry in entries {
            match entry {
                serde_json::Value::Object(obj) => {
                    if obj.get("type").and_then(|t| t.as_str()) == Some("table") {
                        if let Some(rows) = obj.get("rows").and_then(|r| r.as_array()) {
                            let mut pairs = Vec::new();
                            for row in rows {
                                if let Some(arr) = row.as_array() {
                                    if arr.len() >= 2 {
                                        let level_s = arr[0].as_str().unwrap_or("");
                                        let spells_val = &arr[1];
                                        let spells_s = match spells_val {
                                            serde_json::Value::String(s) => s.clone(),
                                            serde_json::Value::Array(a) => a
                                                .iter()
                                                .filter_map(|v| v.as_str())
                                                .collect::<Vec<_>>()
                                                .join("•"),
                                            _ => String::new(),
                                        };
                                        if !level_s.is_empty() && !spells_s.is_empty() {
                                            pairs.push((level_s.to_string(), spells_s));
                                        }
                                    }
                                }
                            }
                            if !pairs.is_empty() {
                                out.push(pairs);
                            }
                        }
                    }
                    // Recurse into sub-entries (5etools often nests: entries → entries → table)
                    if let Some(sub) = obj.get("entries") {
                        if let Some(arr) = sub.as_array() {
                            Self::collect_tables_from_entries(arr, out);
                        }
                    }
                    // Also check for items (type: "list" often has refs to spells)
                    if let Some(items) = obj.get("items") {
                        if let Some(arr) = items.as_array() {
                            Self::collect_tables_from_entries(arr, out);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Split a spell-name string by `,` or `•` (U+2022) and add matching spells.
    fn add_spell_names(
        raw: &str,
        all_spells: &[crate::models::compendium::Spell],
        source_name: &str,
        out: &mut Vec<(i32, String)>,
    ) {
        for part in raw.split(|c| c == ',' || c == '•') {
            let spell_name = part.trim().split('|').next().unwrap_or(part.trim()).trim();
            if spell_name.is_empty() {
                continue;
            }
            if let Some(spell) = all_spells
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(spell_name))
            {
                if !out.iter().any(|(id, _)| *id == spell.id) {
                    out.push((spell.id, source_name.to_string()));
                }
            }
        }
    }

    pub fn always_prepared_spell_ids(&self) -> Vec<i32> {
        self.spell_sources
            .iter()
            .filter(|(_, src)| src.as_str() != "Spellbook")
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn spellcasting_classes(&self) -> Vec<(String, i32, i32, i32)> {
        self.multiclass_spell_stats()
    }

    pub fn filtered_mastery_weapons(&self) -> Vec<&Item> {
        let search = if self.screen == Screen::CharacterBuilder {
            self.builder.feat_picker_search.to_lowercase()
        } else {
            self.picker_search.to_lowercase()
        };

        self.all_items
            .iter()
            .filter(|i| {
                // Check if weapon is in hardcoded mastery list
                if !crate::utils::weapon_mastery::has_mastery(&i.name) {
                    return false;
                }

                // Apply search filter if any
                if !search.is_empty() {
                    return i.name.to_lowercase().contains(&search);
                }
                true
            })
            .collect()
    }

    pub fn toggle_proficiency(&mut self, category: &str, name: &str) {
        let char_id = match self.active_character.as_ref().map(|c| c.id) {
            Some(id) => id,
            None => return,
        };

        // Find existing
        let existing = self
            .char_proficiencies
            .iter()
            .find(|p| p.category == category && p.name == name)
            .cloned();

        let rt = self.rt.clone();
        match existing {
            None => {
                // Add "proficiency"
                let req = crate::models::AddProficiencyRequest {
                    category: category.to_string(),
                    name: name.to_string(),
                    proficiency_type: "proficiency".to_string(),
                };
                if let Ok(new_prof) = rt.block_on(self.client.add_proficiency(char_id, &req)) {
                    self.char_proficiencies.push(new_prof);
                }
            }
            Some(prof) if prof.proficiency_type == "proficiency" => {
                // Patch to "expertise"
                let req = crate::models::PatchProficiencyRequest {
                    proficiency_type: "expertise".to_string(),
                };
                if let Ok(updated) =
                    rt.block_on(self.client.patch_proficiency(char_id, prof.id, &req))
                {
                    if let Some(p) = self.char_proficiencies.iter_mut().find(|p| p.id == prof.id) {
                        p.proficiency_type = updated.proficiency_type;
                    }
                }
            }
            Some(prof) => {
                // Delete (back to None)
                if rt
                    .block_on(self.client.delete_proficiency(char_id, prof.id))
                    .is_ok()
                {
                    self.char_proficiencies.retain(|p| p.id != prof.id);
                }
            }
        }
    }

    /// Re-calculates derived actions (LOH, Channel Divinity, Weapons) and merges them
    /// into the current character action state.
    pub fn refresh_derived_actions(&mut self) {
        let derived = self.derive_actions();
        if let Some(ref mut actions) = self.char_actions {
            for la in derived {
                if la.max_uses.is_some() {
                    if let Some(existing) =
                        actions.limited_use.iter_mut().find(|a| a.name == la.name)
                    {
                        existing.max_uses = la.max_uses;
                        existing.description = la.description.clone();
                    } else {
                        actions.limited_use.push(la.clone());
                    }
                }
                if let Some(existing) = actions.all.iter_mut().find(|a| a.name == la.name) {
                    existing.max_uses = la.max_uses;
                    existing.description = la.description.clone();
                } else {
                    actions.all.push(la.clone());
                }
                if (la.hit_bonus.is_some() || la.damage.is_some())
                    && !actions.attack.iter().any(|a| a.name == la.name)
                {
                    actions.attack.push(la);
                }
            }
        }
    }

    pub fn is_online(&mut self) -> bool {
        let rt = self.rt.clone();
        let online = rt.block_on(self.client.check_health());
        self.is_offline = !online;
        online
    }

    pub fn save_draft(&mut self) -> bool {
        let draft = crate::models::CharacterDraft {
            current_step: match self.builder.step {
                CharacterCreationStep::Class => 1,
                CharacterCreationStep::Background => 2,
                CharacterCreationStep::Species => 3,
                CharacterCreationStep::Abilities => 4,
                CharacterCreationStep::Equipment => 5,
            },
            class_id: self.builder.class_id,
            level: self.builder.level,
            subclass_id: self.builder.subclass_id,
            name: self.builder.name.clone(),
            personality: self.builder.trait_text.clone(),
            background_id: self.builder.bg_id,
            background_feat_id: self.builder.background_feat_id,
            species_id: self.builder.race_id,
            lineage_id: self.builder.lineage_id,
            abilities: self.builder.abilities,
            equipment_option: self.builder.equipment_option,
        };

        let draft_json = serde_json::to_string(&draft).unwrap_or_default();
        let draft_name = if self.builder.name.trim().is_empty() {
            "[DRAFT] Untitled".to_string()
        } else {
            format!("[DRAFT] {}", self.builder.name.trim())
        };

        let rt = self.rt.clone();
        let client = self.client.clone();
        let draft_id = self.builder.draft_id;

        let class_id = self
            .builder
            .class_id
            .unwrap_or_else(|| self.classes.first().map(|c| c.id).unwrap_or(1));

        let n = draft.current_step;

        if !self.is_online() {
            self.status_msg = "Offline! Cannot save draft to server.".to_string();
            return false;
        }

        if let Some(id) = draft_id {
            let req = crate::models::UpdateCharacterRequest {
                name: draft_name,
                class_id,
                strength: self.builder.abilities[0],
                dexterity: self.builder.abilities[1],
                constitution: self.builder.abilities[2],
                intelligence: self.builder.abilities[3],
                wisdom: self.builder.abilities[4],
                charisma: self.builder.abilities[5],
                max_hp: 10,
                // Ensure required runtime fields are present for server validation
                current_hp: Some(10),
                temp_hp: Some(0),
                inspiration: Some(false),
                notes: Some(draft_json.clone()),
                // Server expects experience_pts present for PUT — use 0 for drafts
                experience_pts: Some(0),
                ..Default::default()
            };
            // Log payload for debugging
            if let Ok(payload) = serde_json::to_string_pretty(&req) {
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("draft_payload.log")
                    .and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "UPDATE /characters/{} => {}\n", id, payload)
                    });
            }
            tracing::info!("Attempting sync for Step {n}...");
            match rt.block_on(client.update_character(id, &req)) {
                Ok(_) => {
                    tracing::info!("Sync Successful for Character {id}");
                    self.status_msg = "Draft auto-saved.".to_string();
                    true
                }
                Err(e) => {
                    let (status_code, error_text) = match &e {
                        crate::client::ApiError::Api { status, message } => {
                            (*status, message.clone())
                        }
                        crate::client::ApiError::Request(err) => (
                            err.status().map(|s| s.as_u16()).unwrap_or(0),
                            err.to_string(),
                        ),
                        crate::client::ApiError::Parse(err) => (0, err.clone()),
                    };
                    tracing::error!("Sync Failed: {status_code} - {error_text}");
                    self.status_msg = format!("Failed to auto-save draft: {e}");
                    false
                }
            }
        } else {
            let req = crate::models::CreateCharacterRequest {
                name: draft_name,
                class_id,
                race_id: self.builder.race_id,
                subrace_id: self.builder.lineage_id,
                background_id: self.builder.bg_id,
                strength: self.builder.abilities[0],
                dexterity: self.builder.abilities[1],
                constitution: self.builder.abilities[2],
                intelligence: self.builder.abilities[3],
                wisdom: self.builder.abilities[4],
                charisma: self.builder.abilities[5],
                max_hp: 10,
                bonus_feat_id: None,
                background_feat_id: self.builder.background_feat_id,
            };
            // Log create payload for debugging
            if let Ok(payload) = serde_json::to_string_pretty(&req) {
                let _ = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("draft_payload.log")
                    .and_then(|mut f| {
                        use std::io::Write;
                        writeln!(f, "CREATE /characters => {}\n", payload)
                    });
            }
            tracing::info!("Attempting sync for Step {n}...");
            match rt.block_on(client.create_character(&req)) {
                Ok(character) => {
                    tracing::info!("Sync Successful for Character {}", character.id);
                    self.builder.draft_id = Some(character.id);
                    let update_req = crate::models::UpdateCharacterRequest {
                        name: character.name.clone(),
                        class_id,
                        strength: character.strength,
                        dexterity: character.dexterity,
                        constitution: character.constitution,
                        intelligence: character.intelligence,
                        wisdom: character.wisdom,
                        charisma: character.charisma,
                        max_hp: character.max_hp,
                        // include present runtime fields from created character
                        current_hp: Some(character.current_hp),
                        temp_hp: Some(character.temp_hp),
                        inspiration: Some(character.inspiration),
                        notes: Some(draft_json),
                        // include experience pts from created character
                        experience_pts: Some(character.experience_pts),
                        ..Default::default()
                    };
                    // Log update payload for debugging
                    if let Ok(payload) = serde_json::to_string_pretty(&update_req) {
                        let _ = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("draft_payload.log")
                            .and_then(|mut f| {
                                use std::io::Write;
                                writeln!(f, "UPDATE /characters/{} => {}\n", character.id, payload)
                            });
                    }
                    let _ = rt.block_on(client.update_character(character.id, &update_req));
                    self.status_msg = "Draft created and saved.".to_string();
                    true
                }
                Err(e) => {
                    let (status_code, error_text) = match &e {
                        crate::client::ApiError::Api { status, message } => {
                            (*status, message.clone())
                        }
                        crate::client::ApiError::Request(err) => (
                            err.status().map(|s| s.as_u16()).unwrap_or(0),
                            err.to_string(),
                        ),
                        crate::client::ApiError::Parse(err) => (0, err.clone()),
                    };
                    tracing::error!("Sync Failed: {status_code} - {error_text}");
                    self.status_msg = format!("Failed to create draft: {e}");
                    false
                }
            }
        }
    }
}
