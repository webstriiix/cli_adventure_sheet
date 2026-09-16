# PROJECT_CONTEXT.md
> Persistent working memory for all development sessions.
> Last updated: 2026-07-19

---

## 1. Project Overview & Vision

**CLI Adventure Sheet** is a terminal-based D&D 5e character sheet manager inspired by D&D Beyond's character creation experience. It is a two-component system:

| Component | Technology |
|---|---|
| Frontend (TUI) | Rust · `ratatui 0.30` · `crossterm 0.29` · `tokio` |
| Backend (API) | Rust · `axum` · `sqlx` · `postgresql` |

The frontend binary (`cli_adventure_sheet v0.5.1`, edition 2024) communicates with the backend over HTTP. The backend API URL is embedded at compile time via the `API_URL` env var (default: `http://localhost:8080/api/v1`).

**Supported Content (current version)**
- Classes: Paladin, Tamer
- Backgrounds: Soldier, Sage
- Species: Human

---

## 2. Full Directory Layout

```
cli_adventure_sheet/
├── Cargo.toml                    # Workspace manifest (v0.5.1, edition 2024)
├── PROJECT_CONTEXT.md            # ← This file
├── README.md
├── API_DOCUMENTATION.md          # Full backend API reference
├── logs/
│   └── tui.log                   # TUI-only tracing output (never stdout)
└── src/
    ├── main.rs                   # Entry point: tokio runtime, tracing-appender init
    ├── models/                   # ★ PURE DOMAIN — zero ratatui imports
    │   ├── mod.rs                # Re-exports all public domain types
    │   ├── app_state.rs          # Enums: Screen, CharacterCreationStep, SheetTab,
    │   │                         #   BuilderState, UiState-adjacent state enums
    │   ├── character.rs          # Character, CharacterDraft, all request/response DTOs
    │   ├── compendium.rs         # Class, Race, Spell, Item, Feat, Background, Subrace
    │   ├── rules.rs              # D&D rule constants and calculations
    │   ├── features/
    │   │   ├── mod.rs
    │   │   ├── feature.rs        # Feature enum (structured feature data)
    │   │   └── interpreter.rs    # Feature string → Feature parser
    │   ├── actions.rs
    │   ├── auth.rs               # AuthResponse, LoginRequest, SignupRequest
    │   └── error.rs              # ApiErrorResponse
    ├── app/                      # Application logic & state management
    │   ├── mod.rs                # App struct, Deref<UiState>, core update loop
    │   ├── ui_state.rs           # ★ UiState — ALL ratatui widget/cursor state
    │   ├── character.rs          # Character helper methods & D&D business logic
    │   ├── levelup.rs            # Level-up prompt queue, LevelUpPrompt, AsiMode
    │   ├── multiclass.rs         # Multiclass management logic
    │   ├── spells.rs             # Spell slot tracking helpers
    │   ├── inventory.rs          # Inventory management helpers
    │   ├── feats.rs              # Feat selection helpers
    │   ├── equipment.rs          # Equipment management helpers
    │   └── events/               # Input event handlers (keyboard dispatch)
    │       ├── mod.rs
    │       ├── builder.rs        # Wizard step transitions & API calls
    │       ├── edit.rs           # EditCharacter screen events
    │       ├── auth.rs           # Login/signup events
    │       ├── character_list.rs # Character list events
    │       └── sheet/            # Per-tab event handlers for the character sheet
    ├── ui/                       # ★ PURE RENDERING — zero D&D math
    │   ├── mod.rs
    │   ├── login.rs              # Auth screen renderer
    │   ├── char_list.rs          # Character list renderer
    │   ├── edit_character.rs     # Edit character screen renderer
    │   ├── builder/              # 5-Step Character Creation Wizard UI
    │   │   ├── mod.rs            # Step router + stepper header renderer
    │   │   ├── step_class.rs     # Step 1: Class selection
    │   │   ├── step_background.rs# Step 2: Background + identity
    │   │   ├── step_race.rs      # Step 3: Species/lineage selection
    │   │   ├── step_abilities.rs # Step 4: Ability score entry
    │   │   └── step_equipment.rs # Step 5: Equipment choice
    │   └── sheet/                # Character sheet tab renderers
    │       ├── mod.rs            # Tab router
    │       ├── core_stats.rs
    │       ├── skills.rs
    │       ├── actions.rs
    │       ├── inventory.rs
    │       ├── spells.rs
    │       ├── features.rs
    │       ├── proficiency.rs
    │       ├── background_info.rs
    │       └── notes.rs
    ├── client/                   # HTTP client (reqwest wrappers)
    │   ├── mod.rs                # ApiClient struct, auth token management
    │   ├── auth.rs               # login / signup endpoints
    │   ├── character.rs          # character CRUD + wizard draft endpoints
    │   ├── compendium.rs         # compendium fetch endpoints
    │   └── admin.rs              # admin endpoints
    └── utils/                    # Standalone utility helpers
        ├── mod.rs
        ├── storage.rs            # Local credential/token persistence
        ├── weapon_properties.rs  # Weapon property string helpers
        └── weapon_mastery.rs     # Weapon mastery rule helpers
```

---

## 3. The 5-Step Character Creation Wizard

The wizard is driven by `CharacterCreationStep` (in `models/app_state.rs`) and rendered through `src/ui/builder/`. State lives in `BuilderState` (also in `models/app_state.rs`).

### Step 1 — Class (`step_class.rs`)
- Split-pane: class list (left) + Level Progression table (right)
- `TableState` (`progression_table_state`) scrolls the 1–20 level table
- `+` / `-` keys adjust `BuilderState.level` (clamped 1–20)
- At level 3+: `show_subclass_modal = true` opens subclass picker overlay
- `subclass_list_state: ListState` drives the modal

### Step 2 — Background (`step_background.rs`)
- Character identity: Name, Alignment
- Background selection with pop-up modals for Origin Feats
- `show_feat_modal` flag + `feat_list_state` drive the feat picker overlay
- Background ability bonuses (`bg_ability_bonuses: [i32; 6]`) from XPHB rules
- Multi-step ability choice flow: `bg_ability_choices`, `bg_ability_step`, `bg_ability_focus`

### Step 3 — Species (`step_race.rs`)
- Nested lineage: top-level species → lineage (e.g. Elf → Drow/High/Wood)
- `show_lineage_menu` flag + `lineage_list_state` for nested dropdown
- Level 1/3/5 trait unlock progression table display
- `species_id`, `lineage_id` / `subrace_id` stored in `BuilderState`

### Step 4 — Abilities (`step_abilities.rs`)
- Three methods: `AbilityMethod::{StandardArray, PointBuy, Manual}`
- Direct numeric entry (valid range 3–20 for Manual)
- Live modifier display: `floor((score - 10) / 2)` — **calculated in `models/` not UI**
- `ability_scores: [i32; 6]`, `ability_cursor: usize` in `BuilderState`

### Step 5 — Equipment (`step_equipment.rs`)
- Side-by-side comparison: **Option A** (starting equipment bundle) vs **Option B** (starting gold)
- `equipment_option: Option<usize>` — `Some(0)` = bundle, `Some(1)` = gold
- `equipment_choices: Vec<usize>` for individual item selections within the bundle
- `starting_gold: Option<i32>` if the user picks the gold option

---

## 4. Key Data Structures

### `BuilderState` (in `models/app_state.rs`)
Owns all wizard step data. Lives on `App.builder`. Contains both data fields (selections, ability scores, level) and UI widget state (ListState, TableState, modal flags). This is a deliberate design: builder step state is tightly coupled to its rendering, so UI state lives here rather than in `UiState`.

### `UiState` (in `app/ui_state.rs`)
Owns all post-creation navigation and widget state: sheet tab selection, modal open/close flags, list/table cursors for the character sheet, ASI overlays, picker overlays, and edit-character screen state. Zero D&D calculations here.

### `CharacterDraft` (re-exported from `models/character.rs`)
Serializable raw character data used for API sync across wizard steps.

### `App` (in `app/mod.rs`)
Top-level struct. Implements `Deref<Target = UiState>` (legacy shim from refactor). Holds `builder: BuilderState`, the HTTP client, loaded compendium data, and the active character.

---

## 5. Architectural Laws — MUST NOT VIOLATE

### Law 1: Clean Domain Isolation
```
models/  →  Pure Rust D&D logic
            ✗ ZERO imports from ratatui
            ✓ May use serde, uuid, chrono
```

### Law 2: Pure UI Rendering
```
ui/      →  Pure rendering and user feedback
            ✗ ZERO D&D rule math (no modifier calculations, no HP math)
            ✓ Must call Character / BuilderState methods for computed values
```

### Law 3: State Separation
```
BuilderState  →  Raw wizard selections + wizard widget state
UiState       →  Post-creation navigation + sheet widget state
              →  Never merge these back into a God Struct
```

### Law 4: Observability
```
Backend   →  tracing::info_span! to stdout (JSON importers, wizard handlers)
Frontend  →  tracing-appender to logs/tui.log ONLY
             ✗ NEVER println!() or eprintln!() in TUI code (garbles ratatui)
```

### Law 5: Hybrid Persistence
```
Current step  →  Works offline (in-memory BuilderState)
Step advance  →  Requires online: POST/PUT/PATCH to backend API via client/
Step resume   →  Requires online: GET draft from backend API
```

---

## 6. Testing Safety Net

All tests must pass after every change: `cargo test`

| Suite | Count | Location |
|---|---|---|
| Backend D&D math/clamping/ASI validation | 16 unit tests | `models/` |
| Backend wizard happy path | 1 integration test | `test_wizard_happy_path` |
| Frontend feature string/rule interpreters | 11 unit tests | `models/features/interpreter.rs` |

**Total: 28 tests. All must remain green.**

---

## 7. Development Workflow Rules

1. **Surgical edits only** — Never scan the full workspace or read all `.rs` files. Work on specific modules as needed.
2. **Verify after changes** — Run `cargo test` after every code modification.
3. **Token efficiency** — Read only the files relevant to the current task.
4. **No God Structs** — The 155-field `App` struct was already split. Do not re-consolidate state.
5. **Dependency discipline** — New dependencies must use pinned/exact versions in `Cargo.toml`.

---

## 8. Key Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `ratatui` | 0.30.0 | TUI framework |
| `crossterm` | 0.29 | Terminal backend |
| `tokio` | 1 (rt-multi-thread, macros) | Async runtime |
| `reqwest` | 0.12 (json, rustls-tls) | HTTP client |
| `serde` / `serde_json` | 1.0.228 / 1 | Serialization |
| `uuid` | 1 (serde, v4) | Character/draft IDs |
| `chrono` | 0.4 (serde) | Timestamps |
| `thiserror` | 2 | Error types |
| `tracing` | 0.1 | Structured logging |
| `tracing-appender` | 0.2 | File-based logging (TUI → tui.log) |
| `tracing-subscriber` | 0.3 | Log subscriber setup |
| `directories` | 6.0.0 | Platform config/data dirs |
| `once_cell` | 1.20 | Lazy static initialization |

---

## 9. Screen Navigation Flow

```
Login
  └─▶ CharacterList
        ├─▶ CharacterBuilder (new character → 5-step wizard)
        │     Step 1: Class
        │     Step 2: Background
        │     Step 3: Species
        │     Step 4: Abilities
        │     Step 5: Equipment
        │     └─▶ CharacterSheet (on completion)
        ├─▶ CharacterSheet (load existing)
        │     Tabs: CoreStats | Skills | Actions | Inventory |
        │           Spells | Features | Proficiency | Background | Notes
        └─▶ EditCharacter (quick edit from list or sheet)
```

---

*This file is the authoritative session context. Update it whenever a major architectural decision is made or a new module is added.*
