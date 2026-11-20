// D:\Project\ArsipDok\ui\src\lib\store.svelte.ts

import type {
  Settings,
  ScannedFile,
  DateMetadata,
  PreviewRow,
  DestProgress,
  LogEntry,
  TransferPlan,
  ScanReport,
  TransferReport,
  MetadataSource,
} from './types';

export type AppState =
  | 'welcome'
  | 'kosong'
  | 'memindai'
  | 'siap'
  | 'konfirmasi'
  | 'memindahkan'
  | 'selesai'
  | 'ada_masalah';

function createStore() {
  let appState = $state<AppState>('welcome');
  let sourcePaths = $state<string[]>([]);
  let scanReport = $state<ScanReport | null>(null);
  let scannedFiles = $state<ScannedFile[]>([]);
  let dateMap = $state<Record<string, DateMetadata>>({});
  let previewRows = $state<PreviewRow[]>([]);
  let selectedRows = $state<Set<string>>(new Set());
  let plan = $state<TransferPlan | null>(null);
  let destProgresses = $state<DestProgress[]>([]);
  let transferReport = $state<TransferReport | null>(null);
  let logEntries = $state<LogEntry[]>([]);
  let theme = $state<'system' | 'light' | 'dark'>('system');
  let appVersion = $state('0.1.0');
  let showSettings = $state(false);
  let showConfirm = $state(false);
  let showDateTooltip = $state<string | null>(null);
  let globalError = $state<string | null>(null);

  let settings = $state<Settings>({
    version: 2,
    destinations: [],
    root_history: [],
    activity_history: [],
    suffix_history: [],
    transfer_mode: 'copy',
    verification: 'size_mtime',
    parallel_workers: 4,
    remove_empty_source_dirs: true,
    confirm_before_start: true,
    language: 'id',
  });

  const totalSourceSize = $derived(
    scannedFiles.reduce((s, f) => s + f.size_bytes, 0)
  );

  const needsReviewCount = $derived(
    previewRows.filter(r => r.needs_review && !r.date_override).length
  );

  const hasErrors = $derived(
    previewRows.some(r => r.status === 'error' || r.status === 'partial')
  );

  const totalDone = $derived(
    destProgresses.reduce((s, d) => s + d.done, 0)
  );
  const totalItems = $derived(
    destProgresses.reduce((s, d) => s + d.total, 0)
  );
  const transferProgress = $derived(
    totalItems === 0 ? 0 : Math.round((totalDone / totalItems) * 100)
  );

  return {
    // State getters
    get appState() { return appState; },
    get sourcePaths() { return sourcePaths; },
    get scanReport() { return scanReport; },
    get scannedFiles() { return scannedFiles; },
    get dateMap() { return dateMap; },
    get previewRows() { return previewRows; },
    get selectedRows() { return selectedRows; },
    get plan() { return plan; },
    get destProgresses() { return destProgresses; },
    get transferReport() { return transferReport; },
    get logEntries() { return logEntries; },
    get showSettings() { return showSettings; },
    get showConfirm() { return showConfirm; },
    get showDateTooltip() { return showDateTooltip; },
    get globalError() { return globalError; },
    get settings() { return settings; },
    get theme() { return theme; },
    get appVersion() { return appVersion; },

    // Derived
    get totalSourceSize() { return totalSourceSize; },
    get needsReviewCount() { return needsReviewCount; },
    get hasErrors() { return hasErrors; },
    get transferProgress() { return transferProgress; },

    // Setters / Actions
    set appState(v: AppState) { appState = v; },
    set sourcePaths(v: string[]) { sourcePaths = v; },
    set scanReport(v: ScanReport | null) { scanReport = v; },
    set scannedFiles(v: ScannedFile[]) { scannedFiles = v; },
    set dateMap(v: Record<string, DateMetadata>) { dateMap = v; },
    set previewRows(v: PreviewRow[]) { previewRows = v; },
    set selectedRows(v: Set<string>) { selectedRows = v; },
    set plan(v: TransferPlan | null) { plan = v; },
    set destProgresses(v: DestProgress[]) { destProgresses = v; },
    set transferReport(v: TransferReport | null) { transferReport = v; },
    set logEntries(v: LogEntry[]) { logEntries = v; },
    set showSettings(v: boolean) { showSettings = v; },
    set showConfirm(v: boolean) { showConfirm = v; },
    set showDateTooltip(v: string | null) { showDateTooltip = v; },
    set globalError(v: string | null) { globalError = v; },
    set settings(v: Settings) { settings = v; },
    set theme(v: 'system' | 'light' | 'dark') { theme = v; },
    set appVersion(v: string) { appVersion = v; },

    resetAll() {
      appState = 'kosong';
      sourcePaths = [];
      scanReport = null;
      scannedFiles = [];
      dateMap = {};
      previewRows = [];
      selectedRows = new Set();
      plan = null;
      destProgresses = [];
      transferReport = null;
      logEntries = [];
      globalError = null;
      showConfirm = false;
      showDateTooltip = null;
    },

    addSourcePath(p: string) {
      if (!sourcePaths.includes(p)) sourcePaths = [...sourcePaths, p];
    },

    removeSourcePath(p: string) {
      sourcePaths = sourcePaths.filter(x => x !== p);
    },

    setDateOverride(sourcePath: string, date: string) {
      previewRows = previewRows.map(r =>
        r.source === sourcePath
          ? { ...r, date_override: date, needs_review: false }
          : r
      );
    },

    toggleRowSelect(path: string) {
      const s = new Set(selectedRows);
      if (s.has(path)) s.delete(path); else s.add(path);
      selectedRows = s;
    },

    selectAll() {
      selectedRows = new Set(previewRows.map(r => r.source));
    },

    deselectAll() {
      selectedRows = new Set();
    },

    addDestination() {
      const id = crypto.randomUUID();
      settings = {
        ...settings,
        destinations: [
          ...settings.destinations,
          { id, root: '', activity: '', suffix: undefined, enabled: true }
        ]
      };
    },

    removeDestination(id: string) {
      settings = {
        ...settings,
        destinations: settings.destinations.filter(d => d.id !== id)
      };
    },

    updateDestination(id: string, patch: Partial<{ root: string; activity: string; suffix: string; enabled: boolean }>) {
      settings = {
        ...settings,
        destinations: settings.destinations.map(d =>
          d.id === id ? { ...d, ...patch } : d
        )
      };
    },
  };
}

export const store = createStore();

// ── Helpers ───────────────────────────────────────────────────────
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export function sourceLabel(src: MetadataSource | string): string {
  const map: Record<string, string> = {
    exif: 'EXIF',
    video: 'Video',
    file_creation: 'Waktu Dibuat',
    file_modified: 'Waktu Diubah',
    manual: 'Manual',
  };
  return map[src] ?? src;
}

export function metaSourceBadgeColor(src: MetadataSource | string): string {
  if (src === 'exif' || src === 'video') return '#16A34A';
  if (src === 'manual') return '#0F62FE';
  return '#B45309'; // file timestamps → perlu review
}
