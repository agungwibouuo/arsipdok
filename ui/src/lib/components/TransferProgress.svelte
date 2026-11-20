<script lang="ts">
  import { store, formatBytes } from '../store.svelte';
  import type { DestProgress, PreviewRow } from '../types';

  interface Props {
    onCancel?: () => void;
  }

  let { onCancel }: Props = $props();

  const dests = $derived(store.destProgresses);
  const percent = $derived(store.transferProgress);
  const mode = $derived(store.settings.transfer_mode);

  const totalDone = $derived(dests.reduce((s, d) => s + d.done, 0));
  const totalFiles = $derived(dests.reduce((s, d) => s + d.total, 0));
  const totalBytesDone = $derived(dests.reduce((s, d) => s + d.bytes_done, 0));
  const totalBytesTotal = $derived(dests.reduce((s, d) => s + d.bytes_total, 0));

  const currentFile = $derived(
    store.previewRows.find((r) => r.status === 'copying' || r.status === 'verifying') ?? null
  );

  const activeDestinations = $derived(store.settings.destinations.filter((d) => d.enabled));

  function barPercent(d: DestProgress): number {
    if (d.total <= 0) return d.status === 'done' ? 100 : 0;
    return Math.min(100, Math.round((d.done / d.total) * 100));
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

  function folderName(r: PreviewRow): string {
    const d = r.date_override ?? r.date;
    const dest = activeDestinations[0];
    const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest?.suffix ?? '').trim();
    const parts = d.split('-');
    const mmDd = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d;
    return `${mmDd}_${activity}${suffix ? `_${suffix}` : ''}`;
  }

  function currentDestPath(r: PreviewRow): string {
    const item = store.plan?.items.find((i) => i.source === r.source);
    if (item?.dest_path) return item.dest_path;
    const root = activeDestinations[0]?.root ?? 'D:\\Dokumentasi';
    return `${root}\\${folderName(r)}\\${r.name}`;
  }

  const speedAndEta = $derived.by(() => {
    if (totalFiles === 0) return 'Menyiapkan…';
    const remainingFiles = Math.max(0, totalFiles - totalDone);
    if (remainingFiles === 0) return 'Menyelesaikan verifikasi…';
    return `${remainingFiles} file tersisa`;
  });
</script>

<div class="progress-view" aria-label="Progres transfer">
  <!-- 1. Panel Progres (1160x224) -->
  <section class="panel-progres">
    <div class="progres-top">
      <div class="progres-info">
        <h2 class="prog-title">
          Sedang Memindahkan… &nbsp;<span class="mode-tag">({mode === 'move' ? 'Mode Pindah' : 'Mode Salin'})</span>
        </h2>
        <div class="prog-count">
          {totalDone} dari {totalFiles} file • {formatBytes(totalBytesDone)} dari {formatBytes(totalBytesTotal)}
        </div>
      </div>

      <button class="btn-cancel" type="button" onclick={() => onCancel?.()}>
        <span>■</span> Batalkan
      </button>
    </div>

    <!-- Progress bar & percentage -->
    <div class="bar-row">
      <div class="prog-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent}>
        <div class="prog-fill" style={`width: ${percent}%`}></div>
      </div>
      <div class="prog-pct">{percent}%</div>
    </div>

    <!-- Status baris -->
    <div class="prog-lines">
      <div class="prog-current truncate" title={currentFile ? currentDestPath(currentFile) : ''}>
        {#if currentFile}
          <span class="icon-spin">⟳</span>
          <span class="lbl-now">Sekarang:</span>
          <span class="val-name">{currentFile.name}</span>
          <span class="arrow">→</span>
          <span class="val-path">{currentDestPath(currentFile)}</span>
        {:else}
          <span class="lbl-now">Menunggu pemrosesan file…</span>
        {/if}
      </div>
      <div class="prog-speed">{speedAndEta}</div>
    </div>

    <!-- Progres per tujuan -->
    <div class="per-dest-section">
      <div class="per-dest-header">
        <span class="per-dest-label">Progres per tujuan</span>
        <div class="per-dest-divider"></div>
      </div>

      <div class="per-dest-list">
        {#if dests.length === 0}
          <div class="dest-item-placeholder">Menyiapkan alur transfer...</div>
        {:else}
          {#each dests as d (d.dest_id)}
            {@const pct = barPercent(d)}
            {@const isDone = pct >= 100 || d.status === 'done'}
            <div class="dest-item">
              <div class="dest-info">
                <span class="dest-root truncate" title={d.dest_root}>{d.dest_root}</span>
                <span class="dest-sub truncate">{d.dest_folder || 'Semua folder kegiatan'}</span>
              </div>
              <div class="dest-track">
                <div class="dest-fill" class:done={isDone} style={`width: ${pct}%`}></div>
              </div>
              <span class="dest-cnt">{d.done}/{d.total}</span>
              <span class="dest-st" class:done={isDone}>
                {#if isDone}
                  ✓ Selesai
                {:else if d.status === 'error'}
                  ✕ Gagal
                {:else}
                  ⟳ {pct}%
                {/if}
              </span>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </section>

  <!-- 2. Panel Tabel Progres (1160x396) -->
  <section class="panel-tabel-progres">
    <div class="tabel-top">
      <h2 class="tabel-title">Detail File</h2>
      <div class="tabel-legend">
        <span class="leg-done">✓ Selesai</span>
        <span class="leg-run">⟳ Berjalan</span>
        <span class="leg-wait">○ Menunggu</span>
      </div>
    </div>

    <div class="table-container">
      <div class="table-header">
        <span class="th-nama">Nama File</span>
        <span class="th-tipe">Tipe</span>
        <span class="th-ukuran">Ukuran</span>
        <span class="th-tanggal">Tanggal</span>
        <span class="th-tujuan">Folder Tujuan</span>
        <span class="th-status">Status</span>
      </div>

      <div class="table-body">
        {#each store.previewRows as r (r.source)}
          <div class="table-row">
            <span class="cell-nama truncate" title={r.name}>{r.name}</span>
            <span class="cell-tipe">{r.kind === 'video' ? '🎬' : '📷'}</span>
            <span class="cell-ukuran">{formatBytes(r.size_bytes)}</span>
            <span class="cell-tanggal">{formatDisplayDate(r.date_override ?? r.date)}</span>
            <span class="cell-tujuan truncate" title={folderName(r)}>{folderName(r)}</span>
            <span class="cell-status">
              {#if r.status === 'done'}
                <span class="st-done">✓ Selesai</span>
              {:else if r.status === 'copying' || r.status === 'verifying'}
                <span class="st-run">⟳ Sedang diproses</span>
              {:else if r.status === 'error'}
                <span class="st-err">✕ Gagal</span>
              {:else if r.status === 'skipped'}
                <span class="st-skip">− Dilewati</span>
              {:else}
                <span class="st-wait">○ Menunggu</span>
              {/if}
            </span>
          </div>
        {/each}
      </div>
    </div>
  </section>
</div>

<style>
  .progress-view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    box-sizing: border-box;
    container-type: inline-size;
  }

  /* 1. Panel Progres (1160x224) */
  .panel-progres {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    height: 224px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 0 0 auto;
    overflow: hidden;
  }

  .progres-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .progres-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .prog-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    line-height: 1.2;
  }

  .mode-tag {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-3);
  }

  .prog-count {
    font-size: 12px;
    color: var(--text-2);
  }

  .btn-cancel {
    height: 34px;
    padding: 0 16px;
    background: var(--card);
    border: 1px solid #FCA5A5;
    border-radius: 6px;
    color: #DC2626;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: inherit;
  }

  .btn-cancel:hover {
    background: #FFF1F2;
    border-color: #F87171;
  }

  .bar-row {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 2px;
  }

  .prog-track {
    flex: 1;
    height: 10px;
    background: var(--border);
    border-radius: 5px;
    overflow: hidden;
  }

  .prog-fill {
    height: 100%;
    background: #0F62FE;
    border-radius: 5px;
    transition: width 0.25s ease;
  }

  .prog-pct {
    font-size: 22px;
    font-weight: 700;
    color: #0F62FE;
    min-width: 60px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    line-height: 1;
  }

  .prog-lines {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 11px;
  }

  .prog-current {
    color: var(--text-2);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .icon-spin {
    color: #0F62FE;
  }

  .lbl-now {
    font-weight: 500;
  }

  .val-name {
    font-weight: 600;
  }

  .arrow {
    color: var(--text-3);
  }

  .val-path {
    color: var(--text-3);
  }

  .prog-speed {
    color: var(--text-3);
  }

  .per-dest-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .per-dest-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .per-dest-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    flex: 0 0 auto;
  }

  .per-dest-divider {
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  .per-dest-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 64px;
    overflow-y: auto;
  }

  .dest-item {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 9px;
  }

  .dest-info {
    flex: 1 1 auto;
    min-width: 0;
    max-width: 250px;
    display: flex;
    flex-direction: column;
  }

  .dest-root {
    color: var(--text-2);
    font-weight: 500;
  }

  .dest-sub {
    color: var(--text-3);
  }

  .dest-track {
    flex: 1;
    height: 10px;
    background: var(--bg);
    border-radius: 5px;
    overflow: hidden;
  }

  .dest-fill {
    height: 100%;
    background: #0F62FE;
    border-radius: 5px;
    transition: width 0.25s ease;
  }

  .dest-fill.done {
    background: #16A34A;
  }

  .dest-cnt {
    width: 60px;
    text-align: right;
    color: var(--text-2);
    font-weight: 500;
  }

  .dest-st {
    width: 70px;
    text-align: right;
    color: #0F62FE;
    font-weight: 500;
  }

  .dest-st.done {
    color: #16A34A;
  }

  .dest-item-placeholder {
    font-size: 11px;
    color: var(--text-3);
  }

  /* 2. Panel Tabel Progres (1160x396) */
  .panel-tabel-progres {
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
    container-type: inline-size;
  }

  .tabel-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tabel-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .tabel-legend {
    font-size: 11px;
    display: flex;
    gap: 16px;
    color: var(--text-3);
  }

  .leg-done {
    color: #16A34A;
  }

  .leg-run {
    color: #2563EB;
  }

  .leg-wait {
    color: var(--text-3);
  }

  .table-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    border-radius: 4px;
    overflow: hidden;
  }

  .table-header {
    display: grid;
    grid-template-columns: minmax(140px, 3fr) 48px 80px 108px minmax(140px, 3.5fr) 124px;
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

  .table-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: minmax(140px, 3fr) 48px 80px 108px minmax(140px, 3.5fr) 124px;
    align-items: center;
    height: 30px;
    padding: 0 12px;
    border-bottom: 1px solid var(--bg);
    font-size: 12px;
    flex: 0 0 auto;
  }

  .table-row:hover {
    background: var(--bg);
  }

  /* Panel sempit: sembunyikan kolom minor agar tidak overflow */
  @container (max-width: 720px) {
    .table-header,
    .table-row {
      grid-template-columns: minmax(0, 1fr) 56px 96px minmax(0, 1fr) 108px;
    }

    .th-tipe,
    .cell-tipe,
    .th-tanggal,
    .cell-tanggal {
      display: none;
    }
  }

  @container (max-width: 480px) {
    .table-header,
    .table-row {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 100px;
      padding: 0 8px;
    }

    .th-ukuran,
    .cell-ukuran {
      display: none;
    }
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
  }

  .cell-tujuan {
    color: #0066CC;
  }

  .cell-status {
    font-size: 11px;
    font-weight: 500;
  }

  .st-done {
    color: #16A34A;
  }

  .st-run {
    color: #2563EB;
  }

  .st-err {
    color: #DC2626;
  }

  .st-skip {
    color: var(--text-3);
  }

  .st-wait {
    color: var(--text-3);
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
</style>