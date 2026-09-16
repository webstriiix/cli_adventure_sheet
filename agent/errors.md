# Register Error & Incident Log — CLI Adventure Sheet

Dokumen ini digunakan untuk mencatat error, bug arsitektur, dan regression yang ditemukan selama pengembangan frontend CLI Adventure Sheet. Setiap kejadian harus didokumentasikan agar tidak terulang dan untuk menjadi referensi bagi perbaikan sistem di masa mendatang.

---

## Format Pencatatan Error Baru:

Setiap entry error harus mengikuti format di bawah ini:

```markdown
## [YYYY-MM-DD] Judul Singkat Error
**Gejala:** (Deskripsi singkat masalah yang terlihat di UI/Console)
**Root Cause:** (Analisis penyebab mendasar masalah)
**Fix:** (Solusi yang diterapkan atau rencana perbaikan)
**Prevention Rule:** (Rujuk ke aturan di agent/rules.md jika relevan)
```

---

## Daftar Kejadian Error & Bug

## [2026-08-28] Migrasi ke Multi-Crate Workspace (4 Target)
**Gejala:** Konteks: Proyek awalnya CLI-only, diperluas ke Web/Desktop/Mobile via Dioxus. Bukan error, tapi keputusan arsitektur besar yang perlu dicatat sebagai referensi historis untuk AI agent di masa depan.
**Root Cause:** Eksplorasi kebutuhan multi-platform dan masalah aksesibilitas terminal untuk developer non-CLI, sementara CLI tetap dibutuhkan sebagai daily driver utama.
**Fix:** Split struktur repo menjadi Cargo Workspace (`crates/core`, `crates/services`, `crates/ui` + `apps/cli`, `apps/desktop`, `apps/web`, `apps/mobile`). Dioxus TUI TIDAK dipakai (deprecated), memakai `ratatui` + `crossterm` untuk `apps/cli`.
**Prevention Rule:** Semua developer/AI agent WAJIB membaca `structure.md` dan `rules.md` sebelum menambah komponen baru, karena ada kewajiban paritas 1:1 antara renderer RSX dan Ratatui.

## [2026-08-27] Dummy Entry: Calculation Leak inside Component RSX
**Gejala:** Ability Modifier di `tab_core_stats.rs` menampilkan hasil perhitungan yang salah ketika Ability Score berubah.
**Root Cause:** Komponen UI menghitung modifier secara manual dengan `(score - 10) / 2` alih-alih menggunakan method `AbilityScore::modifier()` dari domain model `src/models/`.
**Fix:** Mengganti ekspresi kalkulasi di RSX dengan panggilan method domain model `character.ability_score.modifier()`.
**Prevention Rule:** Aturan 1 di `agent/rules.md` (Larangan kalkulasi D&D di komponen UI).
