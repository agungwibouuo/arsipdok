# 🛠️ Panduan Developer ArsipDok

> Panduan pengembangan, arsitektur, modul kode, pengujian, dan pembuatan rilis aplikasi ArsipDok.

---

## Daftar Isi

1. [Prasyarat Pengembangan](#1-prasyarat-pengembangan)
2. [Menyiapkan Lingkungan Kerja](#2-menyiapkan-lingkungan-kerja)
3. [Arsitektur Proyek](#3-arsitektur-proyek)
4. [Struktur Kode & Direktori](#4-struktur-kode--direktori)
5. [Modul Backend (Rust)](#5-modul-backend-rust)
6. [Frontend (Svelte 5 + TypeScript)](#6-frontend-svelte-5--typescript)
7. [Protokol Komunikasi (IPC)](#7-protokol-komunikasi-ipc)
8. [Perintah Pengembangan](#8-perintah-pengembangan)
9. [Strategi Pengujian](#9-strategi-pengujian)
10. [Build & Packaging (NSIS Installer)](#10-build--packaging-nsis-installer)
11. [Konvensi Kode & Best Practices](#11-konvensi-kode--best-practices)

---

## 1. Prasyarat Pengembangan

Sebelum mulai, pastikan alat berikut terpasang di sistem Windows Anda:

| Alat | Versi Minimum | Catatan |
|---|---|---|
| **Rust Toolchain** | 1.77+ (edition 2021) | Target `x86_64-pc-windows-msvc` |
| **Node.js** | 20.x LTS atau lebih baru | Runtime frontend |
| **pnpm** | 9.x+ | Package manager utama |
| **Visual Studio Build Tools** | 2022 (v143) | Dibutuhkan untuk kompilasi Rust MSVC & C++ |
| **WebView2 Runtime** | Evergreen | Terpasang default pada Windows 10/11 |
| **Git** | 2.40+ | Version control |

### Verifikasi Instalasi

Jalankan perintah ini di PowerShell untuk memeriksa status:

```powershell
rustc --version
cargo --version
node --version
pnpm --version
```

---

## 2. Menyiapkan Lingkungan Kerja

### 1. Clone Repositori

```bash
git clone https://github.com/agungwibouuo/arsipdok.git
cd arsipdok
```

### 2. Pasang Dependensi Node

```bash
# Pasang dependensi root dan ui
pnpm install
pnpm --dir ui install
```

### 3. Jalankan Mode Pengembangan (Hot Reload)

```bash
pnpm tauri:dev
```

Perintah ini akan:
- Menjalankan Vite dev server untuk UI Svelte di `http://localhost:1420`
- Mengompilasi backend Rust dalam mode `debug`
- Menampilkan jendela aplikasi Tauri dengan hot-module replacement (HMR) aktif

---

## 3. Arsitektur Proyek

ArsipDok dirancang dengan pola **Separation of Concerns**:

```text
┌────────────────────────────────────────────────────────┐
│                   FRONTEND (UI)                        │
│   Svelte 5 + Runes ($state, $derived) + TypeScript     │
│   TailwindCSS-like styling / Vanilla scoped CSS       │
│   Responsif (Snap 2-side: min 620px, stack < 860px)    │
└─────────────────────────▲──────────────────────────────┘
                          │  IPC (Invoke / Events)
┌─────────────────────────▼──────────────────────────────┐
│                   TAURI 2 CORE                         │
│   Commands, AppState, Window management, Permissions   │
└─────────────────────────▲──────────────────────────────┘
                          │  Internal Rust API
┌─────────────────────────▼──────────────────────────────┐
│                 BACKEND ENGINE (RUST)                  │
│                                                        │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────┐  │
│  │   Scanner    │──▶│   Metadata   │──▶│  Planner   │  │
│  └──────────────┘   └──────────────┘   └─────┬──────┘  │
│                                              │         │
│  ┌──────────────┐   ┌──────────────┐         │         │
│  │    Logger    │◀──│   Verifier   │◀────────┘         │
│  │  (CSV Audit) │   │ (Size/BLAKE3)│   Transfer Engine │
│  └──────────────┘   └──────────────┘  (Copy/Move/Skip) │
└────────────────────────────────────────────────────────┘
```

### Prinsip Utama
1. **Frontend hanya menampilkan dan mengumpulkan input**: Tidak ada logika manipulasi file atau verifikasi hash di frontend.
2. **Backend 100% thread-safe**: State dikelola melalui `AppState` dengan `parking_lot::RwLock` dan `Arc<AtomicBool>` untuk pembatalan transfer yang instan.
3. **Fail-Safe Transfer**: Mode `Move` tidak pernah menghapus file sumber sebelum verifikasi integritas di tujuan berhasil.

---

## 4. Struktur Kode & Direktori

```text
ArsipDok/
├── .github/                # GitHub Actions CI/CD workflows
├── design/                 # Design assets
│   ├── ArsipDok.figcraft   # Master file Figma/FigCraft
│   └── preview-welcome.png # Asset screenshot
├── docs/                   # Dokumentasi lengkap proyek
│   ├── USER_GUIDE.md       # Panduan pengguna akhir
│   ├── DEVELOPER.md        # Panduan teknis & pengembangan (file ini)
│   ├── ARCHITECTURE.md     # Arsitektur sistem & alur kerja
│   ├── API.md              # Spesifikasi protokol IPC
│   └── TROUBLESHOOTING.md  # Masalah umum & solusi
├── src-tauri/              # Backend Rust (Tauri 2)
│   ├── Cargo.toml          # Dependensi & metadata Rust
│   ├── tauri.conf.json     # Konfigurasi Tauri 2 (window, security, bundle)
│   ├── icons/              # Ikon aplikasi multi-resolusi
│   ├── src/
│   │   ├── main.rs         # Entrypoint eksekusi binary
│   │   ├── lib.rs          # State aplikasi & bootstrap Tauri
│   │   ├── commands.rs     # IPC Command Handlers
│   │   ├── scanner.rs      # Recursive file scanning & filtering
│   │   ├── metadata.rs     # Parser EXIF & MP4 creation time
│   │   ├── paths.rs        # Sanitasi path Windows & penamaan
│   │   ├── planner.rs      # Perhitungan mapping file -> tujuan
│   │   ├── transfer.rs     # Engine transfer file & penanganan error
│   │   ├── verifier.rs     # Verifikasi integritas (Size/MTime & BLAKE3)
│   │   ├── settings.rs     # Persistence config (%APPDATA%\ArsipDok)
│   │   ├── logger.rs       # CSV session auditing
│   │   └── error.rs        # Definisi enum Error & Result
│   └── tests/
│       └── pipeline_test.rs # E2E Integration tests
├── ui/                     # Frontend Svelte 5
│   ├── package.json        # Dependensi frontend
│   ├── vite.config.ts      # Konfigurasi Vite + Svelte
│   ├── svelte.config.js    # Konfigurasi Svelte preprocessor
│   └── src/
│       ├── App.svelte      # Root component & State router
│       ├── main.ts         # Frontend entrypoint
│       └── lib/
│           ├── types.ts    # TypeScript definitions sinkron dengan Rust
│           ├── api.ts      # Wrapper invokasi Tauri
│           ├── store.svelte.ts # Global reactive state (Svelte 5 runes)
│           └── components/ # Komponen UI modular
│               ├── TitleBar.svelte
│               ├── WelcomeState.svelte
│               ├── SourceQueue.svelte
│               ├── DestPanel.svelte
│               ├── PreviewTable.svelte
│               ├── TransferProgress.svelte
│               ├── DoneState.svelte
│               └── SettingsModal.svelte
├── CHANGELOG.md            # Riwayat perubahan versi
├── PRD.md                  # Product Requirements Document
├── README.md               # Quickstart & dokumentasi publik
└── package.json            # Scripts runner root
```

---

## 5. Modul Backend (Rust)

Backend terbagi ke dalam modul modular yang berdiri sendiri:

### `scanner.rs`
- Bertanggung jawab memindai folder secara rekursif menggunakan crate `walkdir`.
- Melakukan filter berdasarkan whitelist ekstensi foto (`PHOTO_EXTS`) dan video (`VIDEO_EXTS`).
- Mengumpulkan metadata dasar filesystem: ukuran, waktu modifikasi, waktu pembuatan.
- Melewati file tersembunyi atau file sistem yang tidak relevan.

### `metadata.rs`
- Mengekstrak tanggal asli kejadian foto/video menggunakan hierarki prioritas:
  1. **EXIF** via crate `kamadak-exif` (tag `DateTimeOriginal`, `CreateDate`, atau `ModifyDate`).
  2. **Video MP4/MOV** parsing `mvhd` atom creation timestamp.
  3. **Filesystem Creation Time**.
  4. **Filesystem Modified Time** (fallback terakhir).
- Jika menggunakan fallback non-EXIF, `needs_review` diset `true` untuk memperingatkan pengguna di UI.

### `paths.rs`
- Mengonstruksi nama folder sesuai pola: `MM-DD_Nama Kegiatan[_Suffix]`.
- Melakukan sanitasi karakter terlarang Windows: `< > : " / \ | ? *` dan reserved names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`).
- Menangani prefix path panjang Windows `\\?\` untuk mencegah batasan `MAX_PATH` (260 karakter).

### `planner.rs`
- Mengkombinasikan daftar file hasil scan, tanggal metadata, dan daftar destinasi aktif.
- Menghasilkan `TransferPlan` yang berisi array `PlanItem`.
- Menghitung total ukuran transfer dan agregasi statistik.

### `transfer.rs`
- Jantung eksekusi file I/O:
  - Buffer streaming 4 MB untuk efisiensi throughput disk.
  - Memeriksa duplikasi nama: jika file sudah ada dengan konten berbeda, beri suffix increment `_1`, `_2`.
  - Jika file sudah ada dengan konten identik, status diset `Duplicate` atau `Skipped`.
- Mendukung pemutusan instan via `cancel_flag` (`Arc<AtomicBool>`).
- Mode `Move` mengeksekusi penghapusan file sumber **hanya** jika semua tujuan telah tersalin dan diverifikasi sukses.

### `verifier.rs`
- Menyediakan dua mode verifikasi:
  1. `SizeMtime`: Cepat, memeriksa kesamaan ukuran byte dan mtime.
  2. `Blake3`: Hashing kriptografis menggunakan crate `blake3` untuk memastikan bit-level accuracy.

### `logger.rs`
- Mencatat setiap aksi per file ke file CSV di direktori log aplikasi:
  `%APPDATA%\ArsipDok\logs\YYYY-MM-DD_HHMMSS.csv`.
- Format log mencakup: `timestamp`, `mode`, `source_path`, `dest_root`, `dest_path`, `size_bytes`, `status`, `metadata_source`, `duration_ms`, `error`.

### `settings.rs`
- Serialisasi dan deserialisasi konfigurasi aplikasi dari dan ke `%APPDATA%\ArsipDok\settings.json`.
- Menyimpan history input (nama kegiatan, suffix, direktori tujuan) untuk autocomplete.

---

## 6. Frontend (Svelte 5 + TypeScript)

Frontend dibangun menggunakan fitur terbaru **Svelte 5 (Runes)** untuk reaktivitas granular:

### Reaktivitas Menggunakan Runes
State global di `ui/src/lib/store.svelte.ts` tidak lagi menggunakan Svelte 4 writable stores, melainkan runes:
- `$state()`: Mendefinisikan reactive state (`appState`, `sourcePaths`, `destinations`, `previewRows`).
- `$derived()`: Menghitung nilai turunan (`totalBytes`, `photoCount`, `videoCount`, `canTransfer`).
- `$effect()`: Menjalankan side effect seperti auto-save pengaturan atau sinkronisasi tema.

### Alur State Aplikasi (State Machine)
```text
  ┌─────────┐
  │ welcome │ ──(buka sumber)──┐
  └────┬────┘                  │
       │ (tambah file)         │
       ▼                       ▼
  ┌─────────┐             ┌─────────┐
  │ kosong  │ ──────────▶ │ memindai│
  └─────────┘             └────┬────┘
                               │ (scan selesai)
                               ▼
                          ┌─────────┐
                          │  siap   │ ◀──(edit/override tanggal)
                          └────┬────┘
                               │ (klik transfer)
                               ▼
                          ┌─────────────┐
                          │ memindahkan │ ──(batal)──┐
                          └────┬────────┘            │
                               │ (selesai)           │
                               ▼                     ▼
                          ┌─────────┐          ┌─────────────┐
                          │ selesai │          │ ada_masalah │
                          └─────────┘          └─────────────┘
```

### Penanganan Proxy Svelte vs Tauri IPC
Tauri IPC menggunakan `structuredClone` di background webview. Svelte 5 `$state` proxy tidak dapat di-clone secara langsung (`DataCloneError`). Di `ui/src/lib/api.ts`, semua payload disanitasi menggunakan helper `plain(data)` (`JSON.parse(JSON.stringify(data))`) sebelum dikirim ke `invoke`.

---

## 7. Protokol Komunikasi (IPC)

Komunikasi antara UI dan Rust menggunakan dua jalur:

1. **Commands (Request/Response via `invoke`)**:
   - `get_version()`
   - `scan_sources(sources)`
   - `extract_metadata(files)`
   - `build_plan(files, dates, destinations)`
   - `start_transfer(plan, mode, source_roots)`
   - `cancel_transfer()`
   - `load_settings()` / `save_settings(settings)`
   - `check_disk_space(path)`
   - `export_log(log_path, export_path)`
   - `open_path(path)`

2. **Events (Real-time Push dari Backend ke UI)**:
   - `transfer://progress`: Agregasi progres umum (bytes, total file, item selesai, progres per destinasi).
   - `transfer://item`: Status update spesifik per file saat mulai dicopy, diverifikasi, atau selesai.

*(Lihat dokumentasi lengkap di [API.md](API.md))*

---

## 8. Perintah Pengembangan

Berikut script npm/pnpm yang tersedia di root `package.json`:

```bash
# Menjalankan aplikasi dalam mode dev (Vite + Tauri)
pnpm tauri:dev

# Menjalankan build frontend saja (Vite)
pnpm ui:build

# Menjalankan svelte-check untuk validasi tipe TypeScript di UI
pnpm check

# Menjalankan pengujian backend Rust
pnpm test:rust

# Mengompilasi aplikasi release & membuat installer NSIS
pnpm tauri:build
```

---

## 9. Strategi Pengujian

Pengujian mencakup unit test per modul dan end-to-end integration test:

### Menjalankan Test Rust
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Cakupan Test (`src-tauri/tests/pipeline_test.rs`)
1. **Pipeline Copy**: Meniru proses dari scan, ekstraksi EXIF mock, pembuatan plan, eksekusi copy, verifikasi struktur folder, dan audit log.
2. **Pipeline Move**: Memastikan file sumber terhapus hanya setelah berhasil diverifikasi di tujuan.
3. **BLAKE3 Integrity**: Memverifikasi bahwa corrupt bit pada file tujuan terdeteksi gagal saat hash checking.
4. **Duplicate Collision**: Mengetes skenario bentrok nama file agar tidak terjadi overwrite data.

### Pemeriksaan Tipe & Linting UI
```bash
cd ui
pnpm check
```
Memastikan 0 error pada compiler Svelte dan TypeScript.

---

## 10. Build & Packaging (NSIS Installer)

ArsipDok menggunakan NSIS bundler dari Tauri untuk menghasilkan installer Windows standalone.

### Konfigurasi di `src-tauri/tauri.conf.json`
- Target: `x86_64-pc-windows-msvc`
- Output installer: `src-tauri/target/release/bundle/nsis/ArsipDok_<version>_x64-setup.exe`
- Mode: Per-user install (tidak mewajibkan privilege Administrator).

### Menjalankan Build Release
```bash
pnpm tauri:build
```

### Prosedur Release Versi Baru
1. Pastikan working tree bersih dan semua pengujian lolos:
   ```bash
   pnpm check
   pnpm test:rust
   ```
2. Naikkan versi di `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, dan `src-tauri/src/lib.rs` (`VERSION`).
3. Perbarui `CHANGELOG.md`.
4. Commit dan tag versi git:
   ```bash
   git commit -am "chore: release vX.Y.Z"
   git tag vX.Y.Z
   git push origin main --tags
   ```
5. Buat installer release dan upload binary ke GitHub Releases.

---

## 11. Konvensi Kode & Best Practices

1. **Rust**:
   - Selalu gunakan `Result<T, crate::Error>` untuk error handling yang transparan. Hindari `.unwrap()` atau `.expect()` pada alur runtime produksi.
   - I/O berukuran besar wajib menggunakan streaming buffer (default: 4 MB) untuk menjaga penggunaan RAM tetap rendah.
   - Path Windows harus selalu disanitasi terhadap reserved characters dan batas panjang string.
2. **Svelte / TypeScript**:
   - Manfaatkan Svelte 5 Runes secara konsisten (`$state`, `$derived`).
   - Semua interaksi backend wajib melalui modul `ui/src/lib/api.ts`.
   - Gaya tampilan diatur secara modular pada komponen dengan CSS variabel untuk mendukung mode Dark/Light/System tanpa flicker.

---

<p align="center">Dokumen ini bagian dari <a href="../README.md">ArsipDok</a> · v0.1.0</p>
