//! `UiState` — all TUI navigation, widget, and modal state.
//!
//! These fields were extracted from the `App` god-struct as part of the
//! Clean Architecture refactor (Step 1).  `App` implements `Deref<Target =
//! UiState>` so every existing `app.field` access continues to compile
//! unchanged.  Future steps will remove the `Deref` impls and update
//! call-sites explicitly.
//!
//! # Invariant
//! This module must have **zero** dependencies on `crate::client` or any
//! infrastructure concern.  It may import `ratatui` widget types because it
//! owns presentation state.

use ratatui::widgets::{ListState, TableState};
use uuid::Uuid;

use crate::models::{
    app_state::{
        ActionsSubTab, AuthMode, EditSection, MulticlassSection, PickerMode, Screen, SheetTab,
    },
    compendium::ClassDetailResponse,
};
use crate::app::{AsiMode, FeaturesSubTab, LevelUpPrompt};

// ── UiState ───────────────────────────────────────────────────────────────────

pub struct UiState {
    // ── Global navigation ────────────────────────────────────────────────────
    pub screen: Screen,
    pub should_quit: bool,
    /// Status-bar message shown at the bottom of every screen.
    pub status_msg: String,

    // ── Auth screen ──────────────────────────────────────────────────────────
    pub auth_mode: AuthMode,
    /// [username, password, confirm_password]
    pub auth_fields: [String; 3],
    pub auth_focus: usize,

    // ── Character list ───────────────────────────────────────────────────────
    pub selected_char: usize,
    pub char_list_state: ListState,
    /// Pending delete confirmation overlay.
    pub delete_confirm: bool,

    // ── Character sheet tabs / focus ─────────────────────────────────────────
    pub sheet_tab: SheetTab,
    pub sheet_tab_index: usize,
    pub actions_sub_tab: ActionsSubTab,
    pub features_sub_tab: FeaturesSubTab,
    pub sidebar_focused: bool,
    pub content_scroll: usize,

    // ── Widget list/table states ─────────────────────────────────────────────
    pub actions_list_state: ListState,
    pub picker_list_state: ListState,
    pub sheet_table_state: TableState,

    // ── Modals ───────────────────────────────────────────────────────────────
    /// `Some((title, body))` while the action-detail modal is open.
    pub actions_detail_modal: Option<(String, String)>,
    /// `Some((title, body))` while the spell-detail modal is open.
    pub spell_detail_modal: Option<(String, String)>,
    /// `Some((title, body))` while the inventory-item modal is open.
    pub inventory_item_detail_modal: Option<(String, String)>,

    // ── Spell tab UI ─────────────────────────────────────────────────────────
    /// `None` = All, `Some(0)` = cantrips, `Some(1-9)` = spell levels.
    pub spell_level_filter: Option<i32>,
    /// 0 = All, 1 = Cantrips, 2 = 1st, …, 10 = 9th.
    pub spell_level_tab_index: usize,

    // ── Notes tab UI ─────────────────────────────────────────────────────────
    pub editing_notes: bool,
    pub notes_buffer: String,
    /// Byte offset of the cursor within `notes_buffer`.
    pub notes_cursor: usize,

    // ── Picker overlay ───────────────────────────────────────────────────────
    pub picker_mode: PickerMode,
    pub picker_search: String,
    pub picker_selected: usize,
    pub selected_list_index: usize,

    // ── ASI / Feat choice overlay ────────────────────────────────────────────
    /// 0 = ability A, 1 = ability B, 2 = confirm.
    pub asi_choice_index: usize,
    pub asi_ability_a: usize,
    pub asi_ability_b: usize,
    pub asi_ability_c: usize,
    pub asi_mode: AsiMode,
    /// `true` when `FeatPicker` was opened from the ASI choice overlay.
    pub asi_feat_mode: bool,

    // ── Inventory tab UI ─────────────────────────────────────────────────────
    /// 0 = PP, 1 = GP, 2 = EP, 3 = SP, 4 = CP.
    pub currency_selected: usize,

    // ── Proficiency editing overlay ──────────────────────────────────────────
    pub editing_proficiencies: bool,
    pub selected_ability_idx: usize,

    // ── Edit character screen ────────────────────────────────────────────────
    pub edit_character_id: Option<Uuid>,
    /// `true` → return to `CharacterSheet`; `false` → return to `CharacterList`.
    pub edit_return_to_sheet: bool,
    pub edit_field_index: usize,
    /// Text buffers: [name, xp, level, max_hp, cur_hp, temp_hp, str, dex, con, int, wis, cha, inspiration]
    pub edit_buffers: [String; 13],
    pub edit_race_index: usize,
    pub edit_class_index: usize,
    pub edit_subclass_index: usize,
    pub edit_bg_index: usize,
    pub edit_race_state: ListState,
    pub edit_class_state: ListState,
    pub edit_subclass_state: ListState,
    pub edit_bg_state: ListState,
    pub edit_section: EditSection,

    // ── Multiclass picker (within edit screen) ───────────────────────────────
    pub multiclass_section: MulticlassSection,
    pub multiclass_add_index: usize,
    pub multiclass_add_state: ListState,
    pub multiclass_selected: usize,

    // ── Subclass picker overlay ──────────────────────────────────────────────
    /// Cached class detail driving the subclass picker overlay.
    pub class_detail: Option<ClassDetailResponse>,
    pub subclass_picker_class_id: i32,

    // ── Level-up prompt queue ────────────────────────────────────────────────
    /// Queue of pending level-up prompts (drained one at a time).
    pub level_up_queue: Vec<LevelUpPrompt>,
    /// The prompt currently shown in the edit-screen overlay.
    pub level_up_current: Option<LevelUpPrompt>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            screen: Screen::Login,
            should_quit: false,
            status_msg: String::new(),

            auth_mode: AuthMode::Login,
            auth_fields: [String::new(), String::new(), String::new()],
            auth_focus: 0,

            selected_char: 0,
            char_list_state: ListState::default().with_selected(Some(0)),
            delete_confirm: false,

            sheet_tab: SheetTab::CoreStats,
            sheet_tab_index: 0,
            actions_sub_tab: ActionsSubTab::All,
            features_sub_tab: FeaturesSubTab::All,
            sidebar_focused: true,
            content_scroll: 0,

            actions_list_state: ListState::default().with_selected(Some(0)),
            picker_list_state: ListState::default().with_selected(Some(0)),
            sheet_table_state: TableState::default().with_selected(Some(0)),

            actions_detail_modal: None,
            spell_detail_modal: None,
            inventory_item_detail_modal: None,

            spell_level_filter: None,
            spell_level_tab_index: 0,

            editing_notes: false,
            notes_buffer: String::new(),
            notes_cursor: 0,

            picker_mode: PickerMode::None,
            picker_search: String::new(),
            picker_selected: 0,
            selected_list_index: 0,

            asi_choice_index: 0,
            asi_ability_a: 0,
            asi_ability_b: 1,
            asi_ability_c: 2,
            asi_mode: AsiMode::PlusOneTwo,
            asi_feat_mode: false,

            currency_selected: 0,

            editing_proficiencies: false,
            selected_ability_idx: 0,

            edit_character_id: None,
            edit_return_to_sheet: false,
            edit_field_index: 0,
            edit_buffers: Default::default(),
            edit_race_index: 0,
            edit_class_index: 0,
            edit_subclass_index: 0,
            edit_bg_index: 0,
            edit_race_state: ListState::default().with_selected(Some(0)),
            edit_class_state: ListState::default().with_selected(Some(0)),
            edit_subclass_state: ListState::default().with_selected(Some(0)),
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
}
