<script lang="ts">
  import { store, formatBytes } from '../store.svelte';
  import { api } from '../api';
  import { save } from '@tauri-apps/plugin-dialog';
  import type { PreviewRow } from '../types';

  interface Props {
    onNewSession?: () => void;
  }

  let { onNewSession }: Props = $props();

  let busy = $state<'open' | 'export' | null>(null);

  const report = $derived(store.transferReport);
  const plan = $derived(store.plan);
  const mode = $derived(store.settings.transfer_mode);

  // Baris gagal / partial, dikelompokkan untuk tabel kegagalan
  const failedRows = $derived(
    store.previewRows.filter((r) => r.status === 'error' || r.status === 'partial')
  );

  const isPartial = $derived(failedRows.length > 0 || store.hasErrors || (report?.items_error ?? 0) > 0);

  const successCount = $derived(report?.items_done ?? store.previewRows.filter((r) => r.status === 'done').length);
  const errorCount = $derived(report?.items_error ?? failedRows.length);
  const skippedCount = $derived(report?.items_skipped ?? store.previewRows.filter((r) => r.status === 'skipped').length);
  const bytesTransferred = $derived(report?.total_bytes_transferred ?? plan?.total_bytes ?? 0);

  const activeDestinations = $derived(store.settings.destinations.filter((d) => d.enabled));

  function durationLabel(): string {
    const ms = report?.duration_ms ?? store.scanReport?.duration_ms ?? 0;
    if (ms <= 0) return '—';
    const totalSec = Math.round(ms / 1000);
    if (totalSec < 60) return `${totalSec} detik`;
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    return `${min} menit ${sec} detik`;
  }

  function destFolderFor(row: PreviewRow): string {
    const d = row.date_override ?? row.date;
    const dest = activeDestinations[0];
    const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest?.suffix ?? '').trim();
    return `${d}_${activity}${suffix ? `_${suffix}` : ''}`;
  }

  async function openTarget() {
    const dest = activeDestinations[0];
    if (!dest?.root) {
      store.globalError = 'Belum ada folder tujuan untuk dibuka.';
      return;
    }
    busy = 'open';
    try {
      await api.openPath(dest.root);
    } catch (err) {
      store.globalError = `Gagal membuka folder tujuan: ${String(err)}`;
    } finally {
      busy = null;
    }
  }

  async function exportCsv() {
    try {
      const target = await save({
        title: 'Ekspor Laporan CSV',
        defaultPath: `laporan-arsip-${new Date().toISOString().slice(0, 10)}.csv`,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (!target) return;
      const logPath = report?.log_path ?? '';
      if (!logPath) {
        store.globalError = 'Laporan sesi belum tersedia.';
        return;
      }
      busy = 'export';
      await api.exportLog(logPath, target);
    } catch (err) {
      store.globalError = `Gagal mengekspor laporan: ${String(err)}`;
    } finally {
      busy = null;
    }
  }

  function newSession() {
    if (onNewSession) onNewSession();
    else store.resetAll();
  }
</script>

<section class="done panel" aria-label="Hasil transfer">
  {#if !isPartial}
    <div class="success-hero">
      <div class="check-circle" aria-hidden="true">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none">
          <path d="m5 12.5 4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </div>
      <h2 class="hero-title">Transfer Selesai!</h2>
      <p class="hero-sub">
        {successCount} file berhasil {mode === 'move' ? 'dipindahkan' : 'disalin'} ({formatBytes(bytesTransferred)})
      </p>
    </div>
  {:else}
    <div class="partial-banner" role="alert">
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 4 2.8 19.5h18.4L12 4Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
        <path d="M12 10v4M12 17v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
      <div>
        <h2>Transfer Selesai dengan Catatan</h2>
        <p>
          {successCount} berhasil
          {#if errorCount > 0}· {errorCount} gagal{/if}
          {#if skippedCount > 0}· {skippedCount} dilewati{/if}
        </p>
      </div>
    </div>
  {/if}

  <div class="summary-grid">
    <div class="summary-card">
      <span class="summary-value">{successCount}</span>
      <span class="summary-label">File berhasil</span>
    </div>
    <div class="summary-card">
      <span class="summary-value">{formatBytes(bytesTransferred)}</span>
      <span class="summary-label">Total ukuran</span>
    </div>
    <div class="summary-card">
      <span class="summary-value">{durationLabel()}</span>
      <span class="summary-label">Durasi</span>
    </div>
    {#if isPartial}
      <div class="summary-card danger">
        <span class="summary-value">{errorCount}</span>
        <span class="summary-label">File gagal</span>
      </div>
    {/if}
  </div>

  {#if isPartial}
    <div class="failure-section">
      <h3>Rincian Kegagalan</h3>
      <div class="fail-table-wrap">
        <table class="fail-table">
          <thead>
            <tr>
              <th scope="col">Nama File</th>
              <th scope="col">Folder Tujuan</th>
              <th scope="col">Status</th>
              <th scope="col">Keterangan</th>
            </tr>
          </thead>
          <tbody>
            {#if failedRows.length === 0}
              <tr>
                <td colspan="4" class="empty-cell">Tidak ada detail per file. Lihat pesan kesalahan di atas.</td>
              </tr>
            {:else}
              {#each failedRows as row (row.source)}
                <tr>
                  <td class="truncate" title={row.source}>{row.name}</td>
                  <td class="mono-sm">{destFolderFor(row)}</td>
                  <td>
                    <span class="badge {row.status === 'partial' ? 'badge-amber' : 'badge-red'}">
                      {row.status === 'partial' ? 'Sebagian' : 'Gagal'}
                    </span>
                  </td>
                  <td class="error-text">{row.error ?? 'Transfer tidak selesai di semua tujuan'}</td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>

      {#if mode === 'move'}
        <p class="move-note">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.8" />
            <path d="M12 11v5.5M12 7.6v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
          </svg>
          <span>File sumber tidak dihapus karena gagal di sebagian tujuan.</span>
        </p>
      {/if}
    </div>
  {/if}

  <div class="actions">
    <button class="btn btn-secondary" type="button" disabled={busy !== null} onclick={openTarget}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M3 7a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.58L11.4 7H19a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
      </svg>
      <span>Buka Folder Tujuan</span>
    </button>
    <button class="btn btn-secondary" type="button" disabled={busy !== null} onclick={exportCsv}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 3v11m0 0 4-4m-4 4-4-4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <span>{busy === 'export' ? 'Mengekspor...' : 'Ekspor Laporan CSV'}</span>
    </button>
    <button class="btn btn-primary" type="button" onclick={newSession}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      </svg>
      <span>Mulai Sesi Baru</span>
    </button>
  </div>
</section>

<style>
  .done {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 24px;
  }

  .success-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 12px 0 4px;
  }

  .check-circle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 68px;
    height: 68px;
    border-radius: 50%;
    background: var(--success-bg);
    color: var(--success);
    margin-bottom: 6px;
  }

  .hero-title {
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
  }

  .hero-sub {
    font-size: 13.5px;
    color: var(--text-3);
  }

  .partial-banner {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--warning-border);
    border-radius: var(--radius);
    background: var(--warning-bg);
    color: var(--warning);
  }

  .partial-banner h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--warning);
  }

  .partial-banner p {
    font-size: 13px;
    margin-top: 2px;
    color: var(--text-2);
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 10px;
  }

  .summary-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
  }

  .summary-card.danger {
    background: var(--danger-bg);
    border-color: #fecaca;
  }

  .summary-card.danger .summary-value {
    color: var(--danger);
  }

  .summary-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .summary-label {
    font-size: 12px;
    color: var(--text-3);
  }

  .failure-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .failure-section h3 {
    font-size: 13.5px;
  }

  .fail-table-wrap {
    max-height: 220px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .fail-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12.5px;
  }

  .fail-table th {
    position: sticky;
    top: 0;
    background: #f8fafc;
    text-align: left;
    padding: 8px 12px;
    font-size: 11px;
    text-transform: uppercase;
    color: var(--text-3);
    border-bottom: 1px solid var(--border);
  }

  .fail-table td {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }

  .error-text {
    color: var(--danger);
    font-size: 12px;
  }

  .empty-cell {
    text-align: center;
    color: var(--text-3);
    padding: 18px;
  }

  .move-note {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--warning);
    margin-top: 4px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
</style>
