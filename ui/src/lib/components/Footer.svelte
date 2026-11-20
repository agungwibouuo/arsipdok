<script lang="ts">
  import { store, formatBytes } from '../store.svelte';
  import { api } from '../api';
  import { save } from '@tauri-apps/plugin-dialog';

  interface Props {
    onStart?: () => void;
    busy?: boolean;
    onReset?: () => void;
  }

  let { onStart, busy = false, onReset }: Props = $props();

  const appState = $derived(store.appState);
  const rows = $derived(store.previewRows);
  const selectedCount = $derived(store.selectedRows.size);
  const totalBytes = $derived(
    rows
      .filter((r) => store.selectedRows.has(r.source))
      .reduce((s, r) => s + r.size_bytes, 0),
  );

  // Group badges: unique folder names derived from preview rows
  const folderGroups = $derived.by(() => {
    const dest = store.settings.destinations.find((d) => d.enabled);
    if (!dest || rows.length === 0) return [];
    const activity = (dest.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest.suffix ?? '').trim();
    const seen = new Set<string>();
    for (const r of rows) {
      if (!store.selectedRows.has(r.source)) continue;
      const d = r.date_override ?? r.date;
      const parts = d.split('-');
      const mmDd = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d;
      seen.add(`${mmDd}_${activity}${suffix ? `_${suffix}` : ''}`);
    }
    return Array.from(seen).slice(0, 4);
  });

  const isReady = $derived(appState === 'siap' || appState === 'konfirmasi');
  const isTransferring = $derived(appState === 'memindahkan');
  const isDone = $derived(appState === 'selesai' || appState === 'ada_masalah');
  const isScanning = $derived(appState === 'memindai');

  function statusText(): string {
    if (isTransferring) return `Memindahkan… ${store.transferProgress}%`;
    if (isScanning) return 'Memindai sumber…';
    if (isDone && appState === 'selesai') return 'Transfer selesai!';
    if (isDone) return 'Ada masalah';
    if (selectedCount > 0) return `Total: ${selectedCount} file • ${formatBytes(totalBytes)}`;
    if (rows.length > 0) return `${rows.length} file ditemukan — pilih untuk transfer`;
    return 'Belum ada file — tambahkan sumber dokumentasi';
  }

  function btnLabel(): string {
    if (isTransferring) return 'Memindahkan…';
    if (isScanning) return 'Memindai…';
    return '▶ Mulai Salin';
  }

  const canStart = $derived(isReady && selectedCount > 0 && !busy);
  async function exportCsv() {
    try {
      const target = await save({
        title: 'Ekspor Laporan CSV',
        defaultPath: `laporan-arsip-${new Date().toISOString().slice(0, 10)}.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (!target) return;
      const logPath = store.transferReport?.log_path ?? '';
      if (!logPath) {
        store.globalError = 'Laporan sesi belum tersedia.';
        return;
      }
      await api.exportLog(logPath, target);
    } catch (err) {
      store.globalError = `Gagal mengekspor laporan: ${String(err)}`;
    }
  }
</script>

<footer class="footer-bar" aria-label="Status dan aksi utama">
  <div class="footer-left">
    {#if folderGroups.length > 0}
      <span class="groups-label">{isDone ? 'Folder dibuat' : 'Folder akan dibuat'} ({folderGroups.length}):</span>
      <div class="badges">
        {#each folderGroups as grp}
          <span class="folder-badge">📁 {grp}</span>
        {/each}
      </div>
    {:else}
      <span class="status-text">{statusText()}</span>
    {/if}
  </div>

  <div class="footer-right">
    {#if selectedCount > 0 && !isTransferring && !isDone}
      <span class="summary-text">Total: {selectedCount} file • {formatBytes(totalBytes)}</span>
    {/if}

    {#if !isDone}
      <button
        class="btn-start"
        type="button"
        disabled={!canStart}
        onclick={onStart}
        aria-label="Mulai transfer file"
      >
        {btnLabel()}
      </button>
    {:else}
      <button class="btn-footer-csv" type="button" onclick={exportCsv}>
        ⬇ Ekspor CSV
      </button>
      <button class="btn-footer-reset" type="button" onclick={() => onReset?.()}>
        ✓ &nbsp;Selesai / Reset
      </button>
    {/if}

    <span class="version-note">ArsipDok v{store.appVersion} — Pengarsip Dokumentasi Foto &amp; Video</span>
  </div>
</footer>

<style>
  .footer-bar {
    height: 62px;
    flex: 0 0 auto;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    gap: 12px;
    box-sizing: border-box;
  }

  .footer-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .groups-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-3);
    white-space: nowrap;
    flex: 0 0 auto;
  }

  .badges {
    display: flex;
    gap: 6px;
    overflow: hidden;
  }

  .folder-badge {
    display: inline-flex;
    align-items: center;
    height: 24px;
    padding: 0 8px;
    background: #F0FDF4;
    border: 1px solid #BBF7D0;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
    color: #166534;
    white-space: nowrap;
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-text {
    font-size: 12px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .footer-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 0 0 auto;
  }

  .version-note {
    font-size: 10px;
    color: var(--text-3);
    white-space: nowrap;
    padding-left: 10px;
    border-left: 1px solid var(--border);
    line-height: 1.2;
  }

  .summary-text {
    font-size: 12px;
    color: var(--text-2);
    white-space: nowrap;
  }

  .btn-start {
    width: 154px;
    height: 36px;
    background: #0F62FE;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    font-family: inherit;
    white-space: nowrap;
    transition: background 0.12s ease, opacity 0.12s ease;
  }

  .btn-start:hover:not(:disabled) {
    background: var(--primary-hover);
  }

  .btn-start:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-footer-csv {
    height: 36px;
    padding: 0 14px;
    background: var(--card);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    color: #0066CC;
    cursor: pointer;
    font-family: inherit;
    white-space: nowrap;
  }

  .btn-footer-csv:hover {
    background: var(--bg);
  }

  .btn-footer-reset {
    height: 36px;
    padding: 0 18px;
    background: #0F62FE;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
    cursor: pointer;
    font-family: inherit;
    white-space: nowrap;
  }

  .btn-footer-reset:hover {
    background: var(--primary-hover);
  }
</style>
