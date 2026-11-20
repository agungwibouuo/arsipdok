# 🏗️ Arsitektur Sistem ArsipDok

> Dokumen arsitektur teknis mendalam: lapisan sistem, alur data pipeline, dan keputusan desain.

---

## Daftar Isi

1. [Gambaran Umum](#1-gambaran-umum)
2. [Diagram Lapisan Sistem](#2-diagram-lapisan-sistem)
3. [Alur Pipeline Utama](#3-alur-pipeline-utama)
4. [Metadata Extraction Flow](#4-metadata-extraction-flow)
5. [Planner: Perencanaan Penempatan](#5-planner-perencanaan-penempatan)
6. [Transfer Pipeline](#6-transfer-pipeline)
7. [Verification Logic](#7-verification-logic)
8. [Concurrency & Cancellation Model](#8-concurrency--cancellation-model)
9. [State Management (Frontend)](#9-state-management-frontend)
10. [Keputusan Desain Kunci (ADR)](#10-keputusan-desain-kunci-adr)
11. [Batasan & Future Work](#11-batasan--future-work)

---

## 1. Gambaran Umum

ArsipDok menggunakan model aplikasi **desktop 3-tier**:

```text
┌─────────────────────────────────────────────┐
│  Tier 1: Presentasi (Svelte 5 UI)           │
│  - Render komponen, kelola input user       │
│  - State reaktif lokal (runes)              │
└────────────────────┬────────────────────────┘
                     │  WebView2 ↔ Tauri Core (IPC)
┌────────────────────▼────────────────────────┐
│  Tier 2: Aplikasi (Tauri 2 Core)            │
│  - Jembatan invoke/command & event emitter  │
│  - Manajemen window, permission, bundle     │
│  - Shared AppState (RwLock + AtomicBool)    │
└────────────────────┬────────────────────────┘
                     │  Rust internal calls
┌────────────────────▼────────────────────────┐
│  Tier 3: Domain Engine (Rust Modules)       │
│  - scanner, metadata, planner, transfer,    │
│    verifier, logger, paths, settings        │
└─────────────────────────────────────────────┘
```

**Kunci:** Logika file I/O, parsing metadata, dan verifikasi integritas **seluruhnya** berada di Rust. Frontend murni presentasi.

---

## 2. Diagram Lapisan Sistem

```text
┌───────────────────────────────────────────────────────────────┐
│                        FRONTEND (WebView2)                    │
│                                                               │
│   App.svelte (router state)                                   │
│      │                                                        │
│      ├─▶ WelcomeState.svelte     (state: welcome)             │
│      ├─▶ SourceQueue.svelte      (state: kosong/memindai)     │
│      ├─▶ DestPanel.svelte        (tujuan arsip)               │
│      ├─▶ PreviewTable.svelte     (state: siap)                │
│      ├─▶ TransferProgress.svelte (state: memindahkan)         │
│      └─▶ DoneState.svelte        (state: selesai/ada_masalah) │
│                                                               │
│   store.svelte.ts  ◀── $state / $derived (global state)       │
│   api.ts           ◀── invoke() & listen() wrappers           │
└───────────────────────────────┬───────────────────────────────┘
                                │ Tauri IPC Boundary
┌───────────────────────────────▼───────────────────────────────┐
│                      TAURI 2 CORE                             │
│   commands.rs ── 12 command handlers ──▶ Result<T>            │
│   AppState { config, settings: RwLock, cancel_flag: Arc }     │
└───────────────────────────────┬───────────────────────────────┘
                                │
┌───────────────────────────────▼───────────────────────────────┐
│                      RUST DOMAIN ENGINE                       │
│                                                               │
│  scanner.rs ──▶ walkdir → filter ext → ScannedFile[]          │
│       │                                                       │
│       ▼                                                       │
│  metadata.rs ──▶ kamadak-exif → mvhd → fs times → DateMeta    │
│       │                                                       │
│       ▼                                                       │
│  planner.rs ──▶ cross product (files × destinations)          │
│       │         + paths.build_folder_name()                   │
│       ▼                                                       │
│  transfer.rs ──▶ stream copy 4MB → verify → move/delete       │
│       │                                                       │
│       ├──▶ verifier.rs (size+mtime / blake3)                  │
│       └──▶ logger.rs   (CSV audit trail)                      │
└───────────────────────────────────────────────────────────────┘
```

---

## 3. Alur Pipeline Utama

Urutan lengkap dari input user hingga arsip terverifikasi:

```text
[USER] Pilih folder sumber
   │
   ▼
① SCAN ─────────────────────────────────────────────
   scan_sources(sources)
   └─▶ untuk setiap folder: walkdir rekursif
       ├─ filter ekstensi (PHOTO_EXTS / VIDEO_EXTS)
       ├─ baca size, created, modified
       └─ klasifikasi kind (Photo/Video)
   ◀─ ScanReport { files, total_bytes, photos, videos }
   │
   ▼
② EXTRACT METADATA ─────────────────────────────────
   extract_metadata(files)
   └─▶ per file: extract_date(path, created, modified)
       (lihat §4 untuk hierarki)
   ◀─ Record<path, DateMetadata>
   │
   ▼
[USER] Atur tujuan + nama kegiatan + (opsional) override tanggal
   │
   ▼
③ BUILD PLAN ───────────────────────────────────────
   build_plan(files, dates, destinations)
   └─▶ cross product: setiap file × setiap destination enabled
       ├─ paths.build_folder_name(date, activity, suffix)
       └─ PlanItem { source, dest_path, date, ... }
   ◀─ TransferPlan { items, total_files, folder_names }
   │
   ▼
④ TRANSFER ─────────────────────────────────────────
   start_transfer(plan, mode, source_roots)
   └─▶ group items by source file
       └─▶ per group (per file sumber):
           ├─ per destination: execute_item()
           │   ├─ stream copy 4MB buffer
           │   ├─ handle collision (rename / skip)
           │   └─ verify(dest)  ← size+mtime atau BLAKE3
           ├─ jika mode=move DAN semua dest sukses: delete_source()
           └─ emit transfer://item + transfer://progress
       ├─ mode=move: remove_empty_dirs(source_roots)
       └─ log.save() → CSV
   ◀─ TransferReport { items_done, log_path, ... }
   │
   ▼
[USER] Lihat ringkasan + akses log audit
```

---

## 4. Metadata Extraction Flow

Hierarki prioritas penentuan tanggal (fungsi `metadata::extract_date`):

```text
                    ┌─────────────────────────┐
                    │     extract_date()      │
                    └───────────┬─────────────┘
                                │
              ┌─────────────────▼──────────────────┐
              │ 1. EXIF DateTimeOriginal/CreateDate │
              │    (foto: JPG/TIFF/HEIC/RAW)        │
              └──────────┬─────────────────────────┘
                         │ found?
              ┌──────────┴──────────┐
           YES│                     │NO
              ▼                     ▼
     ┌──────────────┐    ┌─────────────────────────┐
     │ source=Exif  │    │ 2. QuickTime/MP4 mvhd   │
     │ needs_review=│    │    creation_time (video)│
     │   false      │    └──────────┬──────────────┘
     └──────────────┘               │ found?
                          ┌─────────┴─────────┐
                       YES│                   │NO
                          ▼                   ▼
                 ┌──────────────┐  ┌──────────────────────┐
                 │ source=Video │  │ 3. FS creation time  │
                 │ needs_review=│  │    source=FileCreation│
                 │   false      │  │    needs_review=true │
                 └──────────────┘  └──────────┬───────────┘
                                               │ available?
                                    ┌──────────┴──────────┐
                                 YES│                     │NO
                                    ▼                     ▼
                          ┌──────────────┐   ┌─────────────────────┐
                          │FileCreation  │   │ 4. FS modified time │
                          └──────────────┘   │ source=FileModified │
                                             │ needs_review=true   │
                                             └─────────────────────┘
```

**PoV:**
- **`needs_review = true`** memicu indikator kuning di UI agar user memeriksa/meng-override.
- **Override manual** di UI mengubah `source = "manual"` dan `needs_review = false`.
- Semua sumber tanggal **dilaporkan** ke log audit (kolom `metadata_source`) untuk traceability.

**Detail parsing video:** Membaca MP4 `mvhd` atom (ISO base media format). Waktu epoch Apple 1904 dikonversi ke Unix epoch, lalu UTC → local timezone.

---

## 5. Planner: Perencanaan Penempatan

`planner::build_plan` melakukan **cross product** antara file dan destinasi:

```text
files = [A.jpg, B.mp4]
destinations = [D1(enabled), D2(enabled), D3(disabled)]

hasil items:
  A.jpg × D1 → D:\Arsip\03-15_Kegiatan\A.jpg
  A.jpg × D2 → E:\Backup\03-15_Kegiatan_Backup\A.jpg
  B.mp4 × D1 → D:\Arsip\03-15_Kegiatan\B.mp4
  B.mp4 × D2 → E:\Backup\03-15_Kegiatan_Backup\B.mp4
  (D3 disabled → dilewati)
```

### Penamaan Folder

Fungsi `paths::build_folder_name`:

```text
"{MM}-{DD}_{sanitize(activity)}[_ {sanitize(suffix)}]"

Contoh:
  date=2026-03-15, activity="Rapat Koordinasi", suffix=None
  → "03-15_Rapat Koordinasi"

  date=2026-03-15, activity="Rapat Koordinasi", suffix="Backup"
  → "03-15_Rapat Koordinasi_Backup"
```

**Sanitasi Windows:**
1. Ganti karakter `<>:"/\|?*` dan kontrol `\x00-\x1F` dengan spasi
2. Rapikan spasi berlebih (regex `\s+` → single space)
3. Hindari reserved names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`) → tambah prefix `_`
4. Batasi panjang segmen maksimum **150 karakter**
5. Trim trailing dot/space (invalid di Windows)

### Path Panjang (>260 char)

`paths` module menambahkan prefix `\\?\` pada path absolut di Windows agar `MAX_PATH` limit tidak dilanggar untuk struktur folder arsip yang dalam.

---

## 6. Transfer Pipeline

Fungsi utama: `transfer::execute_item(item, config, cancel_flag)`.

```text
execute_item(item) ─────────────────────────────────────
   │
   ├─ 1. Cek cancel_flag → jika true: return Skipped
   │
   ├─ 2. Rewrite dest_path:
   │      if exists(dest_path):
   │          a) bandingkan size
   │          b) jika identik → status = Duplicate, return
   │          c) jika beda → tambah suffix _1, _2, ... (collision handling)
   │
   ├─ 3. Buat folder tujuan (create_dir_all) bila belum ada
   │
   ├─ 4. Stream copy:
   │      BufReader(src).read(&mut 4MB_buffer) → BufWriter(dst).write()
   │      loop hingga EOF; cek cancel_flag setiap iterasi
   │
   ├─ 5. Flush + sync (opsional) ke disk
   │
   ├─ 6. Verifikasi via verifier::verify(src, dst, mode):
   │      - size mismatch → Err(VerificationFailed) → status=Error
   │      - mode=Blake3 → hash compare → status=Error bila beda
   │
   └─ 7. Setelah semua dest sukses (di orchestrator commands.rs):
          - mode=move → delete_source() → emit source_deleted event
```

### Buffer & Throughput

- **Buffer size:** 4 MB (`4 * 1024 * 1024`) — kompromi optimal antara syscall overhead dan memori.
- **Paralelisme:** Item diproses **sequential per file** agar mode Move aman (sumber dihapus hanya setelah semua tujuan sukses). Paralelisme terjadi di level **per-destination** dalam satu file.

### Collision Handling

```text
File sumber:  IMG_0001.jpg
File tujuan:  IMG_0001.jpg (sudah ada)

jika size sama   → Duplicate (skip, sukses)
jika size beda   → IMG_0001_1.jpg (copy file baru)
jika _1 juga ada → IMG_0001_2.jpg
dst.
```

---

## 7. Verification Logic

`verifier::verify(source, dest, mode)`:

| Mode | Algoritma | Kompleksitas | Deteksi |
|---|---|---|---|
| **SizeMtime** | `metadata.len()` + `modified()` compare | O(1) | Copy terpotong, file 0-byte |
| **BLAKE3** | Full-file hash 32-byte compare | O(n) I/O | Semua korupsi bit-level |

```text
verify():
  src_meta = metadata(source)
  dst_meta = metadata(dest)

  if src_meta.len() != dst_meta.len()  → Err("ukuran beda")
  if mode == Blake3:
      if hash(src) != hash(dst)        → Err("hash BLAKE3 berbeda")
  return Ok
```

**Trade-off:** SizeMtime hampir gratis tapi tidak mendeteksi korupsi bit-level. BLAKE3 100% akurat tapi ~2x lebih lambat dari copy. Rekomendasi: SizeMtime untuk lokal SSD, BLAKE3 untuk NAS/jaringan/arsip jangka panjang.

---

## 8. Concurrency & Cancellation Model

### Shared State (Tauri `AppState`)

```rust
pub struct AppState {
    pub config: AppConfig,                       // Immutable paths
    pub settings: parking_lot::RwLock<Settings>, // Thread-safe config
    pub cancel_flag: Arc<AtomicBool>,            // Cancellation signal
}
```

### Cancellation Flow

```text
[UI Thread]           [Command Thread]         [Transfer Worker]
    │                       │                        │
    │ cancel_transfer()─────▶│                        │
    │                       │ cancel_flag.store(true)│
    │                       │                        │
    │                       │                        ├─ akses cancel_flag
    │                       │                        │  di setiap iterasi
    │                       │                        │  copy buffer
    │                       │                        ├─ jika true:
    │                       │                        │   break loop
    │                       │◀─── return Skipped ────┤
    │                       │                        │
    │◀── TransferReport ────┤                        │
    │    (cancelled: true)  │                        │
```

**Kunci:** `AtomicBool` memberi **latensi pembatalan minimal** (~4 MB read/write), jauh lebih responsif daripada channel-based cancellation.

### RwLock untuk Settings

`save_settings` mengakuisisi `write()` lock (exclusive), sementara `load_settings` dan pembacaan config transfer mengakuisisi `read()` lock (shared). Ini memastikan settings konsisten selama transfer tanpa memblokir pembacaan paralel.

---

## 9. State Management (Frontend)

### Svelte 5 Runes — Reactive Store

`store.svelte.ts` mengekspor object reaktif global:

```typescript
export const app = $state({
  state: 'welcome' as AppState,  // State machine
  sourcePaths: [] as string[],
  scannedFiles: [] as ScannedFile[],
  dates: {} as Record<string, DateMetadata>,
  destinations: [] as Destination[],
  plan: null as TransferPlan | null,
  progress: 0,
  settings: null as Settings | null,
  appVersion: '' as string,
});
```

### Derived State

```typescript
const totalBytes = $derived(
  app.scannedFiles.reduce((sum, f) => sum + f.size_bytes, 0)
);
const canTransfer = $derived(
  app.state === 'siap' && app.destinations.some(d => d.enabled)
);
```

### State Machine Transitions

| State | Aksi | Transisi ke |
|---|---|---|
| `welcome` | Pilih sumber | `kosong` |
| `kosong` | Scan selesai | `memindai` → `siap` |
| `siap` | Override tanggal | `siap` (in-place) |
| `siap` | Mulai transfer | `memindahkan` |
| `memindahkan` | Selesai semua | `selesai` |
| `memindahkan` | Ada error | `ada_masalah` |
| `memindahkan` | Dibatalkan | `siap` / `ada_masalah` |

---

## 10. Keputusan Desain Kunci (ADR)

### ADR-1: Rust untuk Engine, Bukan JavaScript

**Keputusan:** Seluruh file I/O & paring metadata di Rust.
**Alasan:** Throughput jauh lebih tinggi, memori rendah, verifikasi hash native, integritas threading.
**Konsekuensi:** Frontend tidak bisa melakukan operasi file langsung — semua via IPC.

### ADR-2: Move Menunggu Verifikasi Multi-Tujuan

**Keputusan:** File sumber hanya dihapus setelah **semua** tujuan verifikasi sukses.
**Alasan:** Mencegah **data loss** permanen bila salah satu tujuan gagal.
**Konsekuensi:** Mode Move sedikit lebih lambat (menunggu verifikasi tujuan terakhir).

### ADR-3: `AtomicBool` untuk Cancellation

**Keputusan:** Gunakan `Arc<AtomicBool>` bukan `mpsc` channel.
**Alasan:** Pola check-flag sederhana, latensi minimal, tidak perlu thread tambahan.
**Konsekuensi:** Pembatalan granular per-buffer, bukan per-byte (cukup untuk UX).

### ADR-4: Serialisasi Proxy Svelte via JSON

**Keputusan:** Helper `plain()` (`JSON.parse(JSON.stringify())`) sebelum IPC.
**Alasan:** Proxies Svelte 5 tidak bisa `structuredClone` (DataCloneError Tauri).
**Konsekuensi:** Overhead serialisasi kecil, tapi menjamin kompatibilitas penuh.

### ADR-5: Penamaan Folder Berbasis Metadata, Bukan Tanggal Transfer

**Keputusan:** Folder dinamai dari tanggal foto/video diambil (EXIF/video), bukan tanggal transfer.
**Alasan:** Arsip harus mencerminkan kronologi kejadian, bukan kapan diproses.
**Konsekuensi:** Butuh parsing metadata kompleks + override manual untuk kasus fallback.

---

## 11. Batasan & Future Work

### Batasan Saat Ini (v0.1.0)

- **Windows only** — FFI `GetDiskFreeSpaceExW` dan path handling spesifik Windows.
- **Belum ada resume** — transfer terputus harus diulang dari awal (tapi file yang sudah ada di-skip sebagai duplicate).
- **Belum ada auto-detect SD card** — sumber harus ditambah manual.
- **Video metadata** terbatas pada MP4/MOV `mvhd` atom; format lain (AVI/MKV) fallback ke filesystem time.
- **Tidak ada preview thumbnail** — tabel hanya menampilkan metadata tekstual.

### Roadmap

- [ ] Auto-import saat SD card terdeteksi (Windows device watch)
- [ ] Resume transfer setelah crash/interrupt
- [ ] Ekspor log ke PDF/Excel
- [ ] Thumbnail preview di tabel
- [ ] Dukungan macOS/Linux (cross-platform disk space API)
- [ ] Parsing metadata format video tambahan (MKV, AVI)
- [ ] Batch multi-kegiatan dalam satu sesi

---

<p align="center">Dokumen ini bagian dari <a href="../README.md">ArsipDok</a> · v0.1.0</p>
