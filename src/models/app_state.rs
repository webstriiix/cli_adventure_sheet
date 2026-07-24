use crate::models::features::Feature;
use ratatui::widgets::{ListState, TableState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Login,
    CharacterList,
    CharacterBuilder,
    CharacterSheet,
    EditCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterCreationStep {
    Class,
    Background,
    Species,
    Abilities,
    Equipment,
}

impl std::fmt::Display for CharacterCreationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BgAbilityChoice {
    pub options: Vec<String>,
    pub weights: Vec<i32>,
    pub selected_idx: Option<usize>, // Which option was selected
}

/// State for the choice-picker modal opened when the user presses Enter on a
/// feature slot in the Class Manager feature list.
#[derive(Debug, Clone)]
pub struct FeatureChoiceModal {
    /// The `ClassFeature.id` this modal is choosing for.
    pub feature_id: i32,
    /// Which slot index (0-based) within the feature is being filled.
    pub slot_index: usize,
    /// Human-readable title shown in the modal border.
    pub title: String,
    /// Full option list (pre-filtered for this feature type).
    pub options: Vec<String>,
    /// Live-filter search string.
    pub search: String,
    /// Cursor within the filtered result set.
    pub cursor: usize,
}

pub struct BuilderState {
    pub step: CharacterCreationStep,
    pub name: String,
    pub race_id: Option<i32>,
    /// The class that has been **confirmed** (Tab/confirm) and will be saved.
    pub class_id: Option<i32>,
    /// The class currently **highlighted for preview** (Enter in the list).
    /// The right pane renders this class's details; it becomes `class_id` on Tab-confirm.
    pub previewed_class_id: Option<i32>,
    pub subclass_id: Option<i32>,
    pub bg_id: Option<i32>,
    pub bonus_feat_id: Option<i32>, // race/species bonus feat (Versatile)
    pub background_feat_id: Option<i32>, // background bonus feat
    pub race_skill_choice: Option<String>, // Human Skillful: bonus skill proficiency
    pub feat_picker_search: String,
    pub feat_picker_index: usize,
    pub class_search: String,
    pub background_search: String,
    pub race_search: String,

    // Feature interpreter — stores the structured feature currently driving an
    // "extra" builder step (FeatWeaponMastery / FeatSkillChoice).
    pub builder_pending_feature: Option<Feature>,
    /// Weapon masteries chosen via FeatWeaponMastery step (weapon names).
    pub weapon_mastery_choices: Vec<String>,
    /// Skill proficiencies chosen via FeatSkillChoice step.
    pub feat_skill_choices: Vec<String>,

    // Background ability bonuses (+2/+1 from XPHB backgrounds)
    pub bg_ability_bonuses: [i32; 6],
    pub bg_ability_choices: Vec<BgAbilityChoice>, // Parsed choices
    pub bg_ability_step: usize,                   // Which choice we are making
    pub bg_ability_focus: usize,                  // Which option in current choice is selected

    // Abilities
    pub abilities: [i32; 6],
    pub ability_mode: AbilityMode,
    pub ability_focus: usize,
    pub standard_pool: Vec<bool>,

    // Proficiencies & Choices
    pub skill_choices: Vec<String>,

    // Equipment & Spells
    pub equipment_option: Option<usize>, // 0 for starting equip, 1 for gold
    pub known_spells: Vec<i32>,

    // Details
    pub age: String,
    pub height: String,
    pub weight: String,
    pub appearance: String, // eye, hair, skin
    pub alignment: String,
    pub trait_text: String,
    pub ideal: String,
    pub bond: String,
    pub flaw: String,

    // Orchestration & Limits
    pub skip_subclass: bool,
    pub spellcasting_type: String,
    pub language_count: i32,

    // UI state for lists/focus
    pub list_state: ListState,
    pub alignment_list_state: ListState,
    pub focus_index: usize,

    // --- NEW DRAFT & MODAL STATE FIELDS ---
    pub draft_id: Option<uuid::Uuid>,
    pub level: i32,
    pub species_id: Option<i32>,
    pub lineage_id: Option<i32>,
    pub subrace_id: Option<i32>, // alias for lineage_id used in step_race
    pub show_subclass_modal: bool,
    pub show_feat_modal: bool,
    pub show_lineage_menu: bool,
    pub subclass_list_state: ListState,
    pub feat_list_state: ListState,
    pub lineage_list_state: ListState,
    /// TableState controlling the Level Progression table scroll/selection
    pub progression_table_state: TableState,

    // --- ABILITY SCORE METHOD FIELDS ---
    pub ability_scores: [i32; 6],      // used by step_abilities
    pub ability_method: AbilityMethod, // StandardArray / PointBuy / Manual
    pub ability_cursor: usize,         // currently highlighted ability row

    // --- EQUIPMENT STEP FIELDS ---
    pub equipment_options: Vec<String>, // list of option labels for the class
    pub equipment_choices: Vec<usize>,  // indices of chosen equipment options
    pub starting_gold: Option<i32>,     // if the user opts for gold instead

    // --- CLASS FEATURE PANEL FIELDS ---
    /// Index of the highlighted row in the feature list panel (titles + slot sub-rows).
    pub feature_cursor: usize,
    /// ListState that drives scroll position for the feature list.
    pub feature_list_state: ratatui::widgets::ListState,
    /// Active tab in the Class Manager right-panel: 0 = Features, 1 = Spells.
    pub class_active_tab: usize,
    /// Per-feature choices: key = ClassFeature.id, value = Vec of chosen strings
    /// (weapon names, skill names, feat names, etc.).
    pub class_feature_choices: std::collections::HashMap<i32, Vec<String>>,
    /// ASI choices: key = character_level (4, 8, 12, 16, 20), value = description
    /// (e.g., "+2 Intelligence" or "Feat: Tough").
    pub asi_choices: std::collections::HashMap<i32, String>,
    /// `Some((title, description))` while the Ctrl+K feature detail modal is open.
    pub feature_detail_modal: Option<(String, String)>,
    /// Scroll offset for the feature detail modal body.
    pub feature_modal_scroll: u16,
    /// `Some(...)` while the choice-picker modal is open.
    /// Stores: (feature_id, slot_index, options list, current search, cursor).
    pub feature_choice_modal: Option<FeatureChoiceModal>,
    /// Skill choice modal state
    pub show_skill_choice_modal: bool,
    pub skill_choice_list_state: ListState,
    pub skill_choice_search: String,
    pub skill_choice_cursor: usize,
    pub skill_choice_slot: usize, // Which skill slot (0-indexed)

    // --- UNIFIED PROGRESSION MANIFEST & MODAL FIELDS ---
    pub progression_manifest: Option<crate::models::ProgressionManifest>,
    pub show_progression_asi_modal: bool,
    pub show_progression_wm_modal: bool,
    pub progression_slot_level: Option<i32>,
    pub progression_blink_tick: u64,
}

impl Default for BuilderState {
    fn default() -> Self {
        Self {
            step: CharacterCreationStep::Class,
            name: String::new(),
            race_id: None,
            class_id: None,
            previewed_class_id: None,
            subclass_id: None,
            bg_id: None,
            bonus_feat_id: None,
            background_feat_id: None,
            race_skill_choice: None,
            feat_picker_search: String::new(),
            feat_picker_index: 0,
            class_search: String::new(),
            background_search: String::new(),
            race_search: String::new(),
            builder_pending_feature: None,
            weapon_mastery_choices: Vec::new(),
            feat_skill_choices: Vec::new(),
            bg_ability_bonuses: [0; 6],
            bg_ability_choices: Vec::new(),
            bg_ability_step: 0,
            bg_ability_focus: 0,
            abilities: [10; 6], // default to 10 for point buy / manual
            ability_mode: AbilityMode::Manual,
            ability_focus: 0,
            standard_pool: vec![true; 6],
            skill_choices: Vec::new(),
            equipment_option: None,
            known_spells: Vec::new(),
            age: String::new(),
            height: String::new(),
            weight: String::new(),
            appearance: String::new(),
            alignment: String::new(),
            trait_text: String::new(),
            ideal: String::new(),
            bond: String::new(),
            flaw: String::new(),
            skip_subclass: false,
            spellcasting_type: String::new(),
            language_count: 0,
            list_state: ListState::default().with_selected(Some(0)),
            alignment_list_state: ListState::default().with_selected(Some(0)),
            focus_index: 0,
            draft_id: None,
            level: 1,
            species_id: None,
            lineage_id: None,
            subrace_id: None,
            show_subclass_modal: false,
            show_feat_modal: false,
            show_lineage_menu: false,
            subclass_list_state: ListState::default().with_selected(Some(0)),
            feat_list_state: ListState::default().with_selected(Some(0)),
            lineage_list_state: ListState::default().with_selected(Some(0)),
            // Table state for level progression (used in class selection view)
            progression_table_state: TableState::default().with_selected(Some(0)),
            ability_scores: [8; 6],
            ability_method: AbilityMethod::StandardArray,
            ability_cursor: 0,
            equipment_options: Vec::new(),
            equipment_choices: Vec::new(),
            starting_gold: None,
            feature_cursor: 0,
            feature_list_state: ListState::default().with_selected(Some(0)),
            class_active_tab: 0,
            class_feature_choices: std::collections::HashMap::new(),
            asi_choices: std::collections::HashMap::new(),
            feature_detail_modal: None,
            feature_modal_scroll: 0,
            feature_choice_modal: None,
            show_skill_choice_modal: false,
            skill_choice_list_state: ListState::default().with_selected(Some(0)),
            skill_choice_search: String::new(),
            skill_choice_cursor: 0,
            skill_choice_slot: 0,
            progression_manifest: None,
            show_progression_asi_modal: false,
            show_progression_wm_modal: false,
            progression_slot_level: None,
            progression_blink_tick: 0,
        }
    }
}
impl BuilderState {
    pub fn all_abilities_set(&self) -> bool {
        match self.ability_mode {
            AbilityMode::Manual => true,
            AbilityMode::StandardArray => self.abilities.iter().all(|&v| v != 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthMode {
    Login,
    Signup,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbilityMode {
    Manual,
    StandardArray,
}

/// Method used in the 5-step builder's ability score allocation step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AbilityMethod {
    StandardArray,
    PointBuy,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SheetTab {
    CoreStats,
    Skills,
    Actions,
    Inventory,
    Spells,
    Features,
    Proficiency,
    Background,
    Notes,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PickerMode {
    None,
    SpellPicker,
    ItemPicker,
    FeatPicker,
    AsiFeatChoice, // ASI or feat at a level-up milestone
    WeaponMasteryPicker,
    ConditionPicker, // Toggle active conditions
    SubclassPicker,  // Pick a subclass for the active or a multiclass
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditSection {
    Fields,        // text fields (name, xp, hp, abilities)
    Race,          // race picker list
    Class,         // class picker list
    Subclass,      // subclass picker list
    Background,    // background picker list
    Multiclass,    // multiclass manager
    LevelUpChoice, // ASI / subclass prompt triggered by XP change
}

/// Sub-state within the multiclass manager panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MulticlassSection {
    List, // browsing current multiclass entries (d to remove)
    Add,  // class picker for adding a new multiclass
}

impl SheetTab {
    pub const ALL: [SheetTab; 9] = [
        SheetTab::CoreStats,
        SheetTab::Skills,
        SheetTab::Actions,
        SheetTab::Inventory,
        SheetTab::Spells,
        SheetTab::Features,
        SheetTab::Proficiency,
        SheetTab::Background,
        SheetTab::Notes,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SheetTab::CoreStats => "Core Stats",
            SheetTab::Skills => "Skills",
            SheetTab::Actions => "Actions",
            SheetTab::Inventory => "Inventory",
            SheetTab::Spells => "Spells",
            SheetTab::Features => "Features & Traits",
            SheetTab::Proficiency => "Prof. & Training",
            SheetTab::Background => "Background",
            SheetTab::Notes => "Notes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionsSubTab {
    All,
    Attack,
    Action,
    BonusAction,
    Reaction,
    Other,
    LimitedUse,
}
