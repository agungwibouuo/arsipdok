<script lang="ts">
  import Modal from './Modal.svelte';
  import { store } from '../store.svelte';
  import { api } from '../api';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import type { Settings, TransferMode, VerificationMode } from '../types';

  interface Props {
    open?: boolean;
    onClose?: () => void;
  }

  let { open = store.showSettings, onClose }: Props = $props();

  type Tab = 'umum' | 'ekstensi' | 'verifikasi' | 'lanjutan';

  let tab = $state<Tab>('umum');

  // Draft lokal — hanya dikomit ke store saat "Simpan" ditekan.
  // $state.snapshot() diperlukan: store.settings adalah proxy reaktif Svelte
  // yang TIDAK bisa di-structuredClone (DataCloneError).
  let draft = $state<Settings>($state.snapshot(store.settings));

  const DEFAULT_PHOTO_EXTS = [
    'jpg', 'jpeg', 'png', 'heic', 'heif', 'webp', 'tif', 'tiff', 'cr2', 'cr3', 'nef', 'arw', 'dng',
  ];
  const DEFAULT_VIDEO_EXTS = [
    'mp4', 'mov', 'm4v', 'avi', 'mkv', 'mts', 'm2ts', '3gp', 'wmv', 'flv', 'webm', 'mpg', 'mpeg',
  ];

  // Ekstensi tidak tersimpan di struct Settings Rust — dikelola lokal di modal
  let photoExts = $state<string[]>([...DEFAULT_PHOTO_EXTS]);
  let videoExts = $state<string[]>([...DEFAULT_VIDEO_EXTS]);
  let newPhotoExt = $state('');
  let newVideoExt = $state('');

  let logDir = $state('');
  let saving = $state(false);

  // Muat ulang draft setiap kali modal dibuka
  let wasOpen = false;
  $effect(() => {
    if (open && !wasOpen) {
      draft = $state.snapshot(store.settings);
      tab = 'umum';
    }
    wasOpen = open;
  });

  function normalizeExt(value: string): string {
    return value.trim().replace(/^\./, '').toLowerCase();
  }

  function addExt(kind: 'photo' | 'video') {
    const list = kind === 'photo' ? newPhotoExt : newVideoExt;
    const ext = normalizeExt(list);
    if (!ext) return;
    const target = kind === 'photo' ? photoExts : videoExts;
    const other = kind === 'photo' ? videoExts : photoExts;
    if (target.includes(ext) || other.includes(ext)) return;
    if (kind === 'photo') {
      photoExts = [...photoExts, ext];
      newPhotoExt = '';
    } else {
      videoExts = [...videoExts, ext];
      newVideoExt = '';
    }
  }

  function removeExt(kind: 'photo' | 'video', ext: string) {
    if (kind === 'photo') photoExts = photoExts.filter((x) => x !== ext);
    else videoExts = videoExts.filter((x) => x !== ext);
  }

  function onExtKeydown(e: KeyboardEvent, kind: 'photo' | 'video') {
    if (e.key === 'Enter') {
      e.preventDefault();
      addExt(kind);
    }
  }

  function setMode(mode: TransferMode) {
    draft = { ...draft, transfer_mode: mode };
  }

  function setVerification(mode: VerificationMode) {
    draft = { ...draft, verification: mode };
  }

  function setWorkers(n: number) {
    const clamped = Math.min(8, Math.max(1, Math.round(n) || 1));
    draft = { ...draft, parallel_workers: clamped };
  }

  async function browseLogDir() {
    try {
      const selected = await openDialog({ directory: true, multiple: false, title: 'Pilih direktori log' });
      if (!selected || Array.isArray(selected)) return;
      logDir = selected;
    } catch (err) {
      store.globalError = `Gagal membuka dialog folder: ${String(err)}`;
    }
  }

  function resetDefaults() {
    draft = {
      version: 2,
      destinations: store.settings.destinations,
      root_history: [],
      activity_history: [],
      suffix_history: [],
      transfer_mode: 'copy',
      verification: 'size_mtime',
      parallel_workers: 4,
      remove_empty_source_dirs: true,
      confirm_before_start: true,
      language: 'id',
    };
    photoExts = [...DEFAULT_PHOTO_EXTS];
    videoExts = [...DEFAULT_VIDEO_EXTS];
    logDir = '';
  }

  function close() {
    if (onClose) onClose();
    else store.showSettings = false;
  }

  async function save() {
    saving = true;
    try {
      const next: Settings = { ...draft };
      store.settings = next;
      await api.saveSettings(next);
      close();
    } catch (err) {
      store.globalError = `Gagal menyimpan pengaturan: ${String(err)}`;
    } finally {
      saving = false;
    }
  }
</script>

<Modal title="Pengaturan" subtitle="Sesuaikan cara ArsipDok memindai, mentransfer, dan memverifikasi file." open={open} onClose={close} width={640}>
  <div class="settings">
    <div class="tabs" role="tablist" aria-label="Kategori pengaturan">
      <button class="tab" class:active={tab === 'umum'} role="tab" aria-selected={tab === 'umum'} type="button" onclick={() => (tab = 'umum')}>Umum</button>
      <button class="tab" class:active={tab === 'ekstensi'} role="tab" aria-selected={tab === 'ekstensi'} type="button" onclick={() => (tab = 'ekstensi')}>Ekstensi</button>
      <button class="tab" class:active={tab === 'verifikasi'} role="tab" aria-selected={tab === 'verifikasi'} type="button" onclick={() => (tab = 'verifikasi')}>Verifikasi</button>
      <button class="tab" class:active={tab === 'lanjutan'} role="tab" aria-selected={tab === 'lanjutan'} type="button" onclick={() => (tab = 'lanjutan')}>Lanjutan</button>
    </div>

    <div class="tab-body">
      {#if tab === 'umum'}
        <div class="section">
          <div class="field">
            <span class="field-label">Mode transfer default</span>
            <div class="mode-switch" role="group" aria-label="Mode transfer default">
              <button class="mode-btn" class:active={draft.transfer_mode === 'copy'} type="button" onclick={() => setMode('copy')}>
                Salin
              </button>
              <button class="mode-btn" class:active={draft.transfer_mode === 'move'} type="button" onclick={() => setMode('move')}>
                Pindah
              </button>
            </div>
            <span class="field-hint">Mode Pindah akan menghapus file sumber setelah diverifikasi di semua tujuan.</span>
          </div>

          <label class="switch-row">
            <div class="switch-text">
              <span class="switch-title">Konfirmasi sebelum mulai</span>
              <span class="switch-desc">Tampilkan dialog konfirmasi ringkasan sebelum transfer dimulai.</span>
            </div>
            <input class="toggle" type="checkbox" bind:checked={draft.confirm_before_start} />
          </label>

          <label class="switch-row">
            <div class="switch-text">
              <span class="switch-title">Hapus folder kosong sumber</span>
              <span class="switch-desc">Setelah mode Pindah selesai, hapus folder sumber yang menjadi kosong.</span>
            </div>
            <input class="toggle" type="checkbox" bind:checked={draft.remove_empty_source_dirs} />
          </label>
        </div>
      {:else if tab === 'ekstensi'}
        <div class="section">
          <div class="ext-group">
            <h3 class="ext-title">
              <span class="dot-photo" aria-hidden="true"></span> Ekstensi Foto
              <span class="count">{photoExts.length}</span>
            </h3>
            <div class="chips">
              {#each photoExts as ext (ext)}
                <span class="chip">
                  <span class="mono">{ext}</span>
                  <button class="chip-remove" type="button" aria-label={`Hapus ${ext}`} onclick={() => removeExt('photo', ext)}>✕</button>
                </span>
              {/each}
            </div>
            <div class="add-ext">
              <input
                class="input"
                type="text"
                placeholder="cth: avif"
                bind:value={newPhotoExt}
                onkeydown={(e) => onExtKeydown(e, 'photo')}
              />
              <button class="btn btn-secondary btn-sm" type="button" onclick={() => addExt('photo')}>Tambah</button>
            </div>
          </div>

          <div class="ext-group">
            <h3 class="ext-title">
              <span class="dot-video" aria-hidden="true"></span> Ekstensi Video
              <span class="count">{videoExts.length}</span>
            </h3>
            <div class="chips">
              {#each videoExts as ext (ext)}
                <span class="chip">
                  <span class="mono">{ext}</span>
                  <button class="chip-remove" type="button" aria-label={`Hapus ${ext}`} onclick={() => removeExt('video', ext)}>✕</button>
                </span>
              {/each}
            </div>
            <div class="add-ext">
              <input
                class="input"
                type="text"
                placeholder="cth: mxf"
                bind:value={newVideoExt}
                onkeydown={(e) => onExtKeydown(e, 'video')}
              />
              <button class="btn btn-secondary btn-sm" type="button" onclick={() => addExt('video')}>Tambah</button>
            </div>
          </div>
        </div>
      {:else if tab === 'verifikasi'}
        <div class="section">
          <button
            class="choice-card"
            class:active={draft.verification === 'size_mtime'}
            type="button"
            onclick={() => setVerification('size_mtime')}
          >
            <span class="radio-visual" aria-hidden="true"></span>
            <span class="choice-body">
              <span class="choice-title">
                Ukuran + Waktu Modifikasi
                <span class="badge badge-green">Cepat · Default</span>
              </span>
              <span class="choice-desc">Membandingkan ukuran file dan waktu modifikasi tujuan. Cepat, dengan keandalan yang baik untuk sebagian besar kasus.</span>
            </span>
          </button>

          <button
            class="choice-card"
            class:active={draft.verification === 'blake3'}
            type="button"
            onclick={() => setVerification('blake3')}
          >
            <span class="radio-visual" aria-hidden="true"></span>
            <span class="choice-body">
              <span class="choice-title">
                BLAKE3 Checksum
                <span class="badge badge-amber">Lebih lama</span>
              </span>
              <span class="choice-desc">Menghitung checksum kriptografis di sumber dan tujuan. Paling akurat, tetapi membutuhkan waktu lebih lama pada file besar.</span>
            </span>
          </button>
        </div>
      {:else}
        <div class="section">
          <div class="field">
            <span class="field-label">Jumlah worker paralel</span>
            <div class="workers">
              <input
                class="range"
                type="range"
                min="1"
                max="8"
                step="1"
                value={draft.parallel_workers}
                aria-label="Jumlah worker paralel"
                oninput={(e) => setWorkers(Number(e.currentTarget.value))}
              />
              <span class="worker-value">{draft.parallel_workers}</span>
            </div>
            <span class="field-hint">1–8 worker. Nilai lebih tinggi dapat mempercepat transfer pada disk yang cepat.</span>
          </div>

          <div class="field">
            <span class="field-label">Direktori log default</span>
            <div class="root-input">
              <input class="input" type="text" placeholder="Direktori bawaan aplikasi" bind:value={logDir} />
              <button class="btn btn-secondary btn-sm" type="button" onclick={browseLogDir}>Browse</button>
            </div>
          </div>

          <div class="danger-zone">
            <div class="danger-text">
              <span class="switch-title">Reset ke pengaturan bawaan</span>
              <span class="switch-desc">Kembalikan semua opsi di atas ke nilai default. Tujuan pengarsipan tidak diubah.</span>
            </div>
            <button class="btn btn-secondary btn-sm" type="button" onclick={resetDefaults}>Reset</button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    <button class="btn btn-secondary" type="button" onclick={close} disabled={saving}>Batal</button>
    <button class="btn btn-primary" type="button" onclick={save} disabled={saving}>
      {saving ? 'Menyimpan...' : 'Simpan'}
    </button>
  {/snippet}
</Modal>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 320px;
  }

  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
  }

  .tab {
    position: relative;
    padding: 8px 14px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-3);
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    transition: color 0.12s ease, background 0.12s ease;
  }

  .tab:hover:not(.active) {
    color: var(--text-2);
    background: var(--bg);
  }

  .tab.active {
    color: var(--primary);
  }

  .tab.active::after {
    content: '';
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: -1px;
    height: 2px;
    border-radius: 2px;
    background: var(--primary);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }

  .field-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--text-3);
  }

  .field-hint {
    font-size: 12px;
    color: var(--text-3);
  }

  .mode-switch {
    display: inline-flex;
    align-self: flex-start;
    padding: 3px;
    gap: 2px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 999px;
  }

  .mode-btn {
    height: 28px;
    padding: 0 18px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease;
  }

  .mode-btn.active {
    background: var(--primary);
    color: #fff;
  }

  .switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
  }

  .switch-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .switch-title {
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text);
  }

  .switch-desc {
    font-size: 12px;
    color: var(--text-3);
  }

  /* Toggle switch */
  .toggle {
    flex: 0 0 auto;
    appearance: none;
    width: 38px;
    height: 22px;
    border-radius: 999px;
    background: var(--border-strong);
    position: relative;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .toggle::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow: var(--shadow-sm);
    transition: transform 0.15s ease;
  }

  .toggle:checked {
    background: var(--primary);
  }

  .toggle:checked::after {
    transform: translateX(16px);
  }

  /* Ekstensi */
  .ext-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .ext-title {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
  }

  .dot-photo,
  .dot-video {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--primary);
  }

  .dot-video {
    background: var(--warning);
  }

  .count {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-3);
    background: var(--bg);
    border-radius: 999px;
    padding: 0 7px;
    line-height: 18px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 4px 3px 9px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg);
    font-size: 12px;
    color: var(--text-2);
  }

  .chip .mono {
    font-size: 11.5px;
  }

  .chip-remove {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    font-size: 9px;
    color: var(--text-3);
  }

  .chip-remove:hover {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .add-ext {
    display: flex;
    gap: 8px;
    max-width: 320px;
  }

  /* Verifikasi */
  .choice-card {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px;
    text-align: left;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--card);
    transition: border-color 0.12s ease, background 0.12s ease;
  }

  .choice-card:hover {
    border-color: var(--border-strong);
  }

  .choice-card.active {
    border-color: var(--primary);
    background: var(--primary-soft);
  }

  .radio-visual {
    flex: 0 0 auto;
    width: 18px;
    height: 18px;
    margin-top: 1px;
    border-radius: 50%;
    border: 1.5px solid var(--border-strong);
    background: var(--card);
    transition: border-color 0.12s ease;
  }

  .choice-card.active .radio-visual {
    border-color: var(--primary);
    border-width: 5px;
  }

  .choice-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .choice-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text);
  }

  .choice-desc {
    font-size: 12.5px;
    color: var(--text-3);
    line-height: 1.5;
  }

  /* Lanjutan */
  .workers {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .range {
    flex: 1;
    accent-color: var(--primary);
  }

  .worker-value {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    height: 28px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    font-weight: 600;
    color: var(--text);
    font-variant-numeric: tabular-nums;
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

  .danger-zone {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 14px;
    border: 1px solid #fecaca;
    border-radius: var(--radius);
    background: var(--danger-bg);
  }

  .danger-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .danger-text .switch-title {
    color: var(--danger);
  }
</style>
