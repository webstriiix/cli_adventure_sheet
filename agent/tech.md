# Technical Stack & Architecture Guidelines — CLI Adventure Sheet

Dokumen ini mendefinisikan seluruh pustaka, arsitektur perangkat lunak, standar pengelolaan state, serta batasan arsitektur pada Cargo Workspace multi-crate frontend CLI Adventure Sheet.

---

## 1. Cargo Workspace Architecture

Repositori frontend ini terorganisasi sebagai **Cargo Workspace** (dengan `resolver = "2"` di `Cargo.toml` root) untuk mendukung 4 target platform dengan pembagian tanggung jawab yang bersih:

```
cli_adventure_sheet/
├── crates/
│   ├── core/          -> Domain logic & D&D 5e models. (ZERO Dioxus/Ratatui/reqwest dependencies)
│   ├── services/      -> API client (reqwest), HTTP DTOs, & Local storage. (Depends on core)
│   └── ui/            -> Shared Dioxus RSX components. (Depends on core & services)
└── apps/
    ├── cli/           -> Ratatui TUI Binary. (Depends on core & services; NO ui dependency)
    ├── desktop/       -> Dioxus Desktop Binary. (Depends on core, services, & ui)
    ├── web/           -> Dioxus Web WASM Binary. (Depends on core, services, & ui)
    └── mobile/        -> Dioxus Mobile Android Binary. (Depends on core, services, & ui)
```

---

## 2. Core Technology Stack Per Target

| Komponen / Target | Pustaka Utama | Keterangan / Versi |
|---|---|---|
| **Domain Models (`crates/core`)** | **Rust** (Pure Domain) | `serde`, `uuid`, `chrono`. ZERO UI/HTTP dependencies. |
| **Service Layer (`crates/services`)** | **reqwest**, **directories** | `reqwest v0.12` (`json`, `rustls-tls`), token persistence. |
| **Shared GUI (`crates/ui`)** | **Dioxus v0.6**, **Tailwind CSS** | Shared RSX components untuk GUI targets. |
| **App: CLI (`apps/cli`)** | **ratatui 0.30**, **crossterm 0.29**, **clap** | Terminal UI renderer murni. |
| **App: Desktop (`apps/desktop`)** | **dioxus** (`desktop` feature) | Desktop native window via Wry/WebView. |
| **App: Web (`apps/web`)** | **dioxus** (`web` feature) | WebAssembly (WASM) web application. |
| **App: Mobile (`apps/mobile`)** | **dioxus** (`mobile` feature) | Mobile Android APK / NDK build. |

> **CATATAN EKSPLISIT RENDEERER TERMINAL:**
> **Dioxus TUI renderer TIDAK DIPAKAI** karena statusnya sudah *deprecated* di upstream Dioxus. `apps/cli` menggunakan `ratatui` + `crossterm` sebagai renderer TUI independen untuk menjamin stabilitas terminal UI.

---

## 3. Styling Standards & Conventions

### GUI Targets (`crates/ui` + `apps/{desktop,web,mobile}`)
- **Tailwind CSS via RSX Class Strings**: Utility classes langsung pada atribut `class` RSX Dioxus.
- **Single Source CSS**: `assets/styles.css` mendefinisikan efek glassmorphism, metallic borders, dan custom keyframes glow.
- **Theme Colors**: Slate-950 background, Amber/Gold glow untuk Pending Slot, Emerald/Cyan glow untuk Completed Slot, Zinc Dark untuk Locked Slot.

### CLI Target (`apps/cli`)
- **Ratatui Theme (`apps/cli/src/theme.rs`)**: Menggunakan `ratatui::style::{Style, Color, Modifier}`.
- **Visual Encoding Equivalence**:
  - Pending Slot: `Color::Yellow` / `Color::Rgb(245, 158, 11)` + Modifier `BOLD` + simbol `[◆ PENDING]`.
  - Completed Slot: `Color::Green` / `Color::Cyan` + simbol `[✓ COMPLETED]`.
  - Locked Slot: `Color::DarkGray` + Dimmed + simbol `[🔒 LOCKED]`.

---

## 4. State Management Approach

- **Domain State (`crates/core`)**: Imutabel / murni data struct (`Character`, `ProgressionManifest`, `DecisionSlot`).
- **GUI Apps (`crates/ui` via Dioxus)**: Signals & Context API (`Signal<Character>`, `use_context_provider`).
- **CLI App (`apps/cli` via Ratatui)**: App State Struct (`App`) dipadu event loop `tokio` murni yang menangkap `crossterm::event::Event::Key`.

---

## 5. Kontrak API (API Contract & Synchronization)

1. **`API_DOCUMENTATION.md` sebagai Source of Truth**:
   - File `API_DOCUMENTATION.md` di root repo adalah dokumentasi resmi kontrak REST API antara backend Axum dan frontend ini.
   - File ini ditulis dan diperbarui oleh tim backend, lalu disalin ke repo frontend setiap kali ada endpoint baru atau perubahan struktur response (*response shape*).
2. **Representasi Presisi di `crates/core` & `crates/services`**:
   - `crates/core` dan `crates/services` **WAJIB** menjadi representasi Rust yang akurat dari kontrak API tersebut (struct request/response DTO, enum status, error payload).
   - Jika `API_DOCUMENTATION.md` diperbarui, `crates/core` harus diperiksa dan disesuaikan pada Pull Request (PR) yang sama — **TIDAK BOLEH TERPISAH**.
3. **Risiko Drift Kontrak**:
   - Karena ada 4 target platform (`cli`, `desktop`, `web`, `mobile`) yang semuanya bergantung pada `crates/core` dan `crates/services` untuk komunikasi API, perubahan struktur JSON backend yang tidak disinkronkan ke `crates/core` akan menyebabkan kegagalan deserialisasi (`serde_json::Error`) secara bersamaan di **SEMUA 4 TARGET TARGET sekaligus**.

---

## 6. Library & Pola yang DILARANG (Forbidden Practices)

1. **DILARANG Impor `dioxus` di `crates/core` atau `apps/cli`**: `crates/core` harus murni domain logic tanpa kerangka UI. `apps/cli` murni menggunakan Ratatui.
2. **DILARANG Impor `ratatui` atau `crossterm` di `crates/ui`**: `crates/ui` murni berisi komponen Dioxus RSX untuk target GUI.
3. **DILARANG Pakai Crate Kalkulasi D&D 5e Pihak Ketiga**: Seluruh kalkulasi D&D 5e XPHB adalah wewenang backend Axum atau modul teruji di `crates/core/src/rules.rs`.
4. **DILARANG Direct Network Request di UI Components**: Panggilan HTTP wajib melalui `crates/services/src/api_client.rs`.
5. **DILARANG Membuat Binary "Hybrid"**: Ke-4 app (`cli`, `desktop`, `web`, `mobile`) adalah artefak build terpisah dengan proses kompilasi sendiri (`cargo run -p cli`, `dx serve --platform desktop`, dll.). Logic disatukan oleh `core` dan `services`, bukan oleh binary tunggal.
