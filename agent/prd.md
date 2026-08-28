# Product Requirement Document (PRD) — CLI Adventure Sheet (Frontend)

## 1. Visi & Visi Umum Produk

**CLI Adventure Sheet** adalah aplikasi digital *character sheet manager* untuk permainan meja Dungeons & Dragons edisi ke-5 (D&D 5e / edisi 2024 / XPHB) yang dirancang untuk berjalan di lintas platform (CLI Terminal, Desktop, Web, dan Mobile/Android).

Aplikasi ini mengadopsi arsitektur dua komponen terpisah (Decoupled Clean Architecture):
- **Backend (Axum + Postgres)**: Berperan sebagai "otak" aplikasi yang mengelola aturan game (D&D 5e Rule Engine), memvalidasi pilihan, menghitung Progression Manifest, serta menyimpan statistik karakter secara persisten.
- **Frontend (Cargo Workspace multi-crate, repository ini)**: Berperan sebagai "wajah" aplikasi yang mendukung 4 target platform resmi (`cli`, `desktop`, `web`, `mobile`) dengan logika domain dan komunikasi API terpusat di crate internal `core` dan `services`.

---

## 2. Latar Belakang Arsitektur & Evolusi Multi-Target

### Histori Keputusan Arsitektur
1. **Awalnya CLI-Only (Ratatui)**: Proyek ini bermula sebagai aplikasi antarmuka terminal (TUI) murni berbasis `ratatui` dan `crossterm`.
2. **Masalah Aksesibilitas**: Teman-teman developer dan pengguna non-terminal mengalami kesulitan bernavigasi di environment TUI. Untuk memperluas jangkauan penggunaan, diambil keputusan strategis untuk mengadopsi **Dioxus** guna mendukung target **Web**, **Desktop**, dan **Mobile (Android)**.
3. **CLI Tetap Dipertahankan (Aktif, Bukan Legacy)**: CLI **TIDAK** dibuang atau dianggap legacy. CLI tetap menjadi target resmi yang dikembangkan secara aktif paralel dengan GUI, karena CLI merupakan *daily driver* utama dari lead maintainer proyek ini.
4. **Keputusan Dioxus TUI Deprecated**: Dioxus TUI renderer disengaja **TIDAK DIPAKAI** karena statusnya sudah *deprecated* di upstream Dioxus. Sebagai gantinya, target `apps/cli` dipertahankan menggunakan `ratatui` + `crossterm` sebagai renderer independen yang terpisah dari `crates/ui` Dioxus.

### Target Platform Resmi
1. **CLI (`apps/cli`)**: Terminal UI berbasis `ratatui` + `crossterm` untuk penggunaan cepat & efisien via keyboard.
2. **Desktop (`apps/desktop`)**: Application GUI native via `dioxus` (Wry/WebView) untuk Windows, Linux, macOS.
3. **Web (`apps/web`)**: Web application via `dioxus` (WebAssembly/WASM).
4. **Mobile (`apps/mobile`)**: Native mobile app via `dioxus` (Android/iOS).

---

## 3. Target Pengguna & Masalah yang Diselesaikan

### Target Pengguna
Pemain dan Dungeon Master (DM) D&D 5e yang membutuhkan lembar karakter digital modern yang presisi, cepat, bebas *lag*, dan dapat diakses di berbagai lingkungan kerja: terminal shell bagi power-user, maupun antarmuka grafik di Web, Desktop, dan Mobile.

### Masalah yang Diselesaikan
1. **Inkonsistensi Rule Engine**: Banyak aplikasi sheet digital yang sering salah menghitung aturan edisi 2024 (XPHB). Dengan memisahkan rule engine ke backend dan domain models terisolasi di `crates/core`, frontend dijamin selalu merender statistik yang valid di semua 4 target.
2. **Pengelolaan Progression Karakter yang Rumit**: Karakter D&D 5e memiliki puluhan keputusan perkembangan dari level 1–20 (Ability Score Improvement, Feat, Spell Selection, Weapon Mastery, Subclass). CLI Adventure Sheet menyederhanakan ini melalui mekanik visual **Progression Manifest** dan **Decision Slots**.
3. **Pilihan Antarmuka yang Fleksibel**: Pengguna dapat memilih antarmuka sesuai kenyamanan mereka (Terminal CLI, Web browser, Desktop GUI, atau Smartphone) tanpa kehilangan paritas data atau fitur.

---

## 4. Fitur Pembeda Utama & Mekanik Inti

### A. Progression Manifest
**Progression Manifest** adalah payload data terstruktur dari backend Axum yang mencatat *seluruh* daftar keputusan dan perkembangan karakter dari Level 1 sampai Level 20. Manifest ini mendefinisikan apa saja keputusan yang wajib atau opsional diambil oleh pemain pada setiap tingkatan level (misal: memilih Subclass di Level 3, memilih Origin Feat di Level 1, atau mengambil ASI/Feat di Level 4).

### B. Decision Slots
**Decision Slots** adalah komponen antarmuka di frontend yang memvisualisasikan tiap butir pilihan yang ada di dalam Progression Manifest. Setiap Decision Slot wajib mengimplementasikan **3 State Visual** yang disesuaikan dengan kapabilitas medium masing-masing renderer (RSX/Dioxus vs Ratatui/CLI):

1. **Pending Slot** (Slot Tertunda / Harus Diisi):
   - **Kondisi**: Slot pada level yang sudah/sedang dicapai oleh karakter, namun belum diisi oleh user.
   - **Visual GUI (RSX)**: Garis tepi beranimasi denyut warna emas/kuning (*pulsating gold/yellow border*).
   - **Visual CLI (Ratatui)**: Teks warna kuning terang + indikator simbol (misal `[◆ PENDING]`).
   - **Interaktivitas**: Interaktif sepenuhnya (dapat diklik di GUI atau dipilih via Enter/Space di CLI untuk membuka picker).
2. **Completed Slot** (Slot Selesai):
   - **Kondisi**: Slot yang sudah diisi oleh pemain dan divalidasi oleh backend.
   - **Visual GUI (RSX)**: Efek pendaran (*glow*) warna hijau/cyan magis dengan ringkasan data pilihan.
   - **Visual CLI (Ratatui)**: Teks warna hijau/cyan + simbol centang (misal `[✓ COMPLETED] War Caster`).
   - **Interaktivitas**: Menampilkan detail data pilihan (dapat diedit jika masih dalam window yang diperbolehkan).
3. **Locked Slot** (Slot Terkunci):
   - **Kondisi**: Slot keputusan untuk tingkatan level yang lebih tinggi dari level karakter saat ini (`slot_level > current_level`).
   - **Visual GUI (RSX)**: Berwarna abu-abu gelap/redup (*dark dim*), semi-transparan.
   - **Visual CLI (Ratatui)**: Teks warna abu-abu gelap/dim + simbol gembok (misal `[🔒 LOCKED]`).
   - **Interaktivitas**: Non-interaktif. Menolak semua interaksi di GUI dan non-focusable di CLI.

---

## 5. Estetika Visual & Pengalaman Pengguna Per Renderer

- **GUI (Desktop / Web / Mobile via Dioxus RSX)**: Mengusung gaya **AAA Modern Dark Fantasy RPG** terinspirasi *Baldur's Gate 3* & *Diablo IV* (glassmorphism, metallic/ornate borders, magical glowing aura).
- **CLI (Terminal via Ratatui)**: Mengusung gaya **Rich TUI Dark Fantasy** (panel unicode, perbatasan double/rounded border, styling ANSI RGB 256-color, serta panduan shortcut keyboard yang jelas di footer).

---

## 6. Fitur Utama Wajib (Scope MVP)

1. **5-Step Character Creation Wizard**:
   - Step 1: **Class** (Pemilihan Kelas, level 1–20, Subclass jika level ≥ 3).
   - Step 2: **Background** (Latar belakang, bonus statistik XPHB, Origin Feats, dan identitas).
   - Step 3: **Species** (Spesies/Ras dan Lineage/Subrace beserta trait progression).
   - Step 4: **Abilities** (Alokasi 6 Ability Score via Standard Array, Point Buy, atau Manual).
   - Step 5: **Equipment** (Pemilihan paket perlengkapan awal vs Starting Gold).
   *Paritas wajib diimplementasikan baik di GUI Dioxus maupun CLI Ratatui.*
2. **Tactical Character Dashboard**:
   - Tampilan gameplay utama harian (HP tracker, AC, Initiative, Speed, Spell Slot counter, Actions, Inventory, Features, Notes).
3. **Progression Tracking Lintas Level**:
   - Tampilan visual Progression Manifest dengan rendering Decision Slots 3 state.
4. **Backend Sync**:
   - Komunikasi REST API via `crates/services` ke backend Axum untuk otentikasi, fetch compendium, simpan draft wizard, dan pemutakhiran manifest.

---

## 7. Standar Kepadatan Data (Blueprint D&D Beyond)

Frontend ini menggunakan sampel mentah HTML/TXT dari lembar karakter D&D Beyond sebagai **cetak biru referensi (blueprint)** untuk kepadatan data.
- **Aturan**: Seluruh field statistik yang ada di cetak biru referensi (seperti indikator komponen mantra V/S/M, Weapon Mastery properties, Passive Senses, Saving Throw Proficiencies) **WAJIB** direpresentasikan pada komponen UI di SEMUA renderer (RSX dan CLI widgets).
- Dilarang menyederhanakan, memotong, atau meremove field data tanpa konfirmasi atau dokumentasi eksplisit.

---

## 8. Definisi Selesai (Definition of Done)

### Fase 1: MVP (Minimum Viable Product)
- Cargo workspace terstruktur rapi (`crates/core`, `crates/services`, `crates/ui`, `apps/cli`, `apps/desktop`, `apps/web`, `apps/mobile`).
- Wizard 5-Step dan Tactical Dashboard berfungsi di 4 target platform dengan paritas UI & fitur.
- Decision Slots merender 3 state visual (Pending, Completed, Locked) sesuai kapabilitas renderer masing-masing (RSX CSS glow vs Ratatui ANSI/Unicode).
- Seluruh unit & integration test internal tetap hijau (`cargo test --workspace`).

### Scope Masa Depan (Pasca-MVP)
- Pandangan party real-time / multiplayer session viewer.
- Pembuat konten kustom (*Homebrew Content Builder*).
- Sinkronisasi mode offline (offline queue dengan cache SQLite lokal via `crates/services`).
- Integrasi animasi dadu 3D (*3D Dice Roller*) untuk target Web/Desktop.
