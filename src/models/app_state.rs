use crate::models::features::Feature;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Login,
    CharacterList,
    CharacterBuilder,
    CharacterSheet,
    EditCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharacterCreationStep {
    Race,
    RaceSkill,      // Human Skillful: choose a bonus skill proficiency
    RaceFeat,       // Human Versatile: choose an Origin feat
    BackgroundAbilities, // XPHB backgrounds: assign +2/+1 from 3 abilities
    BackgroundFeat, // XPHB backgrounds: choose a bonus Origin feat
    Class,
    Subclass,
    Abilities,
    Background,
    Languages,
    Proficiencies,
    Equipment,
    Spells,
    Details,
    Summary,
    /// Triggered when the currently-pending feat grants weapon masteries.
    FeatWeaponMastery,
    /// Triggered when the currently-pending feat grants skill proficiency choices.
    FeatSkillChoice,
}

pub struct BuilderState {
    pub step: CharacterCreationStep,
    pub name: String,
    pub race_id: Option<i32>,
    pub subrace_id: Option<i32>,
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
    pub bg_ability_focus: usize,
    pub bg_ability_step: u8, // 0=picking +2, 1=picking +1, 2=done

    // Abilities
    pub abilities: [i32; 6],
    pub ability_mode: AbilityMode,
    pub ability_focus: usize,
    pub standard_pool: Vec<bool>,
    pub point_buy_points: i32,

    // Proficiencies & Choices
    pub skill_choices: Vec<String>,
    pub tool_choices: Vec<String>,
    pub language_choices: Vec<String>,

    // Equipment & Spells
    pub equipment_option: Option<usize>, // 0 for starting equip, 1 for gold
    pub starting_gold: i32,
    pub known_spells: Vec<i32>,
    pub prepared_spells: Vec<i32>,

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
    pub text_buffers: [String; 10], // for details input
    pub summary_scroll: usize,
}

impl Default for BuilderState {
    fn default() -> Self {
        Self {
            step: CharacterCreationStep::Race,
            name: String::new(),
            race_id: None,
            subrace_id: None,
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
            bg_ability_focus: 0,
            bg_ability_step: 0,
            abilities: [8; 6], // default to 8 for point buy / manual
            ability_mode: AbilityMode::StandardArray,
            ability_focus: 0,
            standard_pool: vec![true; 6],
            point_buy_points: 27,
            skill_choices: Vec::new(),
            tool_choices: Vec::new(),
            language_choices: Vec::new(),
            equipment_option: None,
            starting_gold: 0,
            known_spells: Vec::new(),
            prepared_spells: Vec::new(),
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
            text_buffers: Default::default(),
            summary_scroll: 0,
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

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
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
