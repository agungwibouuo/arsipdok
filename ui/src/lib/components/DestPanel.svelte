<script lang="ts">
  import { store } from '../store.svelte';
  import type { TransferMode, Destination } from '../types';
  import { open } from '@tauri-apps/plugin-dialog';

  const destinations = $derived(store.settings.destinations);
  const canRemove = $derived(destinations.length > 1);
  const activeCount = $derived(destinations.filter((d) => d.enabled).length);

  function setMode(mode: TransferMode) {
    store.settings = { ...store.settings, transfer_mode: mode };
  }

  function previewFolder(d: Destination): string {
    const activity = (d.activity || 'Nama Kegiatan').trim();
    const suffix = (d.suffix ?? '').trim();
    return `MM-DD_${activity}${suffix ? `_${suffix}` : ''}`;
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
</script>

<section class="dest-panel panel" aria-label="Tujuan pengarsipan">
  <header class="panel-header">
    <div class="title-wrap">
      <h2>Tujuan Pengarsipan</h2>
      {#if activeCount > 0}
        <span class="badge badge-slate">{activeCount} aktif</span>
      {/if}
    </div>

    <div class="mode-switch" role="group" aria-label="Mode transfer">
      <button
        class="mode-btn"
        class:active={store.settings.transfer_mode === 'copy'}
        type="button"
        aria-pressed={store.settings.transfer_mode === 'copy'}
        onclick={() => setMode('copy')}
      >
        Salin
      </button>
      <button
        class="mode-btn"
        class:active={store.settings.transfer_mode === 'move'}
        type="button"
        aria-pressed={store.settings.transfer_mode === 'move'}
        onclick={() => setMode('move')}
      >
        Pindah
      </button>
    </div>
  </header>

  {#if store.settings.transfer_mode === 'move'}
    <p class="move-warning">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path d="M12 4 2.8 19.5h18.4L12 4Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
        <path d="M12 10v4M12 17v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
      <span>File sumber dihapus setelah diverifikasi di SEMUA tujuan</span>
    </p>
  {/if}

  {#if destinations.length === 0}
    <p class="empty-hint">Belum ada tujuan. Tambahkan setidaknya satu folder tujuan pengarsipan.</p>
  {:else}
    <ul class="dest-list">
      {#each destinations as d (d.id)}
        <li class="dest-row" class:disabled={!d.enabled}>
          <label class="enable-toggle" title={d.enabled ? 'Nonaktifkan tujuan' : 'Aktifkan tujuan'}>
            <input
              type="checkbox"
              checked={d.enabled}
              onchange={(e) => store.updateDestination(d.id, { enabled: e.currentTarget.checked })}
            />
            <span class="checkbox-visual" aria-hidden="true">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                <path d="m5 12.5 4.5 4.5L19 7.5" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </span>
            <span class="sr-only">Aktifkan tujuan</span>
          </label>

          <div class="fields">
            <div class="field root-field">
              <span class="field-label">Root Folder</span>
              <div class="root-input">
                <input
                  class="input"
                  type="text"
                  placeholder="Pilih folder tujuan..."
                  value={d.root}
                  disabled={!d.enabled}
                  oninput={(e) => store.updateDestination(d.id, { root: e.currentTarget.value })}
                />
                <button
                  class="btn btn-secondary btn-sm browse-btn"
                  type="button"
                  disabled={!d.enabled}
                  onclick={() => browse(d.id)}
                >
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <path
                      d="M3 7a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.58L11.4 7H19a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"
                      stroke="currentColor"
                      stroke-width="1.8"
                      stroke-linejoin="round"
                    />
                  </svg>
                  <span>Browse</span>
                </button>
              </div>
            </div>

            <div class="field-grid">
              <div class="field">
                <span class="field-label">Nama Kegiatan</span>
                <input
                  class="input"
                  type="text"
                  placeholder="cth: Kunjungan Lapangan"
                  value={d.activity}
                  disabled={!d.enabled}
                  oninput={(e) => store.updateDestination(d.id, { activity: e.currentTarget.value })}
                />
              </div>
              <div class="field">
                <span class="field-label">Suffix <em>(opsional)</em></span>
                <input
                  class="input"
                  type="text"
                  placeholder="cth: Hari 1"
                  value={d.suffix ?? ''}
                  disabled={!d.enabled}
                  oninput={(e) => store.updateDestination(d.id, { suffix: e.currentTarget.value })}
                />
              </div>
            </div>

            <p class="folder-preview mono-sm" title={previewFolder(d)}>
              Format folder: <strong>{previewFolder(d)}</strong>
            </p>
          </div>

          <button
            class="remove-btn"
            type="button"
            disabled={!canRemove}
            title={canRemove ? 'Hapus tujuan ini' : 'Minimal satu tujuan'}
            aria-label="Hapus tujuan"
            onclick={() => store.removeDestination(d.id)}
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M4 7h16M9 7V4.5h6V7M6.5 7l.8 12.2a2 2 0 0 0 2 1.8h5.4a2 2 0 0 0 2-1.8L17.5 7" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M10 11v5M14 11v5" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <button class="btn btn-secondary add-btn" type="button" onclick={() => store.addDestination()}>
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </svg>
    <span>Tambah Tujuan</span>
  </button>
</section>

<style>
  .dest-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
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

  .mode-switch {
    display: inline-flex;
    padding: 3px;
    gap: 2px;
    background: #f1f5f9;
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .mode-btn {
    height: 28px;
    padding: 0 16px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease, box-shadow 0.12s ease;
  }

  .mode-btn:hover:not(.active) {
    color: var(--text-2);
  }

  .mode-btn.active {
    background: var(--primary);
    color: #fff;
    box-shadow: var(--shadow-sm);
  }

  .move-warning {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--warning);
    background: var(--warning-bg);
    border: 1px solid var(--warning-border);
    border-radius: var(--radius-sm);
    padding: 7px 10px;
  }

  .move-warning svg {
    flex: 0 0 auto;
  }

  .empty-hint {
    font-size: 13px;
    color: var(--text-3);
  }

  .dest-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dest-row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg);
    transition: border-color 0.12s ease, opacity 0.12s ease;
  }

  .dest-row:hover {
    border-color: var(--border-strong);
  }

  .dest-row.disabled {
    opacity: 0.62;
    background: transparent;
  }

  /* Checkbox kustom */
  .enable-toggle {
    flex: 0 0 auto;
    position: relative;
    display: inline-flex;
    align-items: center;
    margin-top: 20px;
    cursor: pointer;
  }

  .enable-toggle input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .checkbox-visual {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: 1.5px solid var(--border-strong);
    border-radius: 5px;
    background: var(--card);
    color: transparent;
    transition: background 0.12s ease, border-color 0.12s ease, color 0.12s ease;
  }

  .enable-toggle input:checked + .checkbox-visual {
    background: var(--primary);
    border-color: var(--primary);
    color: #fff;
  }

  .enable-toggle input:focus-visible + .checkbox-visual {
    box-shadow: 0 0 0 3px rgba(15, 98, 254, 0.2);
  }

  .fields {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--text-3);
  }

  .field-label em {
    font-style: normal;
    font-weight: 500;
    text-transform: none;
    letter-spacing: 0;
    opacity: 0.75;
  }

  .root-input {
    display: flex;
    gap: 8px;
  }

  .root-input .input {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 12.5px;
  }

  .browse-btn {
    flex: 0 0 auto;
  }

  .field-grid {
    display: grid;
    grid-template-columns: 1fr minmax(140px, 0.6fr);
    gap: 8px;
  }

  .folder-preview {
    color: var(--text-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-preview strong {
    color: var(--text-2);
    font-weight: 600;
  }

  .remove-btn {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    margin-top: 18px;
    border-radius: var(--radius-sm);
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease;
  }

  .remove-btn:hover:not(:disabled) {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .remove-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .add-btn {
    align-self: flex-start;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
