# Changelog

Semua perubahan penting pada ArsipDok didokumentasikan di file ini.

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), dan proyek ini mengikuti [Semantic Versioning](https://semver.org/lang/id/).

---

## [0.1.0] — 2026-03-15

Rilis perdana (MVP) — ArsipDok siap digunakan untuk sortir & transfer dokumentasi foto/video berbasis metadata.

### Added

#### Fitur Inti
- **Scan sumber rekursif** — pemindaian folder foto/video dengan filter ekstensi cerdas:
  - Foto: JPG, JPEG, PNG, HEIC, HEIF, WEBP, TIF, TIFF, CR2, CR3, NEF, ARW, DNG
  - Video: MP4, MOV, M4V, AVI, MKV, MTS, M2TS, 3GP, WMV, FLV, WEBM, MPG, MPEG
- **Ekstraksi metadata tanggal** dengan hierarki prioritas:
  - EXIF DateTimeOriginal (foto) via `kamadak-exif`
  - QuickTime/MP4 `mvhd` creation time (video)
  - File creation time (fallback)
  - File modified time (fallback terakhir)
- **Planner multi-tujuan** — satu file dapat ditransfer ke banyak folder tujuan sekaligus
- **Penamaan folder otomatis** format `MM-DD_Nama Kegiatan[_Suffix]` dengan sanitasi Windows
- **Engine transfer** dengan buffer streaming 4 MB, mode Copy & Move
- **Verifikasi integritas** pasca-transfer:
  - Size + Mtime (default) — cepat
  - BLAKE3 — hash kriptografis penuh (opsional)
- **Safety mode Move** — file sumber hanya dihapus setelah semua tujuan terverifikasi
- **Penanganan duplikat** — deteksi file identik (skip) dan collision nama (suffix `_1`, `_2`)
- **Log audit CSV** per sesi di `%APPDATA%\ArsipDok\logs\`
- **Ekspor log** ke lokasi pilihan user
- **Pembatalan transfer** instan via `AtomicBool`
- **Cek ruang disk** via FFI `GetDiskFreeSpaceExW` (Windows, tanpa dependensi tambahan)

#### Antarmuka (UI/UX)
- **Welcome screen** — tampilan selamat datang sebagai state awal aplikasi
- **State machine** — alur `welcome → kosong → memindai → siap → memindahkan → selesai/ada_masalah`
- **Theme toggle 3-state** — Gelap / Terang / Ikut Sistem di title bar
- **Layout responsif** — mendukung Windows Snap 2-sisi (lebar minimum **620px**), stack vertical di bawah **860px**
- **Preview table** — lihat file, seleksi massal, filter per status
- **Override tanggal manual** — per file atau massal, dengan indikator sumber `manual`
- **Indikator "Perlu Review"** untuk file dengan tanggal fallback
- **Progress panel** — progres per tujuan, real-time stats, file sedang diproses
- **Ringkasan hasil** — berhasil / duplikat / error / durasi + akses log
- **Settings** — mode transfer, verifikasi, workers, konfirmasi, bahasa

#### Developer & Build
- **Arsitektur Tauri 2 + Svelte 5 (Runes) + Rust**
- **IPC API** — 12 command (get_version, load_settings, save_settings, scan_sources, extract_metadata, build_plan, start_transfer, cancel_transfer, export_log, check_disk_space, validate_destination, open_path) + 2 event (`transfer://progress`, `transfer://item`)
- **Unit + E2E test** — pipeline copy/move/blake3/duplikat (20+ tes)
- **NSIS installer** — `ArsipDok_0.1.0_x64-setup.exe` (1.73 MB)
  - SHA-256: `2BA095C527750BE4FB416078DA40B681C5258C5369F74AAED4D902BDDD7ED46C`
- **Dokumentasi lengkap** — README, PRD, USER_GUIDE, DEVELOPER, API, ARCHITECTURE, TROUBLESHOOTING

### Technical Details
- Rust edition 2021, target `x86_64-pc-windows-msvc`
- Dependensi utama: `tauri` 2, `walkdir`, `kamadak-exif`, `blake3`, `chrono`, `serde`, `parking_lot`, `regex`, `once_cell`
- Settings schema version 2

### Known Issues
- Verifikasi `SizeMtime` tidak mendeteksi korupsi bit-level — gunakan BLAKE3 untuk arsip kritis
- Belum ada fitur resume setelah transfer terputus (file yang sudah ada di-skip sebagai duplicate saat diulang)
- Belum ada auto-detect SD card — sumber harus ditambah manual
- Metadata video terbatas pada format MP4/MOV (`mvhd` atom); format lain fallback ke filesystem time
- Installer belum code-signed (SmartScreen warning pada instalasi pertama)
- Platform: Windows 10/11 x64 saja
- Belum ada thumbnail preview di tabel

### Roadmap
- [ ] Auto-import saat SD card terdeteksi
- [ ] Resume transfer setelah interrupt
- [ ] Ekspor log ke PDF/Excel
- [ ] Thumbnail preview
- [ ] Dukungan macOS/Linux
- [ ] Parsing metadata video tambahan (MKV, AVI)
- [ ] Custom format penamaan folder
- [ ] Batch multi-kegiatan dalam satu sesi

---

<p align="center">Format berdasarkan <a href="https://keepachangelog.com/">Keep a Changelog</a> · <a href="../README.md">ArsipDok</a></p>
