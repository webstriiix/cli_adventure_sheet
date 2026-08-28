# Aturan Keras & Konvensi Pengodean — CLI Adventure Sheet

Dokumen ini mendaftar seluruh aturan keras (*hard rules*) yang **WAJIB** dipatuhi oleh setiap developer atau AI agent yang bekerja pada Cargo Workspace CLI Adventure Sheet.

---

## 1. ATURAN EKSPLISIT ARSITEKTUR WORKSPACE (MUST NOT VIOLATE)

### Rule 1: Isolation of `crates/core`
**JANGAN PERNAH** mengimpor `dioxus`, `ratatui`, `crossterm`, `reqwest`, atau dependensi UI/HTTP apapun ke dalam `crates/core`. `crates/core` harus murni berisi domain logic Rust standar. Jika ada kebutuhan I/O, jaringan, atau penyimpan lokal, letakkan di `crates/services`, bukan `core`.

### Rule 2: Independence of Renderers
`crates/ui` (Dioxus RSX) dan `apps/cli/src/widgets` (Ratatui TUI) adalah **DUA RENDERER INDEPENDEN**. **JANGAN PERNAH** mengimpor `crates/ui` dari `apps/cli`, dan **JANGAN PERNAH** mengimpor `ratatui` atau widget `apps/cli` dari `crates/ui`. Both depend on `core` and `services`, but never on each other.

### Rule 3: Paritas 1:1 Folder & File Renderer
Struktur folder dan nama file di `apps/cli/src/widgets/` **HARUS** mengikuti struktur nama yang sama persis dengan `crates/ui/src/components/` (contoh: `wizard/step_class.rs` wajib ada di kedua sisi, `dashboard/tab_skills.rs` wajib ada di kedua sisi). Tujuannya agar ketika menambah field/slot baru di `core::compendium` atau `core::manifest`, developer/AI agent tahu persis 2 lokasi renderer yang wajib diperbarui.

### Rule 4: Visual Decision Slot 3-State Equivalence
Decision Slot 3-state (Pending, Completed, Locked) **WAJIB** direpresentasikan di kedua renderer sesuai kapabilitas medium masing-masing:
- **RSX (Web/Desktop/Mobile)**:
  - Pending: `border-amber-500 animate-pulse shadow-[0_0_15px_rgba(245,158,11,0.5)]`
  - Completed: `border-emerald-500 shadow-[0_0_15px_rgba(16,185,129,0.5)]`
  - Locked: `bg-zinc-900/40 border-zinc-800 text-zinc-600 opacity-60`
- **Ratatui (Terminal CLI)**:
  - Pending: `Color::Yellow` + `Modifier::BOLD` + simbol `[◆ PENDING]` (Clickable/Selectable via Enter).
  - Completed: `Color::Green` / `Color::Cyan` + simbol `[✓ COMPLETED]` + ringkasan data.
  - Locked: `Color::DarkGray` / Dimmed + simbol `[🔒 LOCKED]` + Non-focusable.

### Rule 5: Thin App Launchers
Entry point `apps/*/src/main.rs` untuk ke-4 target platform (`cli`, `desktop`, `web`, `mobile`) **HARUS TIPIS** (hanya berisi inisialisasi konfigurasi platform spesifik dan penjalangan runner). **JANGAN PERNAH** menaruh logika bisnis D&D atau parsing API di file `main.rs`.

### Rule 6: No Hybrid Binaries
**TIDAK ADA** binary "hybrid". Ke-4 target di atas adalah artefak build terpisah dengan proses kompilasi dan rilis sendiri (`cargo run -p cli`, `dx serve --platform desktop`, `dx serve --platform web`, `dx build --platform mobile`). Logika aplikasi dijamin konsisten oleh `crates/core` dan `crates/services`, bukan oleh binary tunggal.

---

## 2. CHECKLIST REVIEW PR & PARITAS RENDERER

Setiap Pull Request (PR) atau perubahan kode yang menyentuh `crates/ui/src/components/` **ATAU** `apps/cli/src/widgets/` **WAJIB** menyertakan pembaruan di sisi padanannya, kecuali ada alasan yang terdokumentasi eksplisit mengapa fitur tersebut tidak applicable (misalnya fitur grafis web-only).

```
Checklist PR:
[ ] Apakah ada penambahan/perubahan field UI di crates/ui/src/components/?
[ ] Jika YA, apakah komponen padanannya di apps/cli/src/widgets/ sudah di-update?
[ ] Apakah cargo test --workspace tetap hijau?
```

---

## 3. ATURAN SINKRONISASI KONTRAK API

> **Aturan Sinkronisasi Wajib:**
> "Setiap kali `API_DOCUMENTATION.md` di-update (hasil salin dari backend), WAJIB dicek terhadap struct/enum di `crates/core` dalam PR yang sama. **JANGAN PERNAH** menganggap update dokumentasi selesai tanpa memverifikasi `crates/core` masih sesuai — response shape yang berubah tanpa update struct akan menyebabkan deserialize gagal secara diam-diam di **SEMUA 4 target** (`cli`/`desktop`/`web`/`mobile`) sekaligus."

---

## 4. LARANGAN KALKULASI D&D DI UI (RETAINED)

### Aturan
**JANGAN PERNAH** menghitung ability modifier, Hit Points (HP), Armor Class (AC), Difficulty Class (DC), Spell Save DC, atau Attack Bonus secara langsung di dalam file komponen UI (`crates/ui/src/components/**/*.rs` atau `apps/cli/src/widgets/**/*.rs`).

**SELALU** panggil method dari `crates::core::rules` atau gunakan nilai pre-calculated dari response API backend.

---

## 5. KEPATUHAN CETAK BIRU D&D BEYOND (DATA DENSITY BLUEPRINT)

Sampel HTML/TXT referensi dari D&D Beyond digunakan sebagai cetak biru kepadatan data. Setiap field/statistik yang ada di cetak biru referensi tersebut **WAJIB** direpresentasikan pada komponen UI di **KEDUA RENDERER** (RSX dan Ratatui). **DILARANG** menyederhanakan, menghilangkan, atau menghapus field data tanpa konfirmasi dan dokumentasi eksplisit.

---

## 6. KEAMANAN DASAR & CONVENTION

1. **Jangan Hardcode Secret / Token**: Seluruh autentikasi wajib memuat token dari `crates/services/src/storage.rs`.
2. **Validasi Input Sebelum Network Request**: Validasi kisaran Ability Score (3-20) dan string input di `crates/core` sebelum dikirim via `crates/services`.
3. **Logging via Tracing**: Gunakan `tracing::info!()`, `tracing::warn!()`, atau `tracing::error!()`. Dilarang `println!()` / `eprintln!()` di kode aplikasi.
