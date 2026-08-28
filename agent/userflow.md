# User Flow & Interaction Specification — CLI Adventure Sheet

Dokumen ini menjelaskan alur pengoperasian aplikasi langkah demi langkah dari perspektif pengguna dan bagaimana antarmuka frontend merespons setiap interaksi di 4 target platform (`cli`, `desktop`, `web`, `mobile`).

---

## 1. Alur Penggunaan Utama (Happy Path)

### Langkah 1: Peluncuran Aplikasi & Otentikasi
1. Pengguna membuka aplikasi CLI Adventure Sheet sesuai platform pilihan:
   - **CLI**: Mengatasi perintah `cargo run -p cli` atau mengeksekusi binary `cli`.
   - **Desktop / Web / Mobile**: Mengklik launcher aplikasi atau membuka URL web (menggunakan Dioxus RSX).
2. Layar **Login / Register** ditampilkan jika belum ada token sesi lokal yang tersimpan.
3. Pengguna memasukkan kredensial (username/password) dan menekan tombol *Submit* (atau `Enter` di CLI).
4. Client mengirim permintaan autentikasi via `crates/services/src/api_client.rs`.
5. Jika berhasil, token JWT disimpan di storage lokal via `crates/services/src/storage.rs`, dan pengguna diarahkan ke **Character List Screen**.

### Langkah 2: Pemilihan Karakter & Inisiasi
Di layar **Character List Screen**, pengguna memiliki dua jalur:
- **Opsi A (Pilih Karakter Ada)**: Memilih karakter yang tersimpan → Fetch data `GET /api/v1/characters/{id}` dan `GET /api/v1/characters/{id}/manifest` via `crates/services` → Masuk ke **Tactical Character Dashboard**.
- **Opsi B (Buat Karakter Baru)**: Menekan tombol "Create New Character" (atau shortcut `n` di CLI) → Masuk ke **5-Step Character Creation Wizard**.

---

## 2. Alur 5-Step Character Creation Wizard

Wizard dipandu oleh status stepper di bagian atas antarmuka:

```
[ Step 1: Class ] ➔ [ Step 2: Background ] ➔ [ Step 3: Species ] ➔ [ Step 4: Abilities ] ➔ [ Step 5: Equipment ]
```

### A. Interaksi GUI (Desktop / Web / Mobile via Dioxus RSX)
- Navigasi menggunakan mouse / touch input / keyboard tab.
- Modal overlay terbuka secara pop-up dengan efek glassmorphism & animasi transisi.
- Penyesuaian nilai statistik dan pilihan dilakukan via tombol interactive GUI.

### B. Interaksi CLI (Terminal Shell via Ratatui Widgets)
- **Navigasi Keyboard Murni**:
  - `Tab` / `Shift+Tab` atau `←` / `→`: berpindah antar pane/elemen di dalam layar.
  - `↑` / `↓`: memilih opsi dalam daftar (Class, Species, Feat, Equipment).
  - `+` / `-` atau `k` / `j`: menyesuaikan level karakter atau nilai Ability Score (pada Step 4 Manual).
  - `Enter` / `Space`: mengonfirmasi pilihan atau membuka modal dialog TUI.
  - `Esc`: menutup modal TUI atau kembali ke langkah sebelumnya.
- **Subclass & Feat Modals di CLI**: Ditampilkan sebagai popup centered box berbatas garis ganda (`Block::default().borders(Borders::ALL)`) yang menangkap fokus keyboard sampai ditutup dengan `Enter` atau `Esc`.

---

## 3. Alur Tactical Character Dashboard & Decision Slots

Setelah karakter dimuat, **Tactical Character Dashboard** menjadi tampilan utama gameplay harian.

### A. Navigasi Tab Dashboard
Dashboard memiliki header statistik vital (HP, AC, Initiative, Speed, Level) dan tab navigasi (`Core Stats`, `Skills`, `Actions`, `Inventory`, `Spells`, `Features / Manifest`, `Notes`).
- **Di GUI**: Klik tab atau usap di mobile.
- **Di CLI**: Menggunakan tombol `1` sampai `8` atau `h` / `l` untuk berpindah tab dengan cepat.

### B. Interaksi dengan Decision Slot (Tab Features / Progression View)

```
[ Level 1: Origin Feat ] -> State: COMPLETED
[ Level 4: ASI / Feat   ] -> State: PENDING
[ Level 8: ASI / Feat   ] -> State: LOCKED
```

#### 1. Memproses Pending Slot
- **Visual GUI (RSX)**: Slot Level 4 tampil dengan **pulsating gold border** & glowing aura. Pengguna mengeklik slot tersebut.
- **Visual CLI (Ratatui)**: Slot Level 4 tampil dengan warna **kuning terang (Yellow ANSI)** + teks `[◆ PENDING] Level 4: Ability Score Improvement / Feat`. Pengguna menggunakan `↑`/`↓` untuk menyorot slot lalu menekan `Enter`.
- **Picker Dialog**:
  - Di GUI: Pop-up glassmorphism picker overlay.
  - Di CLI: Centered list dialog TUI.
- **Konfirmasi**: Pengguna memilih opsi (misal Feat "War Caster") dan memilih "Confirm".
- **API Call**: Client mengirim `PUT /api/v1/characters/{id}/manifest/slot/{slot_id}` via `crates/services`.
- **Transisi State**: Setelah backend membalas `200 OK`, state slot berubah menjadi **Completed**:
  - Di GUI: Berubah menjadi **emerald/cyan glow**.
  - Di CLI: Berubah menjadi warna **hijau/cyan** + teks `[✓ COMPLETED] War Caster`.

#### 2. Meninjau / Mengedit Completed Slot
- Pengguna mengeklik (GUI) atau menekan `Enter` pada slot hijau (CLI) untuk membuka modal detail/edit pilihan.

---

## 4. Paritas State & Pengalaman Lintas Platform (Cross-Platform Parity)

Meskipun mekanisme antarmuka dan visualisasi berbeda antara GUI (Dioxus RSX) dan CLI (Ratatui TUI), **state underlying data Progression Manifest-nya identik 100%**:
- Jika seorang pengguna mengisi Pending Slot Level 4 melalui aplikasi **CLI Terminal** di laptop kantor, kemudian membuka aplikasi **Desktop GUI** atau **Web** untuk karakter yang sama:
  - Progression Manifest yang di-fetch dari backend Axum via `crates/services` adalah payload JSON yang persis sama.
  - Slot Level 4 yang tadinya diisi via CLI akan langsung dirender sebagai **Completed Slot (Emerald Glow)** di GUI Desktop/Web.

---

## 5. Alur Penanganan Error & Edge Cases

### Edge Case 1: Klik / Enter pada Locked Slot
- **Aksi Pengguna**: Pengguna mengeklik (GUI) atau menekan `Enter` (CLI) pada slot level 8 sedangkan karakter masih Level 4.
- **Respons Frontend**:
  - Event handler mengecek `SlotState::Locked`.
  - Frontend **DILARANG** memanggil API backend.
  - Di GUI: Efek visual *shake* ringan dan toast notification `"Slot terkunci. Membutuhkan Karakter Level 8"`.
  - Di CLI: Bell/beep terminal atau pesan status bar warna merah di footer: `[Err] Slot terkunci (Butuh Lvl 8)`.

### Edge Case 2: Kegagalan Koneksi / API Error saat Submit Pilihan
- **Respons Frontend**:
  - Slot **TIDAK BOLEH** berubah menjadi Completed.
  - Picker dialog tetap terbuka.
  - Pesan error dari backend (`ApiErrorResponse.message`) ditampilkan di dialog/status line.
  - Slot tetap dalam state **Pending** di kedua renderer sehingga pengguna dapat melakukan *retry*.

### Edge Case 3: Payload Deserialization Error (Manifest Inkompatibel)
- **Respons Frontend**:
  - Error deserialisasi ditangkap di layer `crates/core` / `crates/services`.
  - Aplikasi tidak boleh crash (*panic*).
  - Component / Widget tab features merender *Error Fallback View* dengan opsi "Reload Manifest", serta mencatat detail error log via `tracing`.
