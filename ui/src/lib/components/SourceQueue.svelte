<script lang="ts">
  import { store } from '../store.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    onAddPath?: (path: string) => void;
    onRescan?: () => void;
    rescanDisabled?: boolean;
  }

  let { onAddPath, onRescan, rescanDisabled = false }: Props = $props();

  const hasScanned = $derived(store.scanReport !== null);
  const count = $derived(store.sourcePaths.length);

  function basename(p: string): string {
    const parts = p.replace(/[\\/]+$/, '').split(/[\\/]/);
    return parts[parts.length - 1] || p;
  }

  function isSdCard(p: string): boolean {
    return /^[a-z]:[\\/]?$/i.test(p.trim()) || /removable|sd|xdcard|\bmnt\b/i.test(p);
  }

  async function addSource() {
    try {
      const selected = await open({
        directory: true,
        multiple: true,
        title: 'Pilih folder sumber dokumentasi',
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      for (const p of paths) {
        if (!p) continue;
        if (onAddPath) onAddPath(p);
        else store.addSourcePath(p);
      }
    } catch (err) {
      store.globalError = `Gagal membuka dialog folder: ${String(err)}`;
    }
  }
</script>

<section class="source-queue panel" aria-label="Antrean sumber dokumentasi">
  <header class="queue-header">
    <div class="title-wrap">
      <h2>Sumber Dokumentasi</h2>
      <span class="badge badge-blue" aria-label={`${count} sumber`}>{count} sumber</span>
    </div>
    {#if hasScanned}
      <button
        class="btn btn-secondary btn-sm"
        type="button"
        onclick={() => onRescan?.()}
        disabled={rescanDisabled || count === 0}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path
            d="M20 11a8 8 0 1 0-2.34 5.66M20 4v7h-7"
            stroke="currentColor"
            stroke-width="1.9"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span>Pindai Ulang</span>
      </button>
    {/if}
  </header>

  {#if count === 0}
    <p class="empty-hint">Belum ada folder sumber. Tambahkan folder SD card atau folder lokal untuk mulai memindai.</p>
  {:else}
    <ul class="source-list">
      {#each store.sourcePaths as path (path)}
        {@const sd = isSdCard(path)}
        <li class="source-item">
          <span class="folder-icon" class:sd aria-hidden="true">
            {#if sd}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <rect x="5" y="3" width="14" height="18" rx="2" stroke="currentColor" stroke-width="1.8" />
                <path d="M8 7h8M8 11h8M8 15h8M8 18h4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
              </svg>
            {:else}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <path
                  d="M3 7a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.58L11.4 7H19a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linejoin="round"
                />
              </svg>
            {/if}
          </span>

          <div class="meta">
            <span class="path truncate mono-sm" title={path}>{path}</span>
            {#if hasScanned}
              <span class="summary">
                <span class="file-count">{store.scannedFiles.length} file</span>
                <span class="dot">·</span>
                <span>{formatMb(store.totalSourceSize)} total</span>
              </span>
            {:else}
              <span class="summary waiting">Menunggu pemindaian</span>
            {/if}
          </div>

          <button
            class="remove-btn"
            type="button"
            title={`Hapus ${basename(path)}`}
            aria-label={`Hapus sumber ${basename(path)}`}
            onclick={() => store.removeSourcePath(path)}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M6 6l12 12M18 6L6 18" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <button class="btn btn-secondary add-btn" type="button" onclick={addSource}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </svg>
    <span>Tambah Sumber Lain</span>
  </button>
</section>

<script lang="ts" module>
  function formatMb(bytes: number): string {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }
</script>

<style>
  .source-queue {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .queue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .title-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .title-wrap h2 {
    font-size: 15px;
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-3);
    line-height: 1.5;
  }

  .source-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .source-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg);
    transition: border-color 0.12s ease;
  }

  .source-item:hover {
    border-color: var(--border-strong);
  }

  .folder-icon {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    background: var(--primary-soft);
    color: var(--primary);
  }

  .folder-icon.sd {
    background: #fef3c7;
    color: #b45309;
  }

  .meta {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .path {
    color: var(--text-2);
  }

  .summary {
    font-size: 12px;
    color: var(--text-3);
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .summary.waiting {
    color: var(--warning);
  }

  .file-count {
    font-weight: 500;
    color: var(--text-2);
  }

  .dot {
    opacity: 0.6;
  }

  .remove-btn {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease;
  }

  .remove-btn:hover {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .add-btn {
    align-self: flex-start;
  }
</style>
