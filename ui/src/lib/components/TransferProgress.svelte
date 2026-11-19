<script lang="ts">
  import { store, formatBytes } from '../store.svelte';
  import type { DestProgress } from '../types';

  interface Props {
    onCancel?: () => void;
  }

  let { onCancel }: Props = $props();

  const dests = $derived(store.destProgresses);
  const percent = $derived(store.transferProgress);
  const totalDone = $derived(dests.reduce((s, d) => s + d.done, 0));
  const totalFiles = $derived(dests.reduce((s, d) => s + d.total, 0));
  const totalBytesDone = $derived(dests.reduce((s, d) => s + d.bytes_done, 0));
  const totalBytesTotal = $derived(dests.reduce((s, d) => s + d.bytes_total, 0));

  const itemsDone = $derived(store.transferReport?.items_done ?? 0);
  const itemsTotal = $derived(store.plan?.total_files ?? totalFiles);

  const currentFile = $derived(
    store.previewRows.find((r) => r.status === 'copying' || r.status === 'verifying') ?? null
  );

  function barPercent(d: DestProgress): number {
    if (d.total <= 0) return d.status === 'done' ? 100 : 0;
    return Math.round((d.done / d.total) * 100);
  }

  function destName(d: DestProgress): string {
    return `${d.dest_root}${d.dest_folder ? `\\${d.dest_folder}` : ''}`;
  }

  function statusLabel(d: DestProgress): string {
    switch (d.status) {
      case 'done':
        return 'Selesai';
      case 'error':
        return 'Gagal';
      case 'running':
        return 'Berjalan';
      default:
        return 'Menunggu';
    }
  }

  // Kecepatan rata-rata (tanpa timer real-time; estimasi kasar berbasis byte)
  const speedLabel = $derived.by(() => {
    if (totalFiles === 0) return '—';
    const remaining = totalFiles - totalDone;
    if (remaining <= 0) return 'Menyelesaikan...';
    return `${remaining} file tersisa`;
  });
</script>

<section class="transfer-progress panel" aria-label="Progres transfer">
  <div class="banner" role="status" aria-live="polite">
    <span class="spinner" aria-hidden="true"></span>
    <div class="banner-text">
      <strong>Memindahkan file...</strong>
      <span>Jangan cabut kartu memori atau disk tujuan</span>
    </div>
  </div>

  <div class="aggregate">
    <div class="aggregate-head">
      <span class="agg-label">Progres Keseluruhan</span>
      <span class="agg-percent">{percent}%</span>
    </div>
    <div
      class="progress-track"
      role="progressbar"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={percent}
    >
      <div class="progress-fill" style={`width: ${percent}%`}></div>
    </div>
    <div class="aggregate-meta">
      <span>{totalDone} dari {totalFiles} file</span>
      <span class="dot">·</span>
      <span>{formatBytes(totalBytesDone)} / {formatBytes(totalBytesTotal)}</span>
      <span class="dot">·</span>
      <span>{speedLabel}</span>
      {#if itemsTotal > 0}
        <span class="dot">·</span>
        <span>{itemsDone}/{itemsTotal} item diproses</span>
      {/if}
    </div>
  </div>

  {#if currentFile}
    <div class="current-file">
      <span class="current-label">Sekarang:</span>
      <span class="current-path truncate" title={currentFile.source}>
        {currentFile.name}
        <span class="arrow">→</span>
        {store.plan?.items.find((i) => i.source === currentFile.source)?.dest_path ?? '...'}
      </span>
    </div>
  {/if}

  <div class="dest-progress-list">
    {#if dests.length === 0}
      <p class="empty">Menyiapkan tujuan transfer...</p>
    {:else}
      {#each dests as d (d.dest_id)}
        {@const pct = barPercent(d)}
        <div class="dest-progress" class:error={d.status === 'error'}>
          <div class="dest-head">
            <span class="dest-name truncate" title={destName(d)}>{destName(d)}</span>
            <span class="dest-counter">{d.done}/{d.total}</span>
            <span class="dest-status status-{d.status}">{statusLabel(d)}</span>
          </div>
          <div class="mini-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={pct}>
            <div class="mini-fill" class:error={d.status === 'error'} style={`width: ${pct}%`}></div>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <div class="footer-actions">
    <button class="btn btn-danger" type="button" onclick={() => onCancel?.()}>
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.9" />
        <path d="M9 9l6 6M15 9l-6 6" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
      <span>Batalkan Transfer</span>
    </button>
  </div>
</section>

<style>
  .transfer-progress {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px;
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid #c7d7fe;
    border-radius: var(--radius);
    background: var(--primary-soft);
    color: #1e40af;
  }

  .banner-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-size: 13px;
  }

  .banner-text strong {
    font-weight: 600;
    color: var(--primary);
  }

  .spinner {
    width: 20px;
    height: 20px;
    flex: 0 0 auto;
    border-radius: 50%;
    border: 2.5px solid rgba(15, 98, 254, 0.25);
    border-top-color: var(--primary);
    animation: spin 0.8s linear infinite;
  }

  .aggregate {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .aggregate-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .agg-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-2);
  }

  .agg-percent {
    font-size: 20px;
    font-weight: 700;
    color: var(--primary);
    font-variant-numeric: tabular-nums;
  }

  .progress-track {
    height: 10px;
    border-radius: 999px;
    background: #e2e8f0;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--primary), #4f8cff);
    transition: width 0.25s ease;
  }

  .aggregate-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 5px;
    font-size: 12px;
    color: var(--text-3);
  }

  .dot {
    opacity: 0.6;
  }

  .current-file {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--bg);
    border: 1px solid var(--border);
    font-size: 12px;
    min-width: 0;
  }

  .current-label {
    flex: 0 0 auto;
    font-weight: 600;
    color: var(--text-3);
  }

  .current-path {
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11.5px;
    min-width: 0;
  }

  .arrow {
    color: var(--primary);
    margin: 0 3px;
  }

  .dest-progress-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty {
    font-size: 13px;
    color: var(--text-3);
  }

  .dest-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }

  .dest-progress.error {
    border-color: #fecaca;
    background: var(--danger-bg);
  }

  .dest-head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    font-size: 12.5px;
  }

  .dest-name {
    flex: 1;
    min-width: 0;
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11.5px;
  }

  .dest-counter {
    flex: 0 0 auto;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }

  .dest-status {
    flex: 0 0 auto;
    font-size: 11px;
    font-weight: 600;
    padding: 1px 7px;
    border-radius: 999px;
    background: #f1f5f9;
    color: var(--text-3);
  }

  .dest-status.status-running {
    background: var(--primary-soft);
    color: var(--primary);
  }

  .dest-status.status-done {
    background: var(--success-bg);
    color: var(--success);
  }

  .dest-status.status-error {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .mini-track {
    height: 6px;
    border-radius: 999px;
    background: #e2e8f0;
    overflow: hidden;
  }

  .mini-fill {
    height: 100%;
    border-radius: 999px;
    background: var(--primary);
    transition: width 0.25s ease;
  }

  .mini-fill.error {
    background: var(--danger);
  }

  .footer-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 4px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation: none;
    }
    .progress-fill,
    .mini-fill {
      transition: none;
    }
  }
</style>
