<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/store.svelte';
  import { api } from './lib/api';
  import TitleBar from './lib/components/TitleBar.svelte';
  import Footer from './lib/components/Footer.svelte';
  import Toast from './lib/components/Toast.svelte';
  import Modal from './lib/components/Modal.svelte';
  import EmptyState from './lib/components/EmptyState.svelte';
  import SourceQueue from './lib/components/SourceQueue.svelte';
  import DestPanel from './lib/components/DestPanel.svelte';
  import PreviewTable from './lib/components/PreviewTable.svelte';
  import ConfirmModal from './lib/components/ConfirmModal.svelte';
  import TransferProgress from './lib/components/TransferProgress.svelte';
  import DoneState from './lib/components/DoneState.svelte';
  import SettingsModal from './lib/components/SettingsModal.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import type { BackendProgress, BackendItemEvent, PreviewRow } from './lib/types';

  // Toast
  let toastMessage = $state('');
  let toastType = $state<'info' | 'success' | 'warning' | 'error'>('info');
  let showToast = $state(false);
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  // Muat settings saat app start
  onMount(async () => {
    try {
      const settings = await api.loadSettings();
      store.settings = settings;
    } catch (err) {
      showErrorToast(`Gagal memuat pengaturan: ${String(err)}`);
    }

    // Drag & drop global + event progres transfer
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      await getCurrentWebviewWindow().onDragDropEvent((event) => {
        if (event.payload.type === 'drop' && event.payload.paths?.length) {
          // Saat transfer berjalan, abaikan drop baru
          if (store.appState === 'memindahkan') return;
          onDropPaths(event.payload.paths);
        }
      });
    } catch {
      // Tidak berjalan di Tauri — abaikan.
    }

    try {
      const { listen } = await import('@tauri-apps/api/event');

      // Event progres agregat per tujuan
      await listen<BackendProgress>('transfer://progress', (evt) => {
        const p = evt.payload;
        const existing = new Map(store.destProgresses.map((d) => [d.dest_root, d]));
        store.destProgresses = p.per_dest.map((entry) => {
          const prev = existing.get(entry.dest_root);
          const finished = entry.total > 0 && entry.done >= entry.total;
          // Rasio item selesai → estimasi byte selesai (backend belum mengirim byte per tujuan)
          const ratio = entry.total > 0 ? entry.done / entry.total : 0;
          const bytesTotal = prev?.bytes_total ?? 0;
          return {
            dest_id: prev?.dest_id ?? `dest_${entry.dest_root}`,
            dest_root: entry.dest_root,
            dest_folder: prev?.dest_folder ?? '',
            done: entry.done,
            total: entry.total,
            bytes_total: bytesTotal,
            bytes_done: finished ? bytesTotal : Math.round(bytesTotal * ratio),
            status: finished
              ? entry.has_error
                ? ('error' as const)
                : ('done' as const)
              : entry.done > 0
                ? ('running' as const)
                : ('waiting' as const),
          };
        });

        // Tandai baris pratinjau yang sedang diproses
        if (p.current_source) {
          store.previewRows = store.previewRows.map((r) =>
            r.source === p.current_source ? { ...r, status: 'copying' } : r,
          );
        }
      });

      // Event per-item (optional untuk UI detail)
      await listen<BackendItemEvent>('transfer://item', (evt) => {
        const item = evt.payload;
        // Update status baris tabel dari nama item
        // Mungkin tidak perlu untuk MVP, tapi siap untuk nanti
      });
    } catch (err) {
      // Tauri events tidak tersedia atau terjadi error — OK di dev mode
    }
  });

  function showErrorToast(msg: string) {
    toastMessage = msg;
    toastType = 'error';
    showToast = true;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      showToast = false;
    }, 4000);
  }

  function showSuccessToast(msg: string) {
    toastMessage = msg;
    toastType = 'success';
    showToast = true;
    if (toastTimeout) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      showToast = false;
    }, 3000);
  }

  // ── State Machine Transitions ──

  async function onAddSource(path: string) {
    store.addSourcePath(path);
  }

  async function onDropPaths(paths: string[]) {
    let added = false;
    for (const p of paths) {
      if (!p) continue;
      store.addSourcePath(p);
      added = true;
    }
    if (added) await onRescanSources();
  }

  async function onBrowseSource() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: true,
        title: 'Pilih folder sumber dokumentasi',
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      let added = false;
      for (const p of paths) {
        if (p) {
          store.addSourcePath(p);
          added = true;
        }
      }
      // Dari state kosong: langsung pindai sumber pertama kali
      if (added && store.appState === 'kosong') await onRescanSources();
    } catch (err) {
      showErrorToast(`Gagal membuka dialog: ${String(err)}`);
    }
  }

  async function onRescanSources() {
    if (store.sourcePaths.length === 0) {
      showErrorToast('Belum ada folder sumber untuk dipindai');
      return;
    }

    store.appState = 'memindai';
    try {
      const report = await api.scanSources(store.sourcePaths);
      store.scanReport = report;
      store.scannedFiles = report.files || [];

      // Ekstrak metadata untuk semua file
      const dateMap = await api.extractMetadata(report.files || []);
      store.dateMap = dateMap;

      // Bangun preview rows dari files + metadata
      const rows: PreviewRow[] = (report.files || []).map((f) => {
        const meta = dateMap[f.path] || {
          date: new Date().toISOString().slice(0, 10),
          year: new Date().getFullYear(),
          month: new Date().getMonth() + 1,
          day: new Date().getDate(),
          source: 'file_modified' as const,
          needs_review: true,
        };
        return {
          source: f.path,
          name: f.name,
          size_bytes: f.size_bytes,
          kind: f.kind,
          extension: f.extension,
          date: meta.date,
          month: meta.month,
          day: meta.day,
          metadata_source: meta.source,
          needs_review: meta.needs_review,
          status: 'pending' as const,
        };
      });
      store.previewRows = rows;

      store.appState = 'siap';
      showSuccessToast(`Pemindaian selesai: ${report.total_files} file ditemukan`);
    } catch (err) {
      store.appState = 'kosong';
      showErrorToast(`Pemindaian gagal: ${String(err)}`);
    }
  }

  async function onConfirmTransfer() {
    if (store.previewRows.length === 0 || store.selectedRows.size === 0) {
      showErrorToast('Pilih minimal satu file untuk ditransfer');
      return;
    }

    const activeDestinations = store.settings.destinations.filter((d) => d.enabled);
    if (activeDestinations.length === 0) {
      showErrorToast('Aktifkan minimal satu tujuan pengarsipan');
      return;
    }

    store.appState = 'konfirmasi';
    store.showConfirm = true;
  }

  async function onStartTransfer() {
    store.showConfirm = false;
    store.appState = 'memindahkan';

    try {
      const activeDestinations = store.settings.destinations.filter((d) => d.enabled);
      const selectedRowsSet = store.selectedRows;
      const selectedFiles = store.scannedFiles.filter((f) => selectedRowsSet.has(f.path));

      // Gabungkan metadata asli dengan date override manual dari user
      const dates: typeof store.dateMap = { ...store.dateMap };
      for (const row of store.previewRows) {
        if (row.date_override) {
          const [y, m, d] = row.date_override.split('-').map(Number);
          dates[row.source] = {
            date: row.date_override,
            year: y,
            month: m,
            day: d,
            source: 'manual',
            needs_review: false,
          };
        }
      }

      // Simpan settings terbaru (mode transfer, tujuan) sebelum mulai
      try {
        await api.saveSettings(store.settings);
      } catch {
        // Gagal simpan settings tidak boleh menghentikan transfer
      }

      const plan = await api.buildPlan(selectedFiles, dates, activeDestinations);
      store.plan = plan;

      // Inisialisasi progres per tujuan dari plan
      const perRoot = new Map<string, { done: number; total: number; bytesTotal: number; hasError: boolean }>();
      for (const it of plan.items) {
        const e = perRoot.get(it.dest_root) ?? { done: 0, total: 0, bytesTotal: 0, hasError: false };
        e.total += 1;
        e.bytesTotal += it.size_bytes;
        perRoot.set(it.dest_root, e);
      }
      store.destProgresses = Array.from(perRoot.entries()).map(([root, e], i) => ({
        dest_id: `dest_${i}`,
        dest_root: root,
        dest_folder: '',
        done: 0,
        total: e.total,
        bytes_done: 0,
        bytes_total: e.bytesTotal,
        status: 'waiting' as const,
      }));

      // Mulai transfer dengan source roots untuk cleanup folder kosong
      const sourceRoots = Array.from(new Set(store.sourcePaths));
      const report = await api.startTransfer(plan, store.settings.transfer_mode, sourceRoots);

      store.transferReport = report;
      store.logEntries = report.log_entries ?? [];

      if (report.cancelled) {
        store.appState = 'ada_masalah';
        showErrorToast('Transfer dibatalkan');
      } else if (report.items_error > 0) {
        store.appState = 'ada_masalah';
        showErrorToast(`Transfer selesai dengan ${report.items_error} item gagal`);
      } else {
        store.appState = 'selesai';
        showSuccessToast('Transfer selesai!');
      }
    } catch (err) {
      store.appState = 'ada_masalah';
      showErrorToast(`Transfer gagal: ${String(err)}`);
    }
  }

  function onCancelTransfer() {
    api.cancelTransfer().catch((err) => console.error('Batalkan gagal:', err));
    store.appState = 'ada_masalah';
    showErrorToast('Transfer dibatalkan');
  }

  function onNewSession() {
    store.resetAll();
    store.appState = 'kosong';
  }

  function onShowSettings() {
    store.showSettings = true;
  }

  function onCloseSettings() {
    store.showSettings = false;
  }

  const footerStatus = $derived.by(() => {
    switch (store.appState) {
      case 'memindai':
        return 'Memindai...';
      case 'memindahkan':
        return `Transfer ${store.transferProgress}%`;
      case 'ada_masalah':
        return 'Ada masalah';
      case 'selesai':
        return 'Selesai';
      case 'konfirmasi':
        return 'Siap dikonfirmasi';
      default:
        return 'Siap';
    }
  });

  const footerBusy = $derived(store.appState === 'memindai' || store.appState === 'memindahkan');
</script>

<div class="app-container">
  <TitleBar onSettings={onShowSettings} />

  <main class="main-content">
    {#if store.appState === 'kosong'}
      <EmptyState onBrowse={onBrowseSource} onDropFiles={onDropPaths} />
    {:else}
      <div class="layout-grid">
        <div class="col-left">
          <SourceQueue
            onAddPath={onAddSource}
            onRescan={onRescanSources}
            rescanDisabled={store.appState !== 'siap'}
          />
          <DestPanel />
        </div>

        <div class="col-right">
          {#if store.appState === 'memindai'}
            <div class="loading-state">
              <div class="spinner" aria-hidden="true"></div>
              <p>Memindai folder sumber...</p>
            </div>
          {:else if store.appState === 'siap' || store.appState === 'konfirmasi'}
            <PreviewTable />
          {:else if store.appState === 'memindahkan'}
            <TransferProgress onCancel={onCancelTransfer} />
          {:else if store.appState === 'selesai' || store.appState === 'ada_masalah'}
            <DoneState onNewSession={onNewSession} />
          {/if}
        </div>
      </div>
    {/if}
  </main>

  <!-- Tombol aksi utama: ditampilkan di bagian bawah saat siap -->
  {#if store.appState === 'siap'}
    <div class="action-bar">
      <button class="btn btn-primary btn-lg" onclick={onConfirmTransfer}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M14 15v2.5H8v-2.5M12 3v9m0-9l3 3m-3-3l-3 3"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>Transfer Sekarang</span>
      </button>
    </div>
  {/if}

  <!-- Modal Konfirmasi -->
  <ConfirmModal
    open={store.showConfirm}
    onCancel={() => {
      store.showConfirm = false;
      store.appState = 'siap';
    }}
    onStart={onStartTransfer}
  />

  <!-- Modal Pengaturan -->
  <SettingsModal open={store.showSettings} onClose={onCloseSettings} />

  <!-- Toast -->
  {#if showToast && toastMessage}
    <div class="toast-container">
      <Toast message={toastMessage} type={toastType} onClose={() => (showToast = false)} />
    </div>
  {/if}

  <Footer leftText={footerStatus} rightText="v0.1.0" busy={footerBusy} />
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg);
    color: var(--text-1);
  }

  .main-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .layout-grid {
    display: grid;
    grid-template-columns: 400px 1fr;
    gap: 0;
    height: 100%;
    overflow: hidden;
  }

  .col-left {
    display: flex;
    flex-direction: column;
    gap: 0;
    overflow-y: auto;
    border-right: 1px solid var(--border);
    background: var(--card);
  }

  .col-right {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .action-bar {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    background: var(--bg);
    display: flex;
    gap: 8px;
  }

  .btn-lg {
    padding: 10px 24px;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--text-2);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .toast-container {
    position: fixed;
    bottom: 60px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
  }

  @media (max-width: 1200px) {
    .layout-grid {
      grid-template-columns: 320px 1fr;
    }
  }

  @media (max-width: 900px) {
    .layout-grid {
      grid-template-columns: 1fr;
    }

    .col-left {
      display: none;
    }
  }
</style>
