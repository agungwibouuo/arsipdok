# 🔌 API Reference — ArsipDok IPC Protocol

> Spesifikasi lengkap command IPC (invoke) dan event real-time antara Frontend (Svelte 5) dan Backend (Rust/Tauri 2).

---

## Daftar Isi

1. [Konvensi Umum](#konvensi-umum)
2. [Command Reference](#command-reference)
   - [get_version](#get_version)
   - [load_settings](#load_settings)
   - [save_settings](#save_settings)
   - [scan_sources](#scan_sources)
   - [extract_metadata](#extract_metadata)
   - [build_plan](#build_plan)
   - [start_transfer](#start_transfer)
   - [cancel_transfer](#cancel_transfer)
   - [export_log](#export_log)
   - [check_disk_space](#check_disk_space)
   - [validate_destination](#validate_destination)
   - [open_path](#open_path)
3. [Events (Real-time Push)](#events-real-time-push)
   - [transfer://progress](#transferprogress)
   - [transfer://item](#transferitem)
4. [Tipe Data (Shared Schema)](#tipe-data-shared-schema)
5. [Penanganan Error](#penanganan-error)

---

## Konvensi Umum

### Naming Convention
- **Command names** ditulis dalam `snake_case` di Rust, dan di-invoke dengan nama sama di TypeScript.
- **Payload keys** dikirim sebagai plain JSON object dengan key `camelCase` atau `snake_case` sesuai parameter Rust.

### Serialisasi Proxy Svelte
> ⚠️ Svelte 5 `$state` proxy **tidak bisa** di-`structuredClone` oleh Tauri IPC (`DataCloneError`). Semua payload dari frontend **wajib** melewati helper `plain()`:

```typescript
// ui/src/lib/api.ts
function plain<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}
```

### Format Tanggal
Semua tanggal ditransmisikan sebagai ISO 8601 string (`YYYY-MM-DD`) dalam tipe `NaiveDate`, atau Unix timestamp (i64) untuk filesystem time.

---

## Command Reference

### `get_version`

Mendapatkan versi aplikasi dari backend.

```typescript
api.getVersion(): Promise<string>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn get_version() -> String
```

**Returns:**
```json
"0.1.0"
```

---

### `load_settings`

Memuat pengaturan aplikasi tersimpan dari `%APPDATA%\ArsipDok\settings.json`.

```typescript
api.loadSettings(): Promise<Settings>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn load_settings(state: State<'_, AppState>) -> Result<Settings>
```

**Returns:** Lihat tipe [Settings](#settings).

---

### `save_settings`

Menyimpan pengaturan aplikasi. Melakukan validasi sebelum persist ke disk.

```typescript
api.saveSettings(s: Settings): Promise<void>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, s: Settings) -> Result<()>
```

**Parameter:** `s` — Objek Settings yang akan disimpan (lihat [Settings](#settings)).

**Error:** Mengembalikan error jika validasi gagal (misalnya root folder tidak ada).

---

### `scan_sources`

Memindai satu atau lebih folder sumber secara rekursif untuk menemukan file foto/video.

```typescript
api.scanSources(sources: string[]): Promise<ScanReport>
```

**Rust Signature:**
```rust
#[tauri::command]
pub async fn scan_sources(sources: Vec<PathBuf>) -> Result<ScanReport>
```

**Parameter:**
| Key | Tipe | Deskripsi |
|---|---|---|
| `sources` | `string[]` | Array absolute path folder sumber |

**Returns:** [ScanReport](#scanreport)

**Contoh:**
```typescript
const report = await api.scanSources(['E:\\DCIM', 'F:\\Camera']);
console.log(report.total_files);  // 245
console.log(report.photos);       // 200
console.log(report.videos);       // 45
```

---

### `extract_metadata`

Mengekstrak tanggal dari EXIF/QuickTime/filesystem untuk setiap file.

```typescript
api.extractMetadata(files: ScannedFile[]): Promise<Record<string, DateMetadata>>
```

**Rust Signature:**
```rust
#[tauri::command]
pub async fn extract_metadata(
    files: Vec<ScannedFile>
) -> Result<BTreeMap<PathBuf, DateMetadata>>
```

**Parameter:**
| Key | Tipe | Deskripsi |
|---|---|---|
| `files` | `ScannedFile[]` | Array hasil scan |

**Returns:** Record yang map `path` → [DateMetadata](#datemetadata)

**Contoh:**
```typescript
const dates = await api.extractMetadata(report.files);
console.log(dates['E:\\DCIM\\IMG_0001.jpg']);
// { date: "2026-03-15", year: 2026, month: 3, day: 15,
//   source: "exif", needs_review: false, raw: "2026:03:15 10:30:00" }
```

---

### `build_plan`

Mengkalkulasi rencana transfer lengkap: mapping setiap file ke folder tujuan berdasarkan tanggal + nama kegiatan.

```typescript
api.buildPlan(
    files: ScannedFile[],
    dates: Record<string, DateMetadata>,
    destinations: Destination[]
): Promise<TransferPlan>
```

**Rust Signature:**
```rust
#[tauri::command]
pub async fn build_plan(
    files: Vec<ScannedFile>,
    dates: BTreeMap<PathBuf, DateMetadata>,
    destinations: Vec<Destination>
) -> Result<TransferPlan>
```

**Parameter:**
| Key | Tipe | Deskripsi |
|---|---|---|
| `files` | `ScannedFile[]` | File hasil scan (yang dicentang user) |
| `dates` | `Record<string, DateMetadata>` | Hasil extractMetadata (termasuk override manual) |
| `destinations` | `Destination[]` | Daftar tujuan yang enabled |

**Returns:** [TransferPlan](#transferplan)

---

### `start_transfer`

Memulai eksekusi transfer (copy/move). Ini adalah command async yang memancarkan event progres secara real-time selama eksekusi.

```typescript
api.startTransfer(
    plan: TransferPlan,
    mode: TransferMode,
    sourceRoots: string[]
): Promise<TransferReport>
```

**Rust Signature:**
```rust
#[tauri::command]
pub async fn start_transfer(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    plan: TransferPlan,
    mode: TransferMode,
    source_roots: Vec<PathBuf>
) -> Result<TransferReport>
```

**Parameter:**
| Key | Tipe | Deskripsi |
|---|---|---|
| `plan` | `TransferPlan` | Hasil buildPlan |
| `mode` | `TransferMode` | `"copy"` atau `"move"` |
| `sourceRoots` | `string[]` | Root folder sumber (untuk cleanup folder kosong) |

**Returns:** [TransferReport](#transferreport)

**Side Effects:**
- Memancarkan event `transfer://progress` dan `transfer://item` selama eksekusi
- Mode `move`: menghapus file sumber **hanya** setelah semua tujuan sukses terverifikasi
- Mode `move` + `remove_empty_source_dirs`: membersihkan folder sumber kosong
- Menulis log CSV ke `%APPDATA%\ArsipDok\logs\`

---

### `cancel_transfer`

Mengirim sinyal pembatalan transfer yang sedang berjalan.

```typescript
api.cancelTransfer(): Promise<void>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn cancel_transfer(state: State<'_, AppState>)
```

**Perilaku:**
- Set `cancel_flag` menjadi `true` secara atomik
- Transfer engine mengecek flag ini setelah setiap file selesai
- File yang sedang di-copy akan diselesaikan terlebih dahulu
- Hasil `TransferReport` akan memiliki `cancelled: true`

---

### `export_log`

Menyalin file log CSV ke lokasi ekspor yang dipilih user.

```typescript
api.exportLog(logPath: string, exportPath: string): Promise<void>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn export_log(log_path: PathBuf, export_path: PathBuf) -> Result<()>
```

---

### `check_disk_space`

Mengecek ruang disk tersedia pada path tujuan.

```typescript
api.checkDiskSpace(path: string): Promise<DiskSpaceInfo>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn check_disk_space(path: PathBuf) -> Result<DiskSpaceInfo>
```

**Returns:** [DiskSpaceInfo](#diskspaceinfo)

**Implementasi:** Menggunakan FFI `GetDiskFreeSpaceExW` (kernel32) di Windows tanpa dependensi eksternal.

---

### `validate_destination`

Validasi apakah path tujuan valid dan berupa direktori.

```typescript
api.validateDestination(path: string): Promise<boolean>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn validate_destination(path: PathBuf) -> Result<bool>
```

---

### `open_path`

Membuka path di Windows Explorer.

```typescript
api.openPath(path: string): Promise<void>
```

**Rust Signature:**
```rust
#[tauri::command]
pub fn open_path(app: tauri::AppHandle, path: PathBuf) -> Result<()>
```

**Implementasi:** Menggunakan `tauri_plugin_opener`.

---

## Events (Real-time Push)

### `transfer://progress`

Agregat progres keseluruhan transfer. Dipancarkan sebelum setiap file grup diproses, dan sekali lagi saat selesai.

**Payload:** [TransferProgress](#transferprogress)

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<TransferProgress>('transfer://progress', (e) => {
  console.log(`${e.payload.items_done}/${e.payload.items_total} selesai`);
});
```

### `transfer://item`

Update status per file. Dipancarkan setiap kali satu file selesai diproses.

**Payload:** [ItemProgress](#itemprogress)

```typescript
const unlisten = await listen<ItemProgress>('transfer://item', (e) => {
  if (e.payload.source_deleted) {
    console.log(`Sumber dihapus: ${e.payload.source}`);
  } else {
    console.log(`${e.payload.file_name} → ${e.payload.status}`);
  }
});
```

---

## Tipe Data (Shared Schema)

### `Settings`

```typescript
interface Settings {
  version: number;                    // Skema versi (current: 2)
  destinations: Destination[];
  root_history: string[];             // Riwayat folder tujuan
  activity_history: string[];         // Riwayat nama kegiatan
  suffix_history: string[];           // Riwayat suffix
  transfer_mode: TransferMode;        // "copy" | "move"
  verification: VerificationMode;     // "size_mtime" | "blake3"
  parallel_workers: number;           // Default: 4
  remove_empty_source_dirs: boolean;  // Default: true
  confirm_before_start: boolean;      // Default: true
  language: string;                   // Default: "id"
}
```

### `Destination`

```typescript
interface Destination {
  id: string;          // UUID unik
  root: string;        // Absolute path root tujuan
  activity: string;    // Nama kegiatan
  suffix?: string;     // Suffix opsional
  enabled: boolean;    // Default: true
}
```

### `TransferMode`

```typescript
type TransferMode = "copy" | "move";
```

### `VerificationMode`

```typescript
type VerificationMode = "size_mtime" | "blake3";
```

### `ScannedFile`

```typescript
interface ScannedFile {
  path: string;           // Absolute path
  name: string;           // File name dengan ekstensi
  size_bytes: number;
  extension: string;      // lowercase, tanpa dot
  kind: FileKind;         // "photo" | "video"
  modified?: number;      // Unix timestamp
  created?: number;       // Unix timestamp
}

type FileKind = "photo" | "video";
```

### `DateMetadata`

```typescript
interface DateMetadata {
  date: string;               // "YYYY-MM-DD"
  year: number;
  month: number;              // 1-12
  day: number;                // 1-31
  source: MetadataSource;
  needs_review: boolean;      // true jika pakai fallback filesystem
  raw?: string;               // Raw EXIF string, mis. "2026:03:15 10:30:00"
}

type MetadataSource =
  | "exif"            // EXIF DateTimeOriginal
  | "video"           // QuickTime/MP4 creation_time
  | "file_creation"   // FS creation time (fallback)
  | "file_modified"   // FS modified time (fallback)
  | "manual";         // Override manual oleh user
```

### `ScanReport`

```typescript
interface ScanReport {
  total_files: number;
  total_bytes: number;
  photos: number;
  videos: number;
  skipped: number;          // File non-foto/video yang dilewati
  duration_ms: number;
  files: ScannedFile[];
}
```

### `TransferPlan`

```typescript
interface TransferPlan {
  items: PlanItem[];
  total_files: number;
  total_bytes: number;
  folder_names: string[];   // Daftar unik folder yang akan dibuat
}

interface PlanItem {
  source: string;             // Path file sumber
  file_name: string;
  size_bytes: number;
  dest_root: string;          // Root tujuan
  dest_folder: string;        // "03-15_Rapat Koordinasi"
  dest_path: string;          // Full path tujuan
  date: string;               // "YYYY-MM-DD"
  month: number;
  day: number;
  metadata_source?: string;
}
```

### `TransferReport`

```typescript
interface TransferReport {
  items_done: number;
  items_error: number;
  items_skipped: number;
  total_bytes_transferred: number;
  log_path: string;           // Path ke CSV log
  log_entries: LogEntry[];
  cancelled: boolean;
  duration_ms: number;
}
```

### `LogEntry`

```typescript
interface LogEntry {
  timestamp: string;
  mode: string;               // "copy" | "move"
  source_path: string;
  dest_root: string;
  dest_path: string;
  size_bytes: number;
  status: string;             // "done" | "duplicate" | "skipped" | "error" | "partial"
  metadata_source: string;
  duration_ms: number;
  error?: string;
}
```

### `DiskSpaceInfo`

```typescript
interface DiskSpaceInfo {
  path: string;
  available_bytes: number;
  total_bytes: number;
}
```

### `TransferProgress` (Event)

```typescript
interface TransferProgress {
  items_done: number;
  items_error: number;
  items_skipped: number;
  items_total: number;
  bytes_transferred: number;
  per_dest: DestProgressEntry[];
  current_source: string;     // File yang sedang diproses
  cancelled: boolean;
}

interface DestProgressEntry {
  dest_root: string;
  done: number;
  total: number;
  has_error: boolean;
}
```

### `ItemProgress` (Event)

```typescript
interface ItemProgress {
  source: string;
  file_name: string;
  dest_root: string;
  dest_path: string;
  status: string;             // "done" | "duplicate" | "skipped" | "error" | "source_deleted" | ...
  error?: string;
  source_deleted: boolean;
}
```

---

## Penanganan Error

Semua command mengembalikan `Result<T>` di Rust. Bila error terjadi, Tauri melempar exception di sisi JS dengan struktur:

```typescript
try {
  await api.startTransfer(plan, 'move', roots);
} catch (err) {
  // err adalah string pesan error dari Rust
  console.error('Transfer gagal:', err);
}
```

### Jenis Error Backend

| Error | Penyebab | Solusi |
|---|---|---|
| `IoError` | File tidak ada, disk penuh, permission denied | Cek path & ruang disk |
| `VerificationFailed` | Ukuran/hash berbeda pasca-copy | Coba lagi atau cek kesehatan disk |
| `InvalidSettings` | Validasi settings gagal | Perbaiki field yang invalid |
| `Other` | Error umum lainnya | Baca pesan detail |

---

<p align="center">Dokumen ini bagian dari <a href="../README.md">ArsipDok</a> · v0.1.0</p>
