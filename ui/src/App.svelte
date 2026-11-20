<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/store.svelte';
  import { api } from './lib/api';
  import TitleBar from './lib/components/TitleBar.svelte';
  import Footer from './lib/components/Footer.svelte';
  import Toast from './lib/components/Toast.svelte';
  import SourceQueue from './lib/components/SourceQueue.svelte';
  import DestPanel from './lib/components/DestPanel.svelte';
  import PreviewTable from './lib/components/PreviewTable.svelte';
  import ConfirmModal from './lib/components/ConfirmModal.svelte';
  import TransferProgress from './lib/components/TransferProgress.svelte';
  import DoneState from './lib/components/DoneState.svelte';
  import SettingsModal from './lib/components/SettingsModal.svelte';
  import WelcomeState from './lib/components/WelcomeState.svelte';
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

    // Sinkronkan versi aplikasi dari backend
    try {
      store.appVersion = await api.getVersion();
    } catch {
      /* biarkan fallback versi default */
    }

    // Pastikan ada minimal satu tujuan
    if (store.settings.destinations.length === 0) {
      store.addDestination();
    }

    // Pilih semua baris setelah pemindaian selesai (default select-all)
    // (diatur di onRescanSources)

    // Drag & drop global + event progres transfer
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      await getCurrentWebviewWindow().onDragDropEvent((event) => {
        if (event.payload.type === 'drop' && event.payload.paths?.length) {
          if (store.appState === 'memindahkan') return;
          onDropPaths(event.payload.paths);
        }
      });
    } catch {
      // Tidak berjalan di Tauri — abaikan.
    }

    try {
      const { listen } = await import('@tauri-apps/api/event');

      await listen<BackendProgress>('transfer://progress', (evt) => {
        const p = evt.payload;
        const existing = new Map(store.destProgresses.map((d) => [d.dest_root, d]));
        store.destProgresses = p.per_dest.map((entry) => {
          const prev = existing.get(entry.dest_root);
          const finished = entry.total > 0 && entry.done >= entry.total;
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

        if (p.current_source) {
          store.previewRows = store.previewRows.map((r) =>
            r.source === p.current_source ? { ...r, status: 'copying' } : r,
          );
        }
      });

      await listen<BackendItemEvent>('transfer://item', () => {
        // placeholder untuk detail per-item
      });
    } catch {
      // Tauri events tidak tersedia — OK di dev mode
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

      const dateMap = await api.extractMetadata(report.files || []);
      store.dateMap = dateMap;

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

      // Default: semua baris terseleksi
      store.selectedRows = new Set(rows.map((r) => r.source));

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
    if (store.settings.confirm_before_start) {
      store.appState = 'konfirmasi';
      store.showConfirm = true;
    } else {
      await onStartTransfer();
    }
  }

  async function onStartTransfer() {
    store.showConfirm = false;
    store.appState = 'memindahkan';

    try {
      const activeDestinations = store.settings.destinations.filter((d) => d.enabled);
      const selectedRowsSet = store.selectedRows;
      const selectedFiles = store.scannedFiles.filter((f) => selectedRowsSet.has(f.path));

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

      try {
        await api.saveSettings(store.settings);
      } catch {
        // Gagal simpan settings tidak boleh menghentikan transfer
      }

      const plan = await api.buildPlan(selectedFiles, dates, activeDestinations);
      store.plan = plan;

      const perRoot = new Map<string, { done: number; total: number; bytesTotal: number }>();
      for (const it of plan.items) {
        const e = perRoot.get(it.dest_root) ?? { done: 0, total: 0, bytesTotal: 0 };
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
    if (store.settings.destinations.length === 0) store.addDestination();
  }

  function onShowSettings() {
    store.showSettings = true;
  }

  function onCloseSettings() {
    store.showSettings = false;
  }

  const showWelcome = $derived(store.appState === 'welcome');
  const showUpperPanels = $derived(!showWelcome && store.appState !== 'memindahkan' && store.appState !== 'selesai' && store.appState !== 'ada_masalah');
  const showLowerTable = $derived(showUpperPanels);
  const showTransfer = $derived(store.appState === 'memindahkan');
  const showDone = $derived(store.appState === 'selesai' || store.appState === 'ada_masalah');
</script>

<div class="app-container">
  <TitleBar onSettings={onShowSettings} />

  <main class="main-content">
    {#if showWelcome}
      <WelcomeState />
    {:else if showUpperPanels}
      <!-- Upper row: Panel Sumber (580) + Panel Tujuan (560) -->
      <div class="upper-row" class:tall={store.appState !== 'kosong'}>
        <SourceQueue
          onAddPath={onAddSource}
          onRescan={onRescanSources}
          rescanDisabled={store.appState !== 'siap' && store.appState !== 'kosong'}
          tall={store.appState !== 'kosong'}
        />
        <DestPanel tall={store.appState !== 'kosong'} />
      </div>
    {/if}

    {#if showLowerTable}
      <div class="lower-row">
        {#if store.appState === 'memindai'}
          <div class="scan-overlay">
            <div class="spinner" aria-hidden="true"></div>
            <p>Memindai folder sumber…</p>
          </div>
        {:else}
          <PreviewTable />
        {/if}
      </div>
    {:else if showTransfer}
      <div class="full-card transfer-card">
        <TransferProgress onCancel={onCancelTransfer} />
      </div>
    {:else if showDone}
      <div class="full-card">
        <DoneState onNewSession={onNewSession} />
      </div>
    {/if}
  </main>

  <div class="footer-wrap" class:hidden={showWelcome}>
    <Footer onStart={onConfirmTransfer} busy={store.appState === 'memindai' || store.appState === 'memindahkan'} onReset={onNewSession} />
  </div>

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
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100%;
    background: var(--bg);
    color: var(--text);
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 20px 0;
    box-sizing: border-box;
  }

  .upper-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 20px;
    flex: 0 0 auto;
    align-items: start;
    overflow: hidden;
    min-width: 0;
  }

  .upper-row > :global(*) {
    min-width: 0;
  }

  .upper-row.tall {
    min-height: 320px;
  }

  .lower-row {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    position: relative;
    min-width: 0;
  }

  .full-card {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-sizing: border-box;
  }

  /* TransferProgress/DoneState render their own panel chrome */
  .full-card.transfer-card {
    background: transparent;
    border: none;
    border-radius: 0;
    overflow: hidden;
  }

  .footer-wrap {
    flex: 0 0 auto;
    padding: 16px 20px;
    box-sizing: border-box;
  }

  .footer-wrap.hidden {
    display: none;
  }

  .scan-overlay {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: var(--text-2);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
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
    bottom: 90px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
  }

  /* ── Responsif: jendela sempit / snap 2-side ───────── */
  @media (max-width: 1080px) {
    .main-content {
      padding: 12px 14px 0;
      gap: 12px;
    }

    .upper-row {
      gap: 12px;
    }

    .footer-wrap {
      padding: 12px 14px;
    }
  }

  /* Panels stack vertically; keep component-level rules authoritative. */
  @media (max-width: 860px) {
    .upper-row,
    .upper-row.tall {
      grid-template-columns: minmax(0, 1fr);
      min-height: 0;
    }

    .main-content {
      overflow-y: auto;
    }

    .lower-row {
      min-height: 300px;
    }
  }
</style>
