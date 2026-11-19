<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    onBrowse?: () => void;
    onDropFiles?: (paths: string[]) => void;
    dragging?: boolean;
  }

  let { onBrowse, onDropFiles, dragging = false }: Props = $props();

  let dragDepth = $state(0);
  let hoverDrop = $state(false);
  let hoverBox = $state(false);

  const isDragging = $derived(dragging || dragDepth > 0 || hoverDrop || hoverBox);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    // Tauri drag-drop (v2) — diimpor dinamis agar aman di browser/dev tanpa Tauri.
    (async () => {
      try {
        const mod = await import('@tauri-apps/api/webviewWindow');
        const webview = mod.getCurrentWebviewWindow();
        const un =
          (await webview.onDragDropEvent((event) => {
            const payload = event.payload;
            if (payload.type === 'enter' || payload.type === 'over') {
              dragDepth = 1;
            } else if (payload.type === 'leave') {
              dragDepth = 0;
            } else if (payload.type === 'drop') {
              dragDepth = 0;
              if (payload.paths?.length) onDropFiles?.(payload.paths);
            }
          })) ?? undefined;
        if (!cancelled) unlisten = un as (() => void) | undefined;
        else un?.();
      } catch {
        // Tidak berjalan di Tauri — abaikan.
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  });

  function onDragEnter(e: DragEvent) {
    e.preventDefault();
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragDepth = 1;
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    dragDepth = 0;
    hoverDrop = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragDepth = 0;
    hoverDrop = false;
    const paths: string[] = [];
    const files = e.dataTransfer?.files;
    if (files) {
      for (const file of Array.from(files)) {
        const path = (file as File & { path?: string }).path;
        if (path) paths.push(path);
      }
    }
    const text = e.dataTransfer?.getData('text/uri-list') ?? '';
    if (text) {
      for (const line of text.split(/\r?\n/)) {
        const uri = line.trim();
        if (!uri || uri.startsWith('#')) continue;
        paths.push(uri.startsWith('file://') ? decodeURIComponent(uri.replace(/^file:\/\//, '')) : uri);
      }
    }
    if (paths.length) onDropFiles?.(paths);
  }
</script>

<div
  class="empty"
  ondragenter={onDragEnter}
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
  role="region"
  aria-label="Belum ada sumber dokumentasi"
>
  <div class="icon-box" aria-hidden="true">
    <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
      <path
        d="M6 15a3 3 0 0 1 3-3h8.2a3 3 0 0 1 2.12.88l2.68 2.68a3 3 0 0 0 2.12.88H39a3 3 0 0 1 3 3V36a3 3 0 0 1-3 3H9a3 3 0 0 1-3-3V15Z"
        stroke="var(--primary)"
        stroke-width="2.5"
        stroke-linejoin="round"
      />
      <path d="M24 24v9M19.5 28.5 24 24l4.5 4.5" stroke="var(--primary)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </div>

  <h1 class="title">Belum ada sumber dokumentasi</h1>
  <p class="desc">
    Tarik folder SD card, NAS, atau folder lokal ke jendela ini, atau klik tombol di bawah untuk memilih.
  </p>

  <button class="btn btn-primary btn-lg" type="button" onclick={onBrowse}>
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </svg>
    <span>Pilih Folder Sumber</span>
  </button>

  <p class="hint">Format yang didukung: JPG, PNG, HEIC, MP4, MOV, dan lainnya</p>

  <div
    class="dropzone"
    class:dragging={isDragging}
    onmouseenter={() => (hoverBox = true)}
    onmouseleave={() => (hoverBox = false)}
    role="presentation"
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M12 3v11m0 0 4-4m-4 4-4-4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <span>Lepas di sini untuk menambahkan sumber</span>
  </div>
</div>

<style>
  .empty {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 32px 24px;
    gap: 12px;
    overflow-y: auto;
  }

  .icon-box {
    width: 96px;
    height: 96px;
    border-radius: 24px;
    background: var(--primary-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 8px;
  }

  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
  }

  .desc {
    font-size: 14px;
    color: var(--text-3);
    max-width: 460px;
    line-height: 1.6;
    margin-bottom: 8px;
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
    margin-top: 4px;
  }

  .dropzone {
    width: 520px;
    max-width: 100%;
    height: 120px;
    margin-top: 16px;
    border: 2px dashed var(--border-strong);
    border-radius: 12px;
    background: transparent;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-3);
    font-size: 13px;
    transition: border-color 0.12s ease, background 0.12s ease, color 0.12s ease;
  }

  .dropzone.dragging {
    border-color: var(--primary);
    background: var(--primary-soft);
    color: var(--primary);
  }
</style>
