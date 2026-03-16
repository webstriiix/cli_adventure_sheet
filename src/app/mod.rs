use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::widgets::ListState;
use uuid::Uuid;

use crate::client::ApiClient;
use crate::models::app_state::{
    ActionsSubTab, AuthMode, BuilderState, EditSection, MulticlassSection, PickerMode, Screen,
    SheetTab,
};
use crate::models::character::{
    Character, CharacterClass, CharacterFeat, CharacterSpell, InventoryItem,
};
use crate::models::compendium::{
    Background, Class, ClassDetailResponse, ClassFeature, Feat, Item, Race, Spell,
};

/// A pending prompt that needs to be shown to the player after a level-up.
#[derive(Debug, Clone)]
pub enum LevelUpPrompt {
    /// Player must choose a subclass for this class (shown at the subclass gate level).
    SubclassChoice { class_id: i32, class_name: String },
    /// Player must choose an ASI or a Feat (shown at levels 4, 8, 12, 16, 19…).
    AsiOrFeat { class_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeaturesSubTab {
    All,
    ClassFeatures,
    SpeciesTraits,
    Feats,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsiMode {
    PlusOneTwo,
    PlusOneThree,
}
use crate::ui;

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
        Self {
            client,
            rt,
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

    pub fn fetch_compendium_data(&mut self) {
        let rt = self.rt.clone();

        // Core data — must all succeed
        let core = rt.block_on(async {
            tokio::join!(
                self.client.get_classes(None, None),
                self.client.get_races(None, None),
                self.client.get_backgrounds(None, None),
                self.client.get_spells(None, None),
                self.client.get_items(None, None),
            )
        });

        match core {
            (Ok(classes), Ok(races), Ok(backgrounds), Ok(spells), Ok(items)) => {
                self.classes = classes;
                self.races = races;
                self.backgrounds = backgrounds;
                self.all_spells = spells;
                self.all_items = items;
            }
            (Err(e), _, _, _, _)
            | (_, Err(e), _, _, _)
            | (_, _, Err(e), _, _)
            | (_, _, _, Err(e), _)
            | (_, _, _, _, Err(e)) => {
                self.status_msg = format!("Failed to load data: {e}");
                return;
            }
        }

        // Feats — optional, don't block the rest of the app if unavailable
        match rt.block_on(self.client.get_compendium_feats(None)) {
            Ok(feats) => self.all_feats = feats,
            Err(e) => self.status_msg = format!("Warning: could not load feats: {e}"),
        }
    }

    // ── Combat stat helpers ──

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
                match itype {
                    "LA" => {
                        let ac = item
                            .armor_class
                            .as_ref()
                            .and_then(|v| v.as_i64())
                            .unwrap_or(11) as i32;
                        base = ac + dex_mod;
                    }
                    "MA" => {
                        let ac = item
                            .armor_class
                            .as_ref()
                            .and_then(|v| v.as_i64())
                            .unwrap_or(13) as i32;
                        base = ac + dex_mod.min(2);
                    }
                    "HA" => {
                        let ac = item
                            .armor_class
                            .as_ref()
                            .and_then(|v| v.as_i64())
                            .unwrap_or(16) as i32;
                        base = ac;
                    }
                    "S" => {
                        // Shield always +2
                        shield_bonus = 2;
                    }
                    _ => {}
                }
            }
        }
        base + shield_bonus
    }

    /// The ability that governs spellcasting for the character's class.
    pub fn spellcasting_ability(&self) -> Option<&'static str> {
        // First try to look up from the actual class data
        if let Some(class) = self.classes.iter().find(|cl| cl.id == self.active_class_id) {
            if let Some(ability) = &class.spellcasting_ability {
                return match ability.to_lowercase().as_str() {
                    "int" => Some("int"),
                    "wis" => Some("wis"),
                    "cha" => Some("cha"),
                    _ => None,
                };
            }
        }
        // Fallback to hardcoded class names
        match self.char_class_name.to_lowercase().as_str() {
            "wizard" => Some("int"),
            "cleric" | "druid" | "ranger" => Some("wis"),
            "bard" | "paladin" | "sorcerer" | "warlock" => Some("cha"),
            _ => None,
        }
    }

    /// Spell save DC = 8 + prof bonus + spellcasting ability modifier.
    pub fn spell_save_dc(&self) -> Option<i32> {
        let character = self.active_character.as_ref()?;
        let ability = self.spellcasting_ability()?;
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);
        let score = crate::utils::ch_ability_score(character, ability);
        let modifier = crate::utils::ability_modifier(score);
        Some(8 + prof + modifier)
    }

    /// Spell attack bonus = prof bonus + spellcasting ability modifier.
    pub fn spell_attack_bonus(&self) -> Option<i32> {
        let character = self.active_character.as_ref()?;
        let ability = self.spellcasting_ability()?;
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);
        let score = crate::utils::ch_ability_score(character, ability);
        let modifier = crate::utils::ability_modifier(score);
        Some(prof + modifier)
    }

    /// Returns per-class spellcasting stats: (class_name, ability_mod, spell_attack, save_dc).
    pub fn spellcasting_classes(&self) -> Vec<(String, i32, i32, i32)> {
        let character = match self.active_character.as_ref() {
            Some(c) => c,
            None => return Vec::new(),
        };
        let level = crate::utils::level_from_xp(character.experience_pts);
        let prof = crate::utils::proficiency_bonus(level);
        let mut results = Vec::new();

        for cc in &self.char_classes {
            if let Some(class) = self.classes.iter().find(|cl| cl.id == cc.class_id) {
                if let Some(ability_str) = &class.spellcasting_ability {
                    let ability = match ability_str.to_lowercase().as_str() {
                        "int" => "int",
                        "wis" => "wis",
                        "cha" => "cha",
                        _ => continue,
                    };
                    let score = crate::utils::ch_ability_score(character, ability);
                    let modifier = crate::utils::ability_modifier(score);
                    let attack = prof + modifier;
                    let dc = 8 + prof + modifier;
                    results.push((class.name.clone(), modifier, attack, dc));
                }
            }
        }

        // Deduplicate by class name
        results.dedup_by(|a, b| a.0 == b.0);

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
            if let Some(n) = r.speed.get("walk").and_then(|v| v.as_i64()) {
                return n as i32;
            }
        }
        30 // Default
    }

    /// Whether the character has proficiency in a given skill (matched by substring).
    /// Checks background-granted skills (stored in char_chosen_skills).
    pub fn has_skill_prof(&self, skill: &str) -> bool {
        let skill_lower = skill.to_lowercase();
        if self
            .char_chosen_skills
            .iter()
            .any(|s| s.to_lowercase().contains(&skill_lower))
        {
            return true;
        }
        false
    }

    /// Returns a list of all weapons that have a mastery property, optionally filtered by search.
    /// Includes hardcoded 2024 masteries if the backend doesn't provide them.
    pub fn filtered_mastery_weapons(&self) -> Vec<crate::models::compendium::Item> {
        let q = self.builder.feat_picker_search.to_lowercase();

        // 2024 Weapon Mastery Mapping
        let masteries = [
            ("Greataxe", "Cleave"),
            ("Halberd", "Cleave"),
            ("Glaive", "Graze"),
            ("Greatsword", "Graze"),
            ("Dagger", "Nick"),
            ("Light Hammer", "Nick"),
            ("Sickle", "Nick"),
            ("Scimitar", "Nick"),
            ("Greatclub", "Push"),
            ("Pike", "Push"),
            ("Warhammer", "Push"),
            ("Heavy Crossbow", "Push"),
            ("Mace", "Sap"),
            ("Spear", "Sap"),
            ("Flail", "Sap"),
            ("Longsword", "Sap"),
            ("Morningstar", "Sap"),
            ("War Pick", "Sap"),
            ("Club", "Slow"),
            ("Javelin", "Slow"),
            ("Light Crossbow", "Slow"),
            ("Sling", "Slow"),
            ("Whip", "Slow"),
            ("Longbow", "Slow"),
            ("Musket", "Slow"),
            ("Quarterstaff", "Topple"),
            ("Battleaxe", "Topple"),
            ("Lance", "Topple"),
            ("Maul", "Topple"),
            ("Trident", "Topple"),
            ("Handaxe", "Vex"),
            ("Dart", "Vex"),
            ("Shortbow", "Vex"),
            ("Rapier", "Vex"),
            ("Shortsword", "Vex"),
            ("Blowgun", "Vex"),
            ("Hand Crossbow", "Vex"),
            ("Pistol", "Vex"),
        ];

        self.all_items
            .iter()
            .filter_map(|i| {
                let mut item = i.clone();

                // If backend doesn't have mastery, check our hardcoded 2024 list
                if item.mastery.is_none() || item.mastery.as_ref().unwrap().is_empty() {
                    if let Some((_, m)) = masteries
                        .iter()
                        .find(|(w, _)| i.name.to_lowercase() == w.to_lowercase())
                    {
                        item.mastery = Some(vec![m.to_string()]);
                    }
                }

                if item.mastery.is_some() && (q.is_empty() || item.name.to_lowercase().contains(&q))
                {
                    Some(item)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Apply the selected weapon string to the current character's weapon masteries list.
    pub fn confirm_weapon_mastery_choice(&mut self) {
        // TODO: Implement weapon mastery choice logic
        // E.g. push selected weapon into self.char_weapon_masteries and close picker
        self.picker_mode = PickerMode::None;
    }

    /// Whether the character has expertise (double proficiency) in a given skill.
    pub fn has_expertise(&self, skill: &str) -> bool {
        let skill_lower = skill.to_lowercase();
        self.char_expertise_skills
            .iter()
            .any(|s| s.to_lowercase().contains(&skill_lower))
    }

    /// Whether the character has Perception proficiency from their class.
    pub fn has_perception_prof(&self) -> bool {
        self.has_skill_prof("perception")
    }

    // ── Helper: resolve feat / item / spell name ──

    pub fn feat_name(&self, feat_id: i32) -> String {
        self.all_feats
            .iter()
            .find(|f| f.id == feat_id)
            .map(|f| f.name.clone())
            .unwrap_or_else(|| format!("Feat #{feat_id}"))
    }

    // ── Helper: resolve item name ──

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

        // Add missing ones (local only, normally would persist to API)
        for spell_id in to_add {
            self.char_spells
                .push(crate::models::character::CharacterSpell {
                    character_id: self
                        .active_character
                        .as_ref()
                        .map(|c| c.id)
                        .unwrap_or_default(),
                    spell_id,
                    is_prepared: true,
                });
        }
    }

    /// Returns a list of spell IDs that are granted by features (always prepared).
    pub fn always_prepared_spell_ids(&self) -> Vec<i32> {
        let mut ids = Vec::new();
        for feature in &self.char_class_features {
            if let crate::models::features::Feature::GrantsSpell { spell_name } =
                feature.interpret()
            {
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

    /// Recalculates maximum uses for specific features like "Lay on Hands"
    /// and "Channel Divinity" based on 2024 scaling rules.
    pub fn sync_resource_limits(&mut self) {
        let mut paladin_level = self
            .char_classes
            .iter()
            .find(|cc| {
                self.classes
                    .iter()
                    .find(|cl| cl.id == cc.class_id)
                    .map(|cl| cl.name.to_lowercase() == "paladin")
                    .unwrap_or(false)
            })
            .map(|cc| cc.level)
            .unwrap_or(0);

        // Fallback for mono-class Paladins if char_classes is empty
        if paladin_level == 0 && self.char_class_name.to_lowercase() == "paladin" {
            if let Some(c) = &self.active_character {
                paladin_level = crate::utils::level_from_xp(c.experience_pts);
            }
        }

        if paladin_level == 0 {
            return;
        }

        if let Some(actions) = self.char_actions.as_mut() {
            // Update Lay on Hands: 5 * Paladin Level
            // Search across ALL categories (All, Action, BonusAction, etc.)
            let expected_max = 5 * paladin_level;

            let lists = vec![
                &mut actions.all,
                &mut actions.attack,
                &mut actions.action,
                &mut actions.bonus_action,
                &mut actions.reaction,
                &mut actions.other,
                &mut actions.limited_use,
            ];

            for list in lists {
                for item in list.iter_mut() {
                    if item.name.to_lowercase().contains("lay on hands") {
                        if item.max_uses != Some(expected_max) {
                            item.max_uses = Some(expected_max);
                            if item.current_uses.is_none() || item.current_uses > Some(expected_max)
                            {
                                item.current_uses = Some(expected_max);
                            }
                        }
                    }
                }
            }

            // Update Channel Divinity: 2 at Level 3, 3 at Level 11
            let cd_exists = actions
                .limited_use
                .iter()
                .any(|a| a.name.to_lowercase().contains("channel divinity"));
            if cd_exists {
                if let Some(cd) = actions
                    .limited_use
                    .iter_mut()
                    .find(|a| a.name.to_lowercase().contains("channel divinity"))
                {
                    let expected_max = if paladin_level >= 11 {
                        3
                    } else if paladin_level >= 3 {
                        2
                    } else {
                        cd.max_uses.unwrap_or(1)
                    };

                    if cd.max_uses != Some(expected_max) {
                        cd.max_uses = Some(expected_max);
                        if cd.current_uses.is_none() || cd.current_uses > Some(expected_max) {
                            cd.current_uses = Some(expected_max);
                        }
                    }
                }
            } else if paladin_level >= 3 {
                // AUTO-GRANT if missing
                let max = if paladin_level >= 11 { 3 } else { 2 };
                actions.limited_use.push(crate::models::actions::ActionEntry {
                    name: "Channel Divinity".to_string(),
                    source: Some("Paladin".to_string()),
                    description: Some("You can channel divine energy directly from the Outer Planes, using it to fuel magical effects. You regain one of its expended uses when you finish a Short Rest, and you regain all expended uses when you finish a Long Rest.".to_string()),
                    range: None,
                    hit_bonus: None,
                    damage: None,
                    max_uses: Some(max),
                    current_uses: Some(max),
                    reset_type: Some("Short/Long Rest".to_string()),
                    time: None,
                });
            }

            // Auto-grant Divine Sense if missing
            if paladin_level >= 3 {
                let ds_exists = actions
                    .all
                    .iter()
                    .any(|a| a.name.to_lowercase().contains("divine sense"));
                if !ds_exists {
                    let ds = crate::models::actions::ActionEntry {
                        name: "Divine Sense".to_string(),
                        source: Some("Paladin".to_string()),
                        description: Some("As a Bonus Action, you can open your awareness to detect Celestials, Fiends, and Undead within 60 feet.".to_string()),
                        range: Some("60 ft".to_string()),
                        hit_bonus: None,
                        damage: None,
                        max_uses: None,
                        current_uses: None,
                        reset_type: None,
                        time: Some(serde_json::json!({"number": 1, "unit": "bonus"})),
                    };
                    actions.all.push(ds.clone());
                    actions.bonus_action.push(ds);
                }
            }
        }
    }
}
