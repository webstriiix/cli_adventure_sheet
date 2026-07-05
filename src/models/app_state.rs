use crate::models::features::Feature;
use ratatui::widgets::{ListState, TableState};
use serde::{Serialize, Deserialize};

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

#[derive(Debug, Clone, PartialEq)]
pub struct BgAbilityChoice {
    pub options: Vec<String>,
    pub weights: Vec<i32>,
    pub selected_idx: Option<usize>, // Which option was selected
}

pub struct BuilderState {
    pub step: CharacterCreationStep,
    pub name: String,
    pub race_id: Option<i32>,
    pub class_id: Option<i32>,
    pub subclass_id: Option<i32>,
    pub bg_id: Option<i32>,
    pub bonus_feat_id: Option<i32>, // race/species bonus feat (Versatile)
    pub background_feat_id: Option<i32>, // background bonus feat
    pub race_skill_choice: Option<String>, // Human Skillful: bonus skill proficiency
    pub feat_picker_search: String,
    pub feat_picker_index: usize,

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
    pub bg_ability_step: usize, // Which choice we are making
    pub bg_ability_focus: usize, // Which option in current choice is selected

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
    pub ability_scores: [i32; 6],     // used by step_abilities
    pub ability_method: AbilityMethod, // StandardArray / PointBuy / Manual
    pub ability_cursor: usize,         // currently highlighted ability row

    // --- EQUIPMENT STEP FIELDS ---
    pub equipment_options: Vec<String>, // list of option labels for the class
    pub equipment_choices: Vec<usize>,  // indices of chosen equipment options
    pub starting_gold: Option<i32>,     // if the user opts for gold instead
}

impl Default for BuilderState {
    fn default() -> Self {
        Self {
            step: CharacterCreationStep::Class,
            name: String::new(),
            race_id: None,
            class_id: None,
            subclass_id: None,
            bg_id: None,
            bonus_feat_id: None,
            background_feat_id: None,
            race_skill_choice: None,
            feat_picker_search: String::new(),
            feat_picker_index: 0,
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
