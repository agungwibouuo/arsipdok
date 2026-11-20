<script lang="ts">
  import { store, formatBytes } from '../store.svelte';
  import { api } from '../api';
  import { save } from '@tauri-apps/plugin-dialog';
  import type { PreviewRow, LogEntry } from '../types';

  interface Props {
    onNewSession?: () => void;
  }

  let { onNewSession }: Props = $props();

  let busy = $state<'open' | 'export' | null>(null);

  const report = $derived(store.transferReport);
  const plan = $derived(store.plan);
  const mode = $derived(store.settings.transfer_mode);

  const failedRows = $derived(
    store.previewRows.filter((r) => r.status === 'error' || r.status === 'partial')
  );

  const isPartial = $derived(failedRows.length > 0 || store.hasErrors || (report?.items_error ?? 0) > 0);

  const successCount = $derived(report?.items_done ?? store.previewRows.filter((r) => r.status === 'done').length);
  const errorCount = $derived(report?.items_error ?? failedRows.length);
  const skippedCount = $derived(report?.items_skipped ?? store.previewRows.filter((r) => r.status === 'skipped').length);
  const bytesTransferred = $derived(report?.total_bytes_transferred ?? plan?.total_bytes ?? 0);
  const totalSelected = $derived(plan?.total_files ?? store.previewRows.length);

  const activeDestinations = $derived(store.settings.destinations.filter((d) => d.enabled));

  const folderNames = $derived(
    plan?.folder_names && plan.folder_names.length > 0
      ? plan.folder_names
      : Array.from(
          new Set(
            store.previewRows.map((r) => {
              const d = r.date_override ?? r.date;
              const dest = activeDestinations[0];
              const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
              const suffix = (dest?.suffix ?? '').trim();
              const parts = d.split('-');
              const mmDd = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d;
              return `${mmDd}_${activity}${suffix ? `_${suffix}` : ''}`;
            })
          )
        )
  );

  const logEntries = $derived(store.logEntries);

  function durationLabel(): string {
    const ms = report?.duration_ms ?? store.scanReport?.duration_ms ?? 0;
    if (ms <= 0) return '—';
    const totalSec = Math.round(ms / 1000);
    if (totalSec < 60) return `${totalSec} detik`;
    const min = Math.floor(totalSec / 60);
    const sec = totalSec % 60;
    return `${min} menit ${sec} detik`;
  }

  function formatDisplayDate(dateStr: string): string {
    const parts = dateStr.split('-');
    if (parts.length === 3) {
      const months = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agu', 'Sep', 'Okt', 'Nov', 'Des'];
      const d = parseInt(parts[2], 10);
      const m = parseInt(parts[1], 10) - 1;
      return `${d} ${months[m] ?? parts[1]}`;
    }
    return dateStr;
  }

  function destFolderFor(row: PreviewRow): string {
    const d = row.date_override ?? row.date;
    const dest = activeDestinations[0];
    const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest?.suffix ?? '').trim();
    const parts = d.split('-');
    const mmDd = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d;
    return `${mmDd}_${activity}${suffix ? `_${suffix}` : ''}`;
  }

  const VIDEO_EXT = new Set(['.mp4', '.mov', '.avi', '.mkv', '.3gp', '.m4v', '.wmv', '.mts', '.hevc']);

  function isVideoFile(path: string): boolean {
    const dot = path.lastIndexOf('.');
    const ext = dot >= 0 ? path.slice(dot).toLowerCase() : '';
    return VIDEO_EXT.has(ext);
  }

  // Map source_path → preview row (untuk tanggal) agar tabel log & preview konsisten
  const rowBySource = $derived.by(() => {
    const map = new Map<string, PreviewRow>();
    for (const r of store.previewRows) map.set(r.source, r);
    return map;
  });

  function infoForEntry(entry: LogEntry): { name: string; isVideo: boolean; date: string; folder: string } {
    const name = entry.source_path.split(/[\\/]/).pop() ?? entry.source_path;
    const row = rowBySource.get(entry.source_path);
    const dateStr = row ? (row.date_override ?? row.date) : '';
    const date = dateStr ? formatDisplayDate(dateStr) : '—';
    return {
      name,
      isVideo: isVideoFile(entry.source_path),
      date,
      folder: destFolderNameFromPaths(entry.dest_path, entry.dest_root),
    };
  }

  // Ambil nama folder kegiatan pertama dari dest_path relatif terhadap dest_root,
  // mis. "D:\Arsip\03-15_Kegiatan\DSC_0001.jpg" + root "D:\Arsip" → "03-15_Kegiatan"
  function destFolderNameFromPaths(destPath: string, destRoot: string): string {
    if (!destPath) return '—';
    const normRoot = (destRoot ?? '').replace(/[\\/]+$/, '');
    let rel = destPath;
    if (normRoot && destPath.toLowerCase().startsWith(normRoot.toLowerCase())) {
      rel = destPath.slice(normRoot.length);
    }
    const segs = rel.split(/[\\/]/).filter(Boolean);
    return segs.length > 0 ? segs[0] : (destPath.split(/[\\/]/).filter(Boolean).pop() ?? destPath);
  }

  const avgPerFile = $derived(
    successCount > 0 ? formatBytes(Math.round(bytesTransferred / successCount)) : '—'
  );

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

<div class="done-view" aria-label="Hasil transfer">
  {#if !isPartial}
    <!-- Banner Sukses -->
    <section class="banner banner-success">
      <div class="accent"></div>
      <div class="banner-icon" aria-hidden="true">✅</div>
      <div class="banner-text">
        <h2 class="banner-title">Pemindahan Selesai</h2>
        <p class="banner-sub">
          {successCount} file berhasil {mode === 'move' ? 'dipindahkan' : 'disalin'} ke {activeDestinations.length || 1} folder • {formatBytes(bytesTransferred)} • selesai dalam {durationLabel()}
        </p>
      </div>
      <div class="banner-actions">
        <button class="banner-btn outline-green" type="button" disabled={busy !== null} onclick={exportCsv}>
          {busy === 'export' ? 'Mengekspor…' : '⬇ Ekspor CSV'}
        </button>
        <button class="banner-btn solid-green" type="button" disabled={busy !== null} onclick={openTarget}>
          📂 Buka
        </button>
      </div>
    </section>
  {:else}
    <!-- Banner Peringatan -->
    <section class="banner banner-warn" role="alert">
      <div class="accent"></div>
      <div class="banner-icon" aria-hidden="true">⚠️</div>
      <div class="banner-text">
        <h2 class="banner-title">Pemindahan Selesai dengan {failedRows.length} Masalah</h2>
        <p class="banner-sub">
          {successCount} file berhasil di semua tujuan • {errorCount} file gagal di salah satu tujuan • file sumber TIDAK dihapus
        </p>
      </div>
      <div class="banner-actions">
        <button class="banner-btn outline-red" type="button" disabled={busy !== null} onclick={exportCsv}>
          {busy === 'export' ? 'Mengekspor…' : '⬇ Ekspor CSV'}
        </button>
        <button class="banner-btn solid-red" type="button" onclick={newSession}>
          ↻ Ulangi
        </button>
      </div>
    </section>
  {/if}

  <!-- Stat Cards -->
  <div class="stat-grid">
    <div class="stat-card">
      <span class="stat-lbl">{mode === 'move' ? 'File Dipindahkan' : 'File Disalin'}</span>
      <span class="stat-val" class:green={!isPartial} class:red={isPartial}>{successCount}</span>
      <span class="stat-sub">dari {totalSelected} terpilih</span>
    </div>
    <div class="stat-card">
      <span class="stat-lbl">Total Ukuran</span>
      <span class="stat-val">{formatBytes(bytesTransferred)}</span>
      <span class="stat-sub">rata-rata {avgPerFile}/file</span>
    </div>
    <div class="stat-card">
      <span class="stat-lbl">Dilewati / Gagal</span>
      <span class="stat-val" class:green={errorCount + skippedCount === 0} class:amber={errorCount + skippedCount > 0}>
        {errorCount + skippedCount}
      </span>
      <span class="stat-sub">{errorCount + skippedCount === 0 ? 'tidak ada masalah' : `${errorCount} gagal • ${skippedCount} dilewati`}</span>
    </div>
  </div>

  {#if !isPartial}
    <!-- Panel Log -->
    <section class="panel-log">
      <div class="log-top">
        <h2 class="panel-title">Log Pemindahan</h2>
        <span class="log-sub">— {logEntries.length || successCount} baris, semua berhasil</span>
      </div>

      <div class="table-container">
        <div class="table-header">
          <span class="th-nama">Nama File</span>
          <span class="th-tipe">Tipe</span>
          <span class="th-ukuran">Ukuran</span>
          <span class="th-tanggal">Tanggal</span>
          <span class="th-tujuan">Folder Tujuan</span>
          <span class="th-status">Hasil</span>
        </div>
        <div class="table-body">
          {#if logEntries.length === 0}
            {#each store.previewRows as r (r.source)}
              <div class="table-row">
                <span class="cell-nama truncate" title={r.name}>{r.name}</span>
                <span class="cell-tipe">{r.kind === 'video' ? '🎬' : '📷'}</span>
                <span class="cell-ukuran">{formatBytes(r.size_bytes)}</span>
                <span class="cell-tanggal">{formatDisplayDate(r.date_override ?? r.date)}</span>
                <span class="cell-tujuan truncate" title={destFolderFor(r)}>{destFolderFor(r)}</span>
                <span class="cell-status st-done">✓ {mode === 'move' ? 'Dipindah' : 'Tersalin'}</span>
              </div>
            {/each}
          {:else}
            {#each logEntries as entry, i (i)}
              {@const rowInfo = infoForEntry(entry)}
              <div class="table-row">
                <span class="cell-nama truncate" title={entry.source_path}>{rowInfo.name}</span>
                <span class="cell-tipe">{rowInfo.isVideo ? '🎬' : '📷'}</span>
                <span class="cell-ukuran">{formatBytes(entry.size_bytes)}</span>
                <span class="cell-tanggal">{rowInfo.date}</span>
                <span class="cell-tujuan truncate" title={rowInfo.folder}>{rowInfo.folder}</span>
                <span class="cell-status st-done">✓ {entry.status === 'copied' ? 'Tersalin' : entry.status === 'moved' ? 'Dipindah' : entry.status}</span>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </section>
  {:else}
    <!-- Panel Masalah -->
    <section class="panel-masalah">
      <div class="log-top">
        <h2 class="panel-title">File Gagal ({failedRows.length})</h2>
        <span class="log-sub">— periksa penyebab lalu klik Ulangi untuk file ini saja</span>
      </div>

      <div class="table-container">
        <div class="table-header error-header">
          <span class="th-nama">Nama File</span>
          <span class="th-ukuran">Ukuran</span>
          <span class="th-tanggal">Tanggal</span>
          <span class="th-tujuan">Folder Tujuan</span>
          <span class="th-sebab">Penyebab</span>
          <span class="th-aksi">Aksi</span>
        </div>
        <div class="table-body">
          {#if failedRows.length === 0}
            <div class="empty-cell">Tidak ada detail per file. Lihat pesan kesalahan di atas.</div>
          {:else}
            {#each failedRows as row (row.source)}
              <div class="fail-row">
                <span class="fail-icon">✖</span>
                <div class="fail-nama">
                  <span class="cell-nama truncate" title={row.name}>{row.name}</span>
                  <span class="fail-meta">dari {destFolderFor(row)}</span>
                </div>
                <span class="cell-ukuran">{formatBytes(row.size_bytes)}</span>
                <span class="cell-tanggal">{formatDisplayDate(row.date_override ?? row.date)}</span>
                <span class="cell-tujuan truncate" title={destFolderFor(row)}>{destFolderFor(row)}</span>
                <div class="fail-sebab">
                  <span class="fail-reason">{row.error ?? 'Transfer tidak selesai di semua tujuan'}</span>
                  <span class="fail-hint">Status: {row.status === 'partial' ? 'Parsial' : 'Gagal'} • Sumber tidak dihapus</span>
                </div>
                <button class="btn-retry-row" type="button" onclick={newSession}>↻ Ulangi</button>
              </div>
            {/each}
          {/if}
        </div>
      </div>

      {#if mode === 'move'}
        <div class="info-box">
          <span class="info-line">⚠ Mode Pindah (Move) multi-tujuan: file sumber HANYA dihapus setelah berhasil di SEMUA tujuan.</span>
          <span class="info-line">Saran: buat ulang koneksi NAS/izin folder lalu ulangi. Tujuan yang sudah berhasil tidak akan diduplikasi.</span>
        </div>
      {/if}

      <div class="dest-divider"></div>
      <div class="dest-status-section">
        <span class="dest-status-title">Status per tujuan</span>
        <div class="dest-status-list">
          {#each store.destProgresses as d (d.dest_id)}
            {@const pct = d.total > 0 ? Math.round((d.done / d.total) * 100) : 0}
            {@const failed = d.total - d.done}
            <div class="dest-status-row">
              <div class="dest-info">
                <span class="dest-root">{d.dest_root}</span>
                <span class="dest-folder">{d.dest_folder || 'Semua folder kegiatan'}</span>
              </div>
              <span class="dest-count">{d.done}/{d.total} file</span>
              {#if failed === 0}
                <span class="dest-ok">✓ Semua berhasil</span>
              {:else}
                <span class="dest-fail">⚠ {failed} gagal</span>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    </section>
  {/if}
</div>

<style>
  .done-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    box-sizing: border-box;
    overflow-y: auto;
  }

  /* Banner */
  .banner {
    position: relative;
    display: flex;
    align-items: center;
    gap: 20px;
    height: 80px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0 16px 0 24px;
    box-sizing: border-box;
    flex: 0 0 auto;
  }

  .accent {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 6px;
    border-radius: 8px 0 0 8px;
  }

  .banner-success .accent {
    background: #16A34A;
  }

  .banner-warn .accent {
    background: #DC2626;
  }

  .banner-icon {
    font-size: 26px;
    line-height: 1;
    flex: 0 0 auto;
  }

  .banner-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .banner-title {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    line-height: 1.2;
  }

  .banner-success .banner-title {
    color: #166534;
  }

  .banner-warn .banner-title {
    color: #991B1B;
  }

  .banner-sub {
    margin: 0;
    font-size: 12px;
    line-height: 1.3;
  }

  .banner-success .banner-sub {
    color: #15803D;
  }

  .banner-warn .banner-sub {
    color: #B91C1C;
  }

  .banner-actions {
    display: flex;
    gap: 10px;
    flex: 0 0 auto;
  }

  .banner-btn {
    height: 32px;
    padding: 0 14px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    font-family: inherit;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }

  .banner-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .outline-green {
    background: var(--card);
    border: 1px solid #86EFAC;
    color: #166534;
  }

  .outline-green:hover:not(:disabled) {
    background: #F0FDF4;
  }

  .solid-green {
    background: #16A34A;
    border: none;
    color: #ffffff;
    font-weight: 600;
  }

  .solid-green:hover:not(:disabled) {
    background: #15803D;
  }

  .outline-red {
    background: var(--card);
    border: 1px solid #FCA5A5;
    color: #991B1B;
  }

  .outline-red:hover:not(:disabled) {
    background: #FFF1F2;
  }

  .solid-red {
    background: #DC2626;
    border: none;
    color: #ffffff;
    font-weight: 600;
  }

  .solid-red:hover:not(:disabled) {
    background: #B91C1C;
  }

  /* Stat cards */
  .stat-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
    flex: 0 0 auto;
  }

  .stat-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    height: 76px;
    padding: 12px 16px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .stat-lbl {
    font-size: 11px;
    color: var(--text-3);
  }

  .stat-val {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
  }

  .stat-val.green {
    color: #16A34A;
  }

  .stat-val.red {
    color: #DC2626;
  }

  .stat-val.amber {
    color: #B45309;
  }

  .stat-sub {
    font-size: 10px;
    color: var(--text-3);
  }

  /* Panels */
  .panel-log,
  .panel-masalah {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    flex: 1;
    min-height: 0;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }

  .log-top {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex: 0 0 auto;
  }

  .panel-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .log-sub {
    font-size: 11px;
    color: var(--text-3);
  }

  .table-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    container-type: inline-size;
  }

  .table-header {
    display: grid;
    grid-template-columns: minmax(0, 2.5fr) 42px 70px 88px minmax(0, 2.5fr) 104px;
    align-items: center;
    height: 30px;
    background: var(--bg);
    border-radius: 4px;
    padding: 0 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-2);
    flex: 0 0 auto;
  }

  .table-header.error-header {
    grid-template-columns: 24px minmax(0, 2.2fr) 64px 80px minmax(0, 2fr) minmax(0, 2.2fr) 84px;
    background: #FEF2F2;
    color: #991B1B;
  }

  .table-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: minmax(0, 2.5fr) 42px 70px 88px minmax(0, 2.5fr) 104px;
    align-items: center;
    height: 30px;
    padding: 0 12px;
    border-bottom: 1px solid var(--bg);
    font-size: 12px;
    flex: 0 0 auto;
  }

  /* Panel sempit: sembunyikan kolom minor */
  @container (max-width: 660px) {
    .table-header,
    .table-row {
      grid-template-columns: minmax(0, 1fr) 64px 96px minmax(0, 1fr);
    }

    .th-tipe,
    .cell-tipe,
    .th-ukuran,
    .cell-ukuran {
      display: none;
    }
  }

  @container (max-width: 420px) {
    .table-header,
    .table-row {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
      padding: 0 8px;
    }

    .th-tanggal,
    .cell-tanggal {
      display: none;
    }
  }

  .table-row:hover {
    background: var(--bg);
  }

  .cell-nama {
    color: var(--text);
  }

  .cell-tipe {
    font-size: 13px;
  }

  .cell-ukuran {
    color: var(--text-2);
    font-size: 11px;
  }

  .cell-tanggal {
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cell-tujuan {
    color: #0066CC;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }

  .cell-status {
    font-size: 11px;
    font-weight: 500;
  }

  .cell-status.st-done {
    color: #16A34A;
  }

  /* Fail rows */
  .fail-row {
    display: grid;
    grid-template-columns: 24px minmax(0, 2.2fr) 60px 90px minmax(0, 2fr) minmax(0, 2.4fr) 90px;
    align-items: center;
    min-height: 38px;
    padding: 4px 12px;
    border-bottom: 1px solid #FEE2E2;
    font-size: 12px;
    flex: 0 0 auto;
  }

  /* Error panel sempit queries */
  @container (max-width: 660px) {
    .fail-row {
      grid-template-columns: 24px minmax(0, 1fr) 56px 72px minmax(0, 1fr);
    }

    .fail-row .cell-ukuran,
    .fail-row .cell-tanggal {
      display: none;
    }
  }

  @container (max-width: 420px) {
    .fail-row {
      grid-template-columns: 20px minmax(0, 1fr) minmax(0, 1fr);
      padding: 4px 8px;
    }

    .fail-row .cell-tanggal {
      display: none;
    }

    .btn-retry-row {
      width: 32px;
      padding: 4px;
    }
  }

  .fail-icon {
    color: #DC2626;
    font-weight: 700;
    font-size: 12px;
  }

  .fail-nama {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .fail-meta {
    font-size: 10px;
    color: var(--text-3);
  }

  .fail-sebab {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .fail-reason {
    font-size: 11px;
    color: #B91C1C;
  }

  .fail-hint {
    font-size: 9px;
    color: var(--text-3);
  }

  .btn-retry-row {
    height: 26px;
    padding: 0 10px;
    background: #FFF1F2;
    border: 1px solid #FECDD3;
    border-radius: 5px;
    font-size: 11px;
    font-weight: 500;
    color: #DC2626;
    cursor: pointer;
    font-family: inherit;
    justify-self: start;
  }

  .btn-retry-row:hover {
    background: #FEE2E2;
  }

  .empty-cell {
    text-align: center;
    color: var(--text-3);
    padding: 24px;
    font-size: 12px;
  }

  /* Info box */
  .info-box {
    background: #FFF7E6;
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 0 0 auto;
  }

  .info-line {
    font-size: 11px;
    color: #B45309;
  }

  /* Dest status */
  .dest-divider {
    height: 1px;
    background: var(--border);
    flex: 0 0 auto;
  }

  .dest-status-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 0 0 auto;
  }

  .dest-status-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text);
  }

  .dest-status-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .dest-status-row {
    display: flex;
    align-items: center;
    gap: 16px;
    min-width: 0;
  }

  .dest-info {
    flex: 1 1 auto;
    min-width: 0;
    max-width: 310px;
    display: flex;
    flex-direction: column;
  }

  .dest-root {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dest-folder {
    font-size: 9px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dest-count {
    font-size: 10px;
    color: var(--text-2);
  }

  .dest-ok {
    font-size: 10px;
    font-weight: 500;
    color: #16A34A;
  }

  .dest-fail {
    font-size: 10px;
    font-weight: 500;
    color: #B45309;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
</style>