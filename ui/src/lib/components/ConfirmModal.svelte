<script lang="ts">
  import Modal from './Modal.svelte';
  import { store, formatBytes } from '../store.svelte';

  interface Props {
    open?: boolean;
    onCancel?: () => void;
    onStart?: () => void;
  }

  let { open = store.showConfirm, onCancel, onStart }: Props = $props();

  let acknowledged = $state(false);

  const mode = $derived(store.settings.transfer_mode);
  const isMove = $derived(mode === 'move');
  const selectedRows = $derived(store.previewRows.filter((r) => store.selectedRows.has(r.source)));
  const fileCount = $derived(selectedRows.length);
  const totalBytes = $derived(selectedRows.reduce((s, r) => s + r.size_bytes, 0));
  const activeDestinations = $derived(store.settings.destinations.filter((d) => d.enabled));
  const destCount = $derived(activeDestinations.length);
  const canStart = $derived(fileCount > 0 && destCount > 0 && (!isMove || acknowledged));

  // Reset acknowledgement whenever modal is (re)opened
  let wasOpen = false;
  $effect(() => {
    if (open && !wasOpen) acknowledged = false;
    wasOpen = open;
  });

  function close() {
    acknowledged = false;
    if (onCancel) onCancel();
    else store.showConfirm = false;
  }

  function start() {
    if (!canStart) return;
    if (onStart) onStart();
    else store.appState = 'memindahkan';
  }
</script>

<Modal title="Konfirmasi Mulai Transfer" open={open} onClose={close} width={520}>
  <div class="summary">
    <div class="summary-row">
      <span class="label">Mode transfer</span>
      <span class="value">
        {#if isMove}
          <span class="badge badge-amber">Pindah</span>
        {:else}
          <span class="badge badge-blue">Salin</span>
        {/if}
      </span>
    </div>
    <div class="summary-row">
      <span class="label">File terpilih</span>
      <span class="value">
        <strong>{fileCount}</strong> file
        <span class="muted">· {formatBytes(totalBytes)}</span>
      </span>
    </div>
    <div class="summary-row">
      <span class="label">Tujuan aktif</span>
      <span class="value"><strong>{destCount}</strong> tujuan</span>
    </div>
    {#if destCount > 0}
      <ul class="dest-list">
        {#each activeDestinations as d (d.id)}
          {@const folder = `MM-DD_${(d.activity || 'Nama Kegiatan').trim()}${(d.suffix ?? '').trim() ? `_${(d.suffix ?? '').trim()}` : ''}`}
          <li>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path
                d="M3 7a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.58L11.4 7H19a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linejoin="round"
              />
            </svg>
            <span class="dest-root truncate">{d.root || '(root belum dipilih)'}</span>
            <span class="dest-folder mono-sm">/{folder}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  {#if isMove}
    <div class="warning-banner" role="alert">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 4 2.8 19.5h18.4L12 4Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
        <path d="M12 10v4M12 17v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
      <div>
        <strong>PERHATIAN:</strong> Mode Pindah akan MENGHAPUS file sumber dari kartu memori / disk setelah berhasil
        disalin dan diverifikasi di seluruh tujuan.
      </div>
    </div>

    <label class="ack">
      <input type="checkbox" bind:checked={acknowledged} />
      <span>Saya memahami file sumber akan dihapus permanen dari lokasi asal</span>
    </label>
  {/if}

  {#if fileCount === 0}
    <p class="hint-block">Belum ada file terpilih. Pilih minimal satu file di tabel pratinjau.</p>
  {:else if destCount === 0}
    <p class="hint-block">Belum ada tujuan aktif. Aktifkan setidaknya satu tujuan pengarsipan.</p>
  {/if}

  {#snippet footer()}
    <button class="btn btn-secondary" type="button" onclick={close}>Batalkan</button>
    <button class="btn btn-primary" type="button" disabled={!canStart} onclick={start}>
      Mulai Transfer
    </button>
  {/snippet}
</Modal>

<style>
  .summary {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .summary-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13.5px;
  }

  .summary-row:last-of-type {
    border-bottom: none;
  }

  .label {
    color: var(--text-3);
  }

  .value {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }

  .value strong {
    font-weight: 600;
  }

  .muted {
    color: var(--text-3);
    font-size: 12.5px;
  }

  .dest-list {
    list-style: none;
    margin: 4px 0 0;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .dest-list li {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    font-size: 12.5px;
    color: var(--text-3);
  }

  .dest-root {
    color: var(--text-2);
    max-width: 200px;
  }

  .dest-folder {
    color: var(--text-3);
  }

  .warning-banner {
    display: flex;
    gap: 10px;
    margin-top: 14px;
    padding: 12px;
    border: 1px solid var(--warning-border);
    border-radius: var(--radius);
    background: var(--warning-bg);
    color: var(--warning);
    font-size: 13px;
    line-height: 1.5;
  }

  .warning-banner svg {
    flex: 0 0 auto;
    margin-top: 1px;
  }

  .ack {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin-top: 12px;
    font-size: 13px;
    color: var(--text-2);
    cursor: pointer;
    line-height: 1.45;
  }

  .ack input {
    margin-top: 2px;
    flex: 0 0 auto;
  }

  .hint-block {
    margin-top: 14px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--danger-bg);
    color: var(--danger);
    font-size: 12.5px;
  }
</style>
