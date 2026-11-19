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

export const api = {
  getVersion(): Promise<string> {
    return invoke<string>('get_version');
  },

  loadSettings(): Promise<Settings> {
    return invoke<Settings>('load_settings');
  },

  saveSettings(s: Settings): Promise<void> {
    return invoke<void>('save_settings', { s });
  },

  scanSources(sources: string[]): Promise<ScanReport> {
    return invoke<ScanReport>('scan_sources', { sources });
  },

  extractMetadata(files: ScannedFile[]): Promise<Record<string, DateMetadata>> {
    return invoke<Record<string, DateMetadata>>('extract_metadata', { files });
  },

  buildPlan(
    files: ScannedFile[],
    dates: Record<string, DateMetadata>,
    destinations: Settings['destinations'],
  ): Promise<TransferPlan> {
    return invoke<TransferPlan>('build_plan', { files, dates, destinations });
  },

  startTransfer(
    plan: TransferPlan,
    mode: Settings['transfer_mode'],
    sourceRoots: string[] = [],
  ): Promise<TransferReport> {
    return invoke<TransferReport>('start_transfer', {
      plan,
      mode,
      sourceRoots,
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
