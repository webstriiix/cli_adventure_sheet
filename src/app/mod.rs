use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{Frame, widgets::ListState};
use uuid::Uuid;

use crate::client::ApiClient;
use crate::models::{
    app_state::{
        ActionsSubTab, AuthMode, BuilderState, EditSection,
        MulticlassSection, PickerMode, Screen, SheetTab,
    },
    character::{
        Character, CharacterClass, CharacterFeat, CharacterSpell, InventoryItem,
    },
    compendium::{
        Background, Class, ClassDetailResponse, ClassFeature, Feat, Item, Race, Spell,
    },
};
use crate::ui;
use crate::utils::storage::StorageManager;

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
    PlusTwo,
    PlusOneTwo,
    PlusOneThree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelUpPrompt {
    SubclassChoice { class_id: i32, class_name: String },
    AsiOrFeat { class_name: String },
}

pub mod character;

pub mod equipment;
pub mod feats;
pub mod inventory;
pub mod levelup;
pub mod multiclass;
pub mod spells;

pub struct App {
    pub client: ApiClient,
    pub rt: tokio::runtime::Handle,
    pub storage: StorageManager,
    pub is_offline: bool,
    pub screen: Screen,
    pub should_quit: bool,
    pub status_msg: String,

    // Auth
    pub auth_mode: AuthMode,
    pub auth_fields: [String; 3],
    pub auth_focus: usize,
    pub password_visible: bool,

    // Compendium data
    pub classes: Vec<Class>,
    pub races: Vec<Race>,
    pub backgrounds: Vec<Background>,

    // Character list
    pub characters: Vec<Character>,
    pub selected_char: usize,
    pub char_list_state: ListState,

    // Character creation builder state
    pub builder: BuilderState,

    // Character sheet
    pub active_character: Option<Character>,
    pub sheet_tab: SheetTab,
    pub sheet_tab_index: usize,
    pub actions_sub_tab: ActionsSubTab,
    pub sidebar_focused: bool,
    pub content_scroll: usize,
    pub char_feats: Vec<CharacterFeat>,
    pub char_weapon_masteries: Vec<String>,
    pub char_spells: Vec<CharacterSpell>,
    pub char_inventory: Vec<InventoryItem>,
    pub char_classes: Vec<CharacterClass>, // multiclass entries
    pub char_race_name: String,
    pub char_class_name: String,
    pub char_caster_progression: String, // cached caster_progression from class data (e.g. "full", "1/2", "1/3")
    pub char_bg_name: String,
    pub active_class_id: i32, // cached class_id for the loaded character
    pub char_chosen_skills: Vec<String>, // skill proficiencies from background + class choices
    pub char_expertise_skills: Vec<String>, // skills with double proficiency (expertise)
    pub char_subclass_name: String, // cached subclass name (empty if none/unknown)
    /// Class features up to the character's current level (from class detail API).
    pub char_class_features: Vec<ClassFeature>,
    /// Race traits parsed from race.entries as (name, description) pairs.
    pub char_race_traits: Vec<(String, String)>,
    /// Aggregated combat actions.
    pub char_actions: Option<crate::models::actions::CharacterActionsResponse>,
    /// Selection state for the Limited Use sub-tab list.
    pub actions_list_state: ListState,
    /// Open action detail modal: (feature name, description). None = closed.
    pub actions_detail_modal: Option<(String, String)>,
    /// Open spell detail modal: (spell name, description). None = closed.
    pub spell_detail_modal: Option<(String, String)>,

    // Combat state
    pub conditions: Vec<String>, // active conditions (Poisoned, Blinded, etc.)
    pub concentrating_on: Option<i32>, // spell_id of concentration spell, None if not concentrating

    // Spell slots and hit dice tracking
    pub spell_slots_used: [u8; 9],
    pub spell_level_filter: Option<i32>, // None = All, Some(0) = cantrips, Some(1-9) = spell levels
    pub spell_level_tab_index: usize,    // 0=All, 1=0(cantrips), 2=1st, ..., 6=5th
    pub hit_dice_used: [u8; 4],          // Index 0: d6, 1: d8, 2: d10, 3: d12

    // Compendium data for pickers
    pub all_spells: Vec<Spell>,
    pub all_items: Vec<Item>,
    pub all_feats: Vec<Feat>,

    // Interactive mode state
    pub editing_notes: bool,
    pub notes_buffer: String,
    pub notes_cursor: usize, // byte offset of cursor in notes_buffer
    // Sub-views specifically for the Features tab
    pub features_sub_tab: FeaturesSubTab,

    pub picker_mode: PickerMode,
    pub picker_search: String,
    pub picker_selected: usize,
    pub picker_list_state: ratatui::widgets::ListState,
    pub selected_list_index: usize,
    pub sheet_table_state: ratatui::widgets::TableState,
    pub show_compendium_detail: bool,
    pub show_item_detail: bool,

    // Death saves — synced with API
    pub death_saves_success: u8, // 0–3
    pub death_saves_fail: u8,    // 0–3

    // ASI / Feat choice state
    pub asi_choice_index: usize, // 0 = ability A, 1 = ability B (for +1/+1), 2 = confirm
    pub asi_ability_a: usize,    // index into ABILITY_NAMES
    pub asi_ability_b: usize,    // index into ABILITY_NAMES (for +1/+1 mode)
    pub asi_ability_c: usize,    // index into ABILITY_NAMES (for +1/+1/+1 mode)
    pub asi_mode: AsiMode,       // +2, +1/+1, +1/+1/+1
    pub asi_feat_mode: bool,     // true = FeatPicker was opened from ASI choice overlay

    // Currency selection (Inventory tab): 0=PP, 1=GP, 2=EP, 3=SP, 4=CP
    pub currency_selected: usize,

    // Delete confirmation
    pub delete_confirm: bool,

    // Edit character state
    pub edit_character_id: Option<Uuid>, // ID of character being edited
    pub edit_return_to_sheet: bool,      // true = return to CharacterSheet, false = CharacterList
    pub edit_field_index: usize,         // which field is focused
    pub edit_buffers: [String; 13], // text buffers: [name, xp, level, max_hp, cur_hp, temp_hp, str, dex, con, int, wis, cha, inspiration]
    pub edit_race_index: usize,
    pub edit_class_index: usize,
    pub edit_bg_index: usize,
    pub edit_race_state: ListState,
    pub edit_class_state: ListState,
    pub edit_bg_state: ListState,
    pub edit_section: EditSection, // which section of the form is active

    // Multiclass picker (in edit screen)
    pub multiclass_section: MulticlassSection,
    pub multiclass_add_index: usize, // index into classes list for the "add" picker
    pub multiclass_add_state: ListState,
    pub multiclass_selected: usize, // index into char_classes for removal

    // Subclass picker
    pub class_detail: Option<ClassDetailResponse>, // cached class detail for subclass picker
    pub subclass_picker_class_id: i32,             // class_id being subclassed (0 = primary)

    // Level-up prompt queue — drained one at a time (in edit screen or sheet)
    pub level_up_queue: Vec<LevelUpPrompt>,
    // The prompt currently being shown in the edit screen overlay (None = no overlay)
    pub level_up_current: Option<LevelUpPrompt>,
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
            screen: Screen::Login,
            should_quit: false,
            status_msg: String::new(),

            auth_mode: AuthMode::Login,
            auth_fields: [String::new(), String::new(), String::new()],
            auth_focus: 0,
            password_visible: false,

            classes: Vec::new(),
            races: Vec::new(),
            backgrounds: Vec::new(),

            characters: Vec::new(),
            selected_char: 0,
            char_list_state: ListState::default().with_selected(Some(0)),

            builder: BuilderState::default(),

            active_character: None,
            sheet_tab: SheetTab::CoreStats,
            actions_sub_tab: ActionsSubTab::All,
            features_sub_tab: FeaturesSubTab::All,
            sheet_tab_index: 0,
            sidebar_focused: true,
            content_scroll: 0,
            char_feats: Vec::new(),
            char_weapon_masteries: Vec::new(),
            char_spells: Vec::new(),
            char_inventory: Vec::new(),
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
            char_race_traits: Vec::new(),
            char_actions: None,
            actions_list_state: ListState::default().with_selected(Some(0)),
            actions_detail_modal: None,
            spell_detail_modal: None,
            conditions: Vec::new(),
            concentrating_on: None,
            spell_slots_used: [0u8; 9],
            spell_level_filter: None,
            spell_level_tab_index: 0,
            hit_dice_used: [0u8; 4],

            all_spells: Vec::new(),
            all_items: Vec::new(),
            all_feats: Vec::new(),

            editing_notes: false,
            notes_buffer: String::new(),
            notes_cursor: 0,
            picker_mode: PickerMode::None,
            picker_search: String::new(),
            picker_selected: 0,
            picker_list_state: ratatui::widgets::ListState::default().with_selected(Some(0)),
            selected_list_index: 0,
            sheet_table_state: ratatui::widgets::TableState::default().with_selected(Some(0)),
            show_compendium_detail: false,
            show_item_detail: false,

            death_saves_success: 0,
            death_saves_fail: 0,

            asi_choice_index: 0,
            asi_ability_a: 0,
            asi_ability_b: 1,
            asi_ability_c: 2,
            asi_mode: AsiMode::PlusOneTwo,
            asi_feat_mode: false,

            currency_selected: 0,

            delete_confirm: false,

            edit_character_id: None,
            edit_return_to_sheet: false,
            edit_field_index: 0,
            edit_buffers: Default::default(),
            edit_race_index: 0,
            edit_class_index: 0,
            edit_bg_index: 0,
            edit_race_state: ListState::default().with_selected(Some(0)),
            edit_class_state: ListState::default().with_selected(Some(0)),
            edit_bg_state: ListState::default().with_selected(Some(0)),
            edit_section: EditSection::Fields,

            multiclass_section: MulticlassSection::List,
            multiclass_add_index: 0,
            multiclass_add_state: ListState::default().with_selected(Some(0)),
            multiclass_selected: 0,

            class_detail: None,
            subclass_picker_class_id: 0,

            level_up_queue: Vec::new(),
            level_up_current: None,
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
            )
        });

        match core {
            (Ok(classes), Ok(races), Ok(backgrounds), Ok(spells), Ok(items), Ok(feats)) => {
                self.classes = classes.clone();
                self.races = races.clone();
                self.backgrounds = backgrounds.clone();
                self.all_spells = spells.clone();
                self.all_items = items.clone();
                self.all_feats = feats.clone();

                // Save to cache
                let cache = crate::utils::storage::CompendiumCache {
                    classes,
                    races,
                    backgrounds,
                    spells,
                    items,
                    feats,
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
                self.char_list_state.select(Some(self.selected_char));
            }
            Err(_) => {
                // Fallback to cache
                if let Some(chars) = self
                    .storage
                    .load_cache::<Vec<Character>>("characters.json")
                {
                    self.characters = chars;
                    self.is_offline = true;
                    self.status_msg = "Loaded characters from cache (Offline).".into();
                    if self.selected_char >= self.characters.len() {
                        self.selected_char = self.characters.len().saturating_sub(1);
                    }
                    self.char_list_state.select(Some(self.selected_char));
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
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                self.should_quit = true;
                return;
            }

            match self.screen {
                Screen::Login => crate::handlers::auth::handle_login_key(self, key),
                Screen::CharacterList => {
                    crate::handlers::character_list::handle_char_list_key(self, key)
                }
                Screen::CharacterBuilder => crate::handlers::builder::handle_builder_key(self, key),
                Screen::CharacterSheet => crate::handlers::sheet::handle_sheet_key(self, key),
                Screen::EditCharacter => {
                    crate::handlers::edit::handle_edit_character_key(self, key)
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

        let str_mod = crate::utils::ability_modifier(character.strength);
        let dex_mod = crate::utils::ability_modifier(character.dexterity);
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);

        // Scan inventory for weapons
        for inv in self.char_inventory.iter().filter(|i| i.is_equipped) {
            if let Some(item) = self.all_items.iter().find(|i| i.id == inv.item_id) {
                let itype = item.item_type.as_deref().unwrap_or("");
                if itype.contains('W') { // 'M'elee Weapon, 'R'anged Weapon
                    let is_finesse = item.properties.as_ref().map(|p| p.iter().any(|s| s.to_lowercase() == "finesse")).unwrap_or(false);
                    let is_ranged = itype.contains('R');
                    
                    let ability_mod = if is_ranged || (is_finesse && dex_mod > str_mod) {
                        dex_mod
                    } else {
                        str_mod
                    };

                    let hit_bonus = prof + ability_mod;
                    let damage_val = item.damage.as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("1d4");
                    
                    derived.push(crate::models::actions::ActionEntry {
                        name: item.name.clone(),
                        source: Some("Inventory".into()),
                        description: None,
                        range: Some(if is_ranged { "80/320".into() } else { "5 ft".into() }),
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
        derived.push(crate::models::actions::ActionEntry {
            name: "Unarmed Strike".into(),
            source: Some("Rules".into()),
            description: Some("You make a melee attack that involves using your body.".into()),
            range: Some("5 ft".into()),
            hit_bonus: Some(format!("{:+}", prof + str_mod)),
            damage: Some(format!("{}", 1 + str_mod)),
            max_uses: None,
            current_uses: None,
            reset_type: None,
            time: None,
        });

        derived
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
        let score = crate::utils::ch_ability_score(character, ability);
        let modifier = crate::utils::ability_modifier(score);
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);
        Some(8 + prof + modifier)
    }

    pub fn spell_attack_bonus(&self) -> Option<i32> {
        let ability = self.spellcasting_ability()?;
        let character = self.active_character.as_ref()?;
        let score = crate::utils::ch_ability_score(character, ability);
        let modifier = crate::utils::ability_modifier(score);
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);
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
                    let score = crate::utils::ch_ability_score(character, ability_key);
                    let modifier = crate::utils::ability_modifier(score);
                    let char_level = crate::utils::level_from_xp(character.experience_pts);
                    let prof = crate::utils::proficiency_bonus(char_level);

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
                let score = crate::utils::ch_ability_score(character, ability);
                let modifier = crate::utils::ability_modifier(score);
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

    /// Checks all current class features and ensures any "always prepared" spells
    /// are present in the character's spell list and marked as prepared.
    pub fn sync_always_prepared_spells(&mut self) {
        let mut to_add = Vec::new();

        for feature in &self.char_class_features {
            if let crate::models::features::Feature::GrantsSpell { spell_name } =
                feature.interpret()
            {
                // Find this spell in the compendium (lenient name matching)
                let target_name = spell_name.to_lowercase();
                if let Some(spell) = self.all_spells.iter().find(|s| {
                    let s_name = s.name.to_lowercase();
                    s_name == target_name || s_name.starts_with(&format!("{} ", target_name))
                }) {
                    // Check if character already has it
                    if !self.char_spells.iter().any(|cs| cs.spell_id == spell.id) {
                        to_add.push(spell.id);
                    } else if let Some(cs) = self
                        .char_spells
                        .iter_mut()
                        .find(|cs| cs.spell_id == spell.id)
                    {
                        // Ensure it is prepared if already present
                        cs.is_prepared = true;
                    }
                }
            }
        }
    }

    pub fn has_skill_prof(&self, skill: &str) -> bool {
        let skill_lower = skill.to_lowercase();
        self.char_chosen_skills
            .iter()
            .any(|s| s.to_lowercase() == skill_lower)
    }

    pub fn has_expertise(&self, skill: &str) -> bool {
        let skill_lower = skill.to_lowercase();
        self.char_expertise_skills
            .iter()
            .any(|s| s.to_lowercase().contains(&skill_lower))
    }

    pub fn has_perception_prof(&self) -> bool {
        self.has_skill_prof("perception")
    }

    pub fn always_prepared_spell_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for feature in &self.char_class_features {
            if let crate::models::features::Feature::GrantsSpell { spell_name } = feature.interpret() {
                let target_name = spell_name.to_lowercase();
                if let Some(spell) = self.all_spells.iter().find(|s| {
                    let s_name = s.name.to_lowercase();
                    s_name == target_name || s_name.starts_with(&format!("{} ", target_name))
                }) {
                    ids.push(spell.id);
                }
            }
        }
        ids
    }

    pub fn spellcasting_classes(&self) -> Vec<(String, i32, i32, i32)> {
        self.multiclass_spell_stats()
    }

    pub fn filtered_mastery_weapons(&self) -> Vec<&Item> {
        self.all_items
            .iter()
            .filter(|i| {
                let itype = i.item_type.as_deref().unwrap_or("");
                itype.contains('W') && (self.picker_search.is_empty() || i.name.to_lowercase().contains(&self.picker_search.to_lowercase()))
            })
            .collect()
    }

    pub fn sync_resource_limits(&mut self) {
        // Implementation for syncing resource limits based on level/class
    }
}
