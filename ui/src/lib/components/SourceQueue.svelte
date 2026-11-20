<script lang="ts">
  import { store } from '../store.svelte';
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    onAddPath?: (path: string) => void;
    onRescan?: () => void;
    rescanDisabled?: boolean;
    tall?: boolean;
  }

  let { onAddPath, onRescan, rescanDisabled = false, tall = false }: Props = $props();

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

  function fileCount(): number {
    return store.scannedFiles.length;
  }
</script>

<section class="source-panel" class:tall aria-label="Sumber dokumentasi">
  <div class="panel-head">
    <h2 class="step-title">1&nbsp; Sumber Dokumentasi</h2>
    {#if hasScanned}
      <button class="btn-rescan" type="button" onclick={() => onRescan?.()} disabled={rescanDisabled || count === 0}>
        Pindai Ulang
      </button>
    {/if}
  </div>

  {#if count === 0}
    <div class="dropzone-large">
      <span class="drop-icon" aria-hidden="true">⬇</span>
      <span class="drop-line">Tarik folder / SD Card / drive ke sini</span>
      <button class="btn-browse" type="button" onclick={addSource}>Pilih Folder…</button>
      <span class="drop-hint">Bisa juga tarik beberapa sumber sekaligus</span>
    </div>
    <p class="format-hint">Didukung: JPG, PNG, HEIC, TIFF, MP4, MOV, AVI, MKV, 3GP</p>
  {:else}
    <div class="queue-list">
      {#each store.sourcePaths as path (path)}
        {@const sd = isSdCard(path)}
        <div class="queue-row" class:done={hasScanned}>
          <span class="q-path truncate" title={path}>
            {sd ? '🖧' : '📁'} {basename(path)}
          </span>
          {#if hasScanned}
            <span class="q-status">✓ {fileCount()} file siap</span>
          {:else}
            <span class="q-status pending">○ menunggu pindai</span>
          {/if}
          <button
            class="q-remove"
            type="button"
            title={`Hapus ${basename(path)}`}
            aria-label={`Hapus sumber ${basename(path)}`}
            onclick={() => store.removeSourcePath(path)}
          >✕</button>
        </div>
      {/each}
    </div>

    <button class="dropzone-mini" type="button" onclick={addSource}>
      <span class="mini-icon" aria-hidden="true">⬇</span>
      <span class="mini-text">
        <span class="mini-line">Tambah sumber lain — tarik folder ke sini</span>
        <span class="mini-link">atau Pilih Folder…</span>
      </span>
    </button>
  {/if}
</section>

<style>
  .source-panel {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-height: 240px;
    height: 240px;
    box-sizing: border-box;
    overflow: hidden;
  }

  .source-panel.tall {
    height: 320px;
  }

  .source-panel.tall .dropzone-large {
    min-height: 232px;
  }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .step-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    line-height: 1.2;
  }

  .btn-rescan {
    height: 22px;
    padding: 0 8px;
    background: transparent;
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    font-size: 10px;
    color: var(--text-2);
    cursor: pointer;
    font-family: inherit;
  }

  .btn-rescan:hover:not(:disabled) {
    background: var(--bg);
  }

  .btn-rescan:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Empty dropzone — large */
  .dropzone-large {
    flex: 1;
    background: var(--bg);
    border: 1px dashed var(--text-3);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 10px;
    min-height: 0;
  }

  .drop-icon {
    font-size: 28px;
    color: var(--text-3);
    line-height: 1;
  }

  .drop-line {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-2);
    text-align: center;
  }

  .btn-browse {
    height: 30px;
    padding: 0 14px;
    background: var(--card);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--primary-hover);
    cursor: pointer;
    font-family: inherit;
  }

  .btn-browse:hover {
    background: var(--border);
  }

  .drop-hint {
    font-size: 10px;
    color: var(--text-3);
  }

  .format-hint {
    margin: 0;
    font-size: 10px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Compact dropzone + queue */
  .queue-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 2px;
  }

  .queue-row {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    flex: 0 0 auto;
  }

  .queue-row.done {
    background: #F0FDF4;
    border-color: #BBDDE6;
  }

  .q-path {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    font-weight: 500;
    color: var(--text);
  }

  .queue-row.done .q-path {
    color: #166534;
  }

  .q-status {
    flex: 0 0 auto;
    font-size: 11px;
    font-weight: 500;
    color: #16A34A;
    white-space: nowrap;
  }

  .q-status.pending {
    color: var(--text-3);
  }

  .q-remove {
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    font-size: 11px;
    color: var(--text-3);
    cursor: pointer;
    border-radius: 3px;
    padding: 0;
    font-family: inherit;
  }

  .q-remove:hover {
    color: #DC2626;
    background: #FEF2F2;
  }

  .dropzone-mini {
    width: 100%;
    height: 44px;
    flex: 0 0 auto;
    background: var(--bg);
    border: 1px dashed var(--text-3);
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px;
    cursor: pointer;
    font-family: inherit;
    text-align: left;
  }

  .dropzone-mini:hover {
    background: var(--bg);
  }

  .mini-icon {
    font-size: 18px;
    color: var(--text-3);
  }

  .mini-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .mini-line {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-2);
  }

  .mini-link {
    font-size: 11px;
    color: #0066CC;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
