# Directory Layout & Workspace Structure — CLI Adventure Sheet

Dokumen ini memetakan struktur Cargo Workspace multi-crate repositori CLI Adventure Sheet beserta aturan paritas komponen antar-renderer.

---

## 1. Peta Direktori Workspace LENGKAP

```
cli_adventure_sheet/
├── Cargo.toml                              # Workspace root manifest (resolver = "2")
├── README.md                               # Ringkasan proyek & instruksi jalankan per target
├── API_DOCUMENTATION.md                    # Kontrak resmi REST API Backend Axum
├── PROJECT_CONTEXT.md                      # Catatan konteks pengembangan
├── assets/                                 # Asset statis & Tailwind CSS global
│   ├── styles.css                          # Custom Tailwind CSS & glassmorphism rules
│   ├── fonts/                              # Font Dark Fantasy
│   └── icons/                              # SVG icons
├── agent/                                  # Dokumentasi internal AI Agent
│   ├── prd.md                              # Product Requirement Document (Histori & 4 Target)
│   ├── userflow.md                         # Alur interaksi pengguna (GUI & CLI)
│   ├── tech.md                             # Stack teknologi workspace & batasan
│   ├── structure.md                        # Peta folder workspace & 1:1 Parity Table (dokumen ini)
│   ├── rules.md                            # Aturan keras workspace & konvensi
│   └── errors.md                           # Log pencatatan error & insiden
│
├── crates/                                 # ★ INTERNAL WORKSPACE CRATES
│   ├── core/                               # PURE DOMAIN LAYER (Zero UI/HTTP Dependencies)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                      # Re-export modul core domain
│   │       ├── character.rs                # Struct Character, CharacterDraft, Stats DTOs
│   │       ├── manifest.rs                 # Struct ProgressionManifest, DecisionSlot, SlotState
│   │       ├── compendium.rs               # Struct Class, Background, Species, Feat, Spell
│   │       ├── rules.rs                    # Formulasi aturan D&D 5e (XPHB 2024)
│   │       ├── error.rs                    # Domain Error & ApiErrorResponse definitions
│   │       └── features/                   # Feature parsers & interpreters
│   │           ├── mod.rs
│   │           ├── feature.rs
│   │           └── interpreter.rs
│   │
│   ├── services/                           # API CLIENT & STORAGE SERVICE LAYER
│   │   ├── Cargo.toml                      # Depends on core
│   │   └── src/
│   │       ├── lib.rs                      # Service re-exports
│   │       ├── api_client.rs               # HTTP Client Wrapper (Reqwest auth & endpoints)
│   │       ├── manifest_service.rs         # Manifest fetcher & slot update logic
│   │       └── storage.rs                  # Persistence lokal (auth token & session cache)
│   │
│   └── ui/                                 # DIOXUS RSX SHARED GUI COMPONENTS
│       ├── Cargo.toml                      # Depends on core, services, dioxus
│       └── src/
│           ├── lib.rs                      # Component exports
│           ├── theme/                      # Styling tokens & Tailwind CSS helpers
│           │   ├── mod.rs
│           │   ├── colors.rs
│           │   └── styles.rs
│           └── components/                 # RSX Component Tree
│               ├── mod.rs
│               ├── common/                 # Modal, Button, Card RSX
│               ├── decision_slot/          # Decision Slot RSX (3-State)
│               │   ├── mod.rs
│               │   ├── pending.rs          # RSX Gold Pulsating Border
│               │   ├── completed.rs        # RSX Emerald Glow
│               │   └── locked.rs           # RSX Dark Dimmed
│               ├── wizard/                 # 5-Step Creation Wizard RSX
│               │   ├── mod.rs
│               │   ├── step_class.rs
│               │   ├── step_background.rs
│               │   ├── step_species.rs
│               │   ├── step_abilities.rs
│               │   └── step_equipment.rs
│               └── dashboard/              # Tactical Dashboard Tabs RSX
│                   ├── mod.rs
│                   ├── header.rs
│                   ├── tab_core_stats.rs
│                   ├── tab_skills.rs
│                   ├── tab_actions.rs
│                   ├── tab_spells.rs
│                   ├── tab_inventory.rs
│                   ├── tab_features.rs
│                   └── tab_notes.rs
│
└── apps/                                   # ★ TARGET PLATFORM APPLICATIONS
    ├── cli/                                # TERMINAL APP (Ratatui TUI Binary)
    │   ├── Cargo.toml                      # Depends on core, services, ratatui, crossterm
    │   └── src/
    │       ├── main.rs                     # Thin launcher & tokio terminal loop
    │       ├── theme.rs                    # Ratatui Style & ANSI Color definitions
    │       ├── app.rs                      # Terminal app state handler
    │       └── widgets/                    # ★ RATATUI WIDGET TREE (Mirrors crates/ui)
    │           ├── mod.rs
    │           ├── common/
    │           ├── decision_slot/
    │           │   ├── mod.rs
    │           │   ├── pending.rs          # Ratatui Yellow ANSI + [◆ PENDING]
    │           │   ├── completed.rs        # Ratatui Green ANSI + [✓ COMPLETED]
    │           │   └── locked.rs           # Ratatui Dark Gray + [🔒 LOCKED]
    │           ├── wizard/                 # 5-Step Creation Wizard TUI Widgets
    │           │   ├── mod.rs
    │           │   ├── step_class.rs
    │           │   ├── step_background.rs
    │           │   ├── step_species.rs
    │           │   ├── step_abilities.rs
    │           │   └── step_equipment.rs
    │           └── dashboard/              # Tactical Dashboard TUI Widgets
    │               ├── mod.rs
    │               ├── header.rs
    │               ├── tab_core_stats.rs
    │               ├── tab_skills.rs
    │               ├── tab_actions.rs
    │               ├── tab_spells.rs
    │               ├── tab_inventory.rs
    │               ├── tab_features.rs
    │               └── tab_notes.rs
    │
    ├── desktop/                            # DIOXUS DESKTOP APP BINARY
    │   ├── Cargo.toml                      # Feature "desktop"
    │   └── src/main.rs                     # Desktop thin entry point
    │
    ├── web/                                # DIOXUS WEB WASM BINARY
    │   ├── Cargo.toml                      # Feature "web"
    │   └── src/main.rs                     # Web thin entry point
    │
    └── mobile/                             # DIOXUS MOBILE ANDROID BINARY
        ├── Cargo.toml                      # Feature "mobile"
        └── src/main.rs                     # Mobile thin entry point
```

---

## 2. Tabel Pemetaan 1:1 Paritas Komponen UI (RSX vs Ratatui)

Untuk memastikan kelengkapan antarmuka di seluruh target platform, setiap komponen visual GUI di `crates/ui/src/components/` **WAJIB** memiliki komponen padanannya di `apps/cli/src/widgets/`:

| Fitur / Sub-Sistem | Komponen RSX GUI (`crates/ui/src/components/`) | Widget Ratatui TUI (`apps/cli/src/widgets/`) | Status Paritas |
|---|---|---|---|
| **Decision Slot Router** | `decision_slot/mod.rs` | `decision_slot/mod.rs` | WAJIB 1:1 |
| **Slot Pending** | `decision_slot/pending.rs` | `decision_slot/pending.rs` | WAJIB 1:1 |
| **Slot Completed** | `decision_slot/completed.rs` | `decision_slot/completed.rs` | WAJIB 1:1 |
| **Slot Locked** | `decision_slot/locked.rs` | `decision_slot/locked.rs` | WAJIB 1:1 |
| **Wizard Step 1: Class** | `wizard/step_class.rs` | `wizard/step_class.rs` | WAJIB 1:1 |
| **Wizard Step 2: Background** | `wizard/step_background.rs` | `wizard/step_background.rs` | WAJIB 1:1 |
| **Wizard Step 3: Species** | `wizard/step_species.rs` | `wizard/step_species.rs` | WAJIB 1:1 |
| **Wizard Step 4: Abilities** | `wizard/step_abilities.rs` | `wizard/step_abilities.rs` | WAJIB 1:1 |
| **Wizard Step 5: Equipment** | `wizard/step_equipment.rs` | `wizard/step_equipment.rs` | WAJIB 1:1 |
| **Dashboard Header** | `dashboard/header.rs` | `dashboard/header.rs` | WAJIB 1:1 |
| **Dashboard Tab Stats** | `dashboard/tab_core_stats.rs` | `dashboard/tab_core_stats.rs` | WAJIB 1:1 |
| **Dashboard Tab Skills** | `dashboard/tab_skills.rs` | `dashboard/tab_skills.rs` | WAJIB 1:1 |
| **Dashboard Tab Actions** | `dashboard/tab_actions.rs` | `dashboard/tab_actions.rs` | WAJIB 1:1 |
| **Dashboard Tab Spells** | `dashboard/tab_spells.rs` | `dashboard/tab_spells.rs` | WAJIB 1:1 |
| **Dashboard Tab Inventory**| `dashboard/tab_inventory.rs` | `dashboard/tab_inventory.rs` | WAJIB 1:1 |
| **Dashboard Tab Features** | `dashboard/tab_features.rs` | `dashboard/tab_features.rs` | WAJIB 1:1 |
| **Dashboard Tab Notes** | `dashboard/tab_notes.rs` | `dashboard/tab_notes.rs` | WAJIB 1:1 |

---

## 3. Aturan Penempatan File Baru (Location Rules)

1. **Logika Domain / Model Data / Formulasi D&D Baru**:
   - Taruh di `crates/core/src/`. Dilarang ada kode Dioxus, Ratatui, atau HTTP di crate ini.
2. **Panggilan API / Persistence / Storage Baru**:
   - Taruh di `crates/services/src/`.
3. **Komponen Visual GUI Baru (Web/Desktop/Mobile)**:
   - Taruh di `crates/ui/src/components/`.
   - **WAJIB SEGERA MEMBUAT WIDGET PADANANNYA** di `apps/cli/src/widgets/` untuk menjaga paritas.
4. **Widget Visual Terminal Baru (CLI)**:
   - Taruh di `apps/cli/src/widgets/`.
   - **WAJIB SEGERA MEMBUAT KOMPONEN PADANANNYA** di `crates/ui/src/components/`.
5. **Entry Point App (`apps/*/src/main.rs`)**:
   - Harus **TIPIS** (kurang dari 50 baris kode). Hanya berisi inisialisasi konfigurasi platform dan pemanggilan runner. Dilarang menaruh logika bisnis di `main.rs`.
