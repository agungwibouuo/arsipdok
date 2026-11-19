// Tipe-tipe yang mirror struct Rust di src-tauri/src/

export type TransferMode = 'copy' | 'move';
export type VerificationMode = 'size_mtime' | 'blake3';
export type MetadataSource = 'exif' | 'video' | 'file_creation' | 'file_modified' | 'manual';
export type FileKind = 'photo' | 'video';
export type ItemStatus =
  | 'pending'
  | 'copying'
  | 'verifying'
  | 'done'
  | 'skipped'
  | 'duplicate'
  | 'error'
  | 'partial';

export interface Destination {
  id: string;
  root: string;
  activity: string;
  suffix?: string;
  enabled: boolean;
}

export interface Settings {
  version: number;
  destinations: Destination[];
  root_history: string[];
  activity_history: string[];
  suffix_history: string[];
  transfer_mode: TransferMode;
  verification: VerificationMode;
  parallel_workers: number;
  remove_empty_source_dirs: boolean;
  confirm_before_start: boolean;
  language: string;
}

export interface ScannedFile {
  path: string;
  name: string;
  size_bytes: number;
  extension: string;
  kind: FileKind;
  modified?: number | null;
  created?: number | null;
}

export interface DateMetadata {
  date: string; // "YYYY-MM-DD"
  year: number;
  month: number;
  day: number;
  source: MetadataSource;
  needs_review: boolean;
  raw?: string | null;
}

export interface PlanItem {
  source: string;
  file_name: string;
  size_bytes: number;
  dest_root: string;
  dest_folder: string;
  dest_path: string;
  date: string;
  month: number;
  day: number;
  metadata_source?: string | null;
}

export interface TransferPlan {
  items: PlanItem[];
  total_files: number;
  total_bytes: number;
  folder_names: string[];
}

export interface ScanReport {
  total_files: number;
  total_bytes: number;
  photos: number;
  videos: number;
  skipped: number;
  duration_ms: number;
  files: ScannedFile[];
}

export interface TransferReport {
  items_done: number;
  items_error: number;
  items_skipped: number;
  total_bytes_transferred: number;
  log_path?: string;
  log_entries?: LogEntry[];
  cancelled?: boolean;
  duration_ms?: number;
}

export interface DiskSpaceInfo {
  path: string;
  available_bytes: number;
  total_bytes: number;
}

// Row yang ditampilkan di tabel preview (file + metadata gabungan)
export interface PreviewRow {
  source: string;
  name: string;
  size_bytes: number;
  kind: FileKind;
  extension: string;
  date: string; // "YYYY-MM-DD"
  month: number;
  day: number;
  metadata_source: MetadataSource;
  needs_review: boolean;
  // date yang bisa dioverride manual oleh user
  date_override?: string;
  // status setelah transfer
  status?: ItemStatus;
  error?: string;
}

// Status per tujuan saat transfer berlangsung
export interface DestProgress {
  dest_id: string;
  dest_root: string;
  dest_folder: string;
  done: number;
  total: number;
  bytes_done: number;
  bytes_total: number;
  status: 'waiting' | 'running' | 'done' | 'error';
}

// Log entry untuk tabel log session
export interface LogEntry {
  timestamp: string;
  mode: string;
  source_path: string;
  dest_root: string;
  dest_path: string;
  size_bytes: number;
  status: string;
  metadata_source: string;
  duration_ms: number;
  error?: string;
}

// Payload event `transfer://progress` dari backend
export interface BackendProgress {
  items_done: number;
  items_error: number;
  items_skipped: number;
  items_total: number;
  bytes_transferred: number;
  per_dest: Array<{
    dest_root: string;
    done: number;
    total: number;
    has_error: boolean;
  }>;
  current_source: string;
  cancelled: boolean;
}

// Payload event `transfer://item` dari backend
export interface BackendItemEvent {
  source: string;
  file_name: string;
  dest_root: string;
  dest_path: string;
  status: string;
  error?: string | null;
  source_deleted: boolean;
}
