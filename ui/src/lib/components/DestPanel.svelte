<script lang="ts">
  import { store } from '../store.svelte';
  import type { TransferMode, Destination } from '../types';
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    tall?: boolean;
  }

  let { tall = false }: Props = $props();

  const destinations = $derived(store.settings.destinations);

  function setMode(mode: TransferMode) {
    store.settings = { ...store.settings, transfer_mode: mode };
  }

  async function browse(id: string) {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih folder tujuan pengarsipan',
      });
      if (!selected || Array.isArray(selected)) return;
      store.updateDestination(id, { root: selected });
    } catch (err) {
      store.globalError = `Gagal membuka dialog folder: ${String(err)}`;
    }
  }

  function destLabel(d: Destination): string {
    return d.root || 'Pilih folder…';
  }
</script>

<section class="dest-panel" class:tall aria-label="Tujuan dan penamaan">
  <div class="panel-head">
    <h2 class="step-title">2 · Tujuan &amp; Penamaan</h2>
    <span class="step-desc">Boleh lebih dari satu tujuan</span>
  </div>

  <!-- Header row -->
  <div class="dest-header">
    <span class="h-root">Folder Tujuan</span>
    <span class="h-keg">Nama Kegiatan</span>
    <span class="h-sfx">Suffix</span>
    <span class="h-del">Hapus</span>
  </div>

  <div class="dest-rows">
    {#each destinations as d (d.id)}
      <div class="dest-row" class:disabled={!d.enabled}>
        <button
          class="field root-field"
          type="button"
          title={destLabel(d)}
          onclick={() => browse(d.id)}
        >
          <span class="field-val truncate">{destLabel(d)}</span>
        </button>
        <input
          class="field keg-field"
          type="text"
          placeholder="Nama kegiatan"
          value={d.activity}
          disabled={!d.enabled}
          oninput={(e) => store.updateDestination(d.id, { activity: e.currentTarget.value })}
        />
        <input
          class="field sfx-field"
          type="text"
          placeholder="Opsional"
          value={d.suffix ?? ''}
          disabled={!d.enabled}
          oninput={(e) => store.updateDestination(d.id, { suffix: e.currentTarget.value })}
        />
        <button
          class="del-field"
          type="button"
          disabled={destinations.length <= 1}
          title={destinations.length <= 1 ? 'Minimal satu tujuan' : 'Hapus tujuan ini'}
          onclick={() => store.removeDestination(d.id)}
        >Hapus</button>
      </div>
    {/each}
  </div>

  <button class="btn-add-dest" type="button" onclick={() => store.addDestination()}>
    + Tambah Tujuan
  </button>

  <p class="multi-hint">Setiap file akan dikirim ke SEMUA tujuan di atas</p>

  <div class="mode-section">
    <span class="lbl-mode">Mode Transfer</span>
    <div class="mode-group" role="group" aria-label="Mode transfer">
      <button
        class="mode-btn copy"
        class:active={store.settings.transfer_mode === 'copy'}
        type="button"
        aria-pressed={store.settings.transfer_mode === 'copy'}
        onclick={() => setMode('copy')}
      >
        <span class="mode-ico" aria-hidden="true">⧉</span>
        <span class="mode-text">
          <span class="mode-lbl">Copy</span>
          <span class="mode-sub">Sumber tetap</span>
        </span>
      </button>
      <button
        class="mode-btn move"
        class:active={store.settings.transfer_mode === 'move'}
        type="button"
        aria-pressed={store.settings.transfer_mode === 'move'}
        onclick={() => setMode('move')}
      >
        <span class="mode-ico" aria-hidden="true">↦</span>
        <span class="mode-text">
          <span class="mode-lbl">Move</span>
          <span class="mode-sub">Sumber dihapus</span>
        </span>
      </button>
    </div>
  </div>
</section>

<style>
  .dest-panel {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    height: 240px;
    box-sizing: border-box;
    overflow: hidden;
  }

  .dest-panel.tall {
    height: 320px;
  }

  .dest-panel.tall .dest-rows {
    max-height: 160px;
  }

  .panel-head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 6px;
  }

  .step-title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--text);
    line-height: 1.2;
    white-space: nowrap;
  }

  .step-desc {
    font-size: 10px;
    color: var(--text-3);
    white-space: nowrap;
  }

  /* Header */
  .dest-header,
  .dest-row {
    display: grid;
    grid-template-columns: minmax(0, 2.5fr) minmax(0, 2fr) minmax(0, 1.8fr) 50px;
    gap: 8px;
    align-items: center;
  }

  .dest-header {
    height: 20px;
    padding: 0 6px;
    background: var(--bg);
    border-radius: 4px;
  }

  .dest-header span {
    font-size: 9px;
    font-weight: 500;
    color: var(--text-3);
    white-space: nowrap;
  }

  .dest-rows {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 118px;
    overflow-y: auto;
    padding-right: 2px;
  }

  .dest-row {
    height: 36px;
    padding: 0 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--card);
  }

  .dest-row.disabled {
    opacity: 0.55;
  }

  .field {
    height: 26px;
    padding: 0 6px;
    background: var(--bg);
    border: none;
    border-radius: 3px;
    font-size: 10px;
    color: var(--text);
    font-family: inherit;
    min-width: 0;
    text-align: left;
  }

  input.field {
    outline: none;
    width: 100%;
  }

  input.field:focus {
    background: var(--primary-soft);
    box-shadow: 0 0 0 1px #0F62FE;
  }

  input.field::placeholder {
    color: var(--text-3);
  }

  .root-field {
    cursor: pointer;
    display: flex;
    align-items: center;
    overflow: hidden;
  }

  .root-field:hover {
    background: var(--bg);
  }

  .field-val {
    font-size: 10px;
    color: var(--text);
  }

  .del-field {
    height: 26px;
    background: #FFF1F2;
    border: none;
    border-radius: 3px;
    font-size: 9px;
    color: #DC2626;
    cursor: pointer;
    font-family: inherit;
  }

  .del-field:hover:not(:disabled) {
    background: #FFE4E6;
  }

  .del-field:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-add-dest {
    width: 100%;
    height: 30px;
    margin-top: 4px;
    background: var(--primary-soft);
    border: 1px solid #BFDBFE;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    color: #0F62FE;
    cursor: pointer;
    font-family: inherit;
  }

  .btn-add-dest:hover {
    background: #DBEAFE;
  }

  .multi-hint {
    margin: 2px 0 0;
    font-size: 9px;
    color: var(--text-3);
  }

  .mode-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 6px;
  }

  .lbl-mode {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-2);
  }

  .mode-group {
    display: flex;
    gap: 10px;
  }

  .mode-btn {
    height: 34px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px;
    border-radius: 5px;
    border: 1px solid transparent;
    background: var(--bg);
    border-color: var(--border-strong);
    cursor: pointer;
    font-family: inherit;
    transition: background 0.12s ease, border-color 0.12s ease;
  }

  .mode-btn.copy {
    width: 130px;
  }

  .mode-btn.move {
    width: 150px;
  }

  .mode-btn.copy.active {
    background: #0F62FE;
    border-color: #0F62FE;
  }

  .mode-btn.move.active {
    background: #0F62FE;
    border-color: #0F62FE;
  }

  .mode-ico {
    font-size: 13px;
    color: var(--text-2);
  }

  .mode-btn.active .mode-ico {
    color: #ffffff;
  }

  .mode-text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0;
    line-height: 1.15;
  }

  .mode-lbl {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-2);
  }

  .mode-btn.copy.active .mode-lbl {
    font-weight: 700;
    color: #ffffff;
  }

  .mode-btn.move.active .mode-lbl {
    color: #ffffff;
  }

  .mode-sub {
    font-size: 8px;
    color: var(--text-3);
  }

  .mode-btn.copy.active .mode-sub {
    color: #DBEAFE;
  }

  .mode-btn.move.active .mode-sub {
    color: #DBEAFE;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
