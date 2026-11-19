// Wrapper tipis di atas @tauri-apps/api/core untuk semua command IPC.
import { invoke } from '@tauri-apps/api/core';
import type {
  Settings,
  ScannedFile,
  DateMetadata,
  TransferPlan,
  ScanReport,
  TransferReport,
  DiskSpaceInfo,
} from './types';

// Proxy $state Svelte TIDAK bisa di-structuredClone oleh IPC Tauri
// (DataCloneError). Kita serialisasi dulu ke JSON lalu parse kembali
// untuk mendapatkan objek polos (plain object) yang bisa di-clone.
function plain<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export const api = {
  getVersion(): Promise<string> {
    return invoke<string>('get_version');
  },

  loadSettings(): Promise<Settings> {
    return invoke<Settings>('load_settings');
  },

  saveSettings(s: Settings): Promise<void> {
    return invoke<void>('save_settings', { s: plain(s) });
  },

  scanSources(sources: string[]): Promise<ScanReport> {
    return invoke<ScanReport>('scan_sources', { sources: plain(sources) });
  },

  extractMetadata(files: ScannedFile[]): Promise<Record<string, DateMetadata>> {
    return invoke<Record<string, DateMetadata>>('extract_metadata', { files: plain(files) });
  },

  buildPlan(
    files: ScannedFile[],
    dates: Record<string, DateMetadata>,
    destinations: Settings['destinations'],
  ): Promise<TransferPlan> {
    return invoke<TransferPlan>('build_plan', {
      files: plain(files),
      dates: plain(dates),
      destinations: plain(destinations),
    });
  },

  startTransfer(
    plan: TransferPlan,
    mode: Settings['transfer_mode'],
    sourceRoots: string[] = [],
  ): Promise<TransferReport> {
    return invoke<TransferReport>('start_transfer', {
      plan: plain(plan),
      mode: plain(mode),
      sourceRoots: plain(sourceRoots),
    });
  },

  cancelTransfer(): Promise<void> {
    return invoke<void>('cancel_transfer');
  },

  exportLog(logPath: string, exportPath: string): Promise<void> {
    return invoke<void>('export_log', { logPath, exportPath });
  },

  checkDiskSpace(path: string): Promise<DiskSpaceInfo> {
    return invoke<DiskSpaceInfo>('check_disk_space', { path });
  },

  validateDestination(path: string): Promise<boolean> {
    return invoke<boolean>('validate_destination', { path });
  },

  openPath(path: string): Promise<void> {
    return invoke<void>('open_path', { path });
  },
};
