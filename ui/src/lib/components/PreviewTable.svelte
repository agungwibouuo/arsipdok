<script lang="ts">
  import { store, formatBytes, sourceLabel, metaSourceBadgeColor } from '../store.svelte';
  import type { PreviewRow, FileKind } from '../types';

  interface Props {
    onEditDate?: (row: PreviewRow, applyToFolder: boolean) => void;
  }

  let { onEditDate }: Props = $props();

  type KindFilter = 'all' | FileKind;
  type ReviewFilter = 'all' | 'review';

  let search = $state('');
  let kindFilter = $state<KindFilter>('all');
  let reviewFilter = $state<ReviewFilter>('all');

  // Baris yang sedang diedit tanggalnya (source path) + state popover
  let editingSource = $state<string | null>(null);
  let editDate = $state('');
  let applyScope = $state<'file' | 'folder'>('file');

  const rows = $derived(store.previewRows);

  const filtered = $derived(
    rows.filter((r) => {
      if (kindFilter !== 'all' && r.kind !== kindFilter) return false;
      if (reviewFilter === 'review' && !(r.needs_review && !r.date_override)) return false;
      const q = search.trim().toLowerCase();
      if (q && !r.name.toLowerCase().includes(q)) return false;
      return true;
    })
  );

  const allSelected = $derived(filtered.length > 0 && filtered.every((r) => store.selectedRows.has(r.source)));
  const someSelected = $derived(filtered.some((r) => store.selectedRows.has(r.source)));
  const selectedCount = $derived(store.selectedRows.size);

  const editingRow = $derived(rows.find((r) => r.source === editingSource) ?? null);

  function toggleAll() {
    const next = new Set(store.selectedRows);
    if (allSelected) {
      for (const r of filtered) next.delete(r.source);
    } else {
      for (const r of filtered) next.add(r.source);
    }
    store.selectedRows = next;
  }

  function toggleRow(source: string) {
    store.toggleRowSelect(source);
  }

  function dateOf(r: PreviewRow): string {
    return r.date_override ?? r.date;
  }

  function folderName(r: PreviewRow): string {
    const d = dateOf(r);
    const dest = store.settings.destinations.find((x) => x.enabled);
    const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest?.suffix ?? '').trim();
    return `${d}_${activity}${suffix ? `_${suffix}` : ''}`;
  }

  function openEditor(r: PreviewRow) {
    editingSource = r.source;
    editDate = dateOf(r);
    applyScope = 'file';
  }

  function closeEditor() {
    editingSource = null;
  }

  function applyEdit() {
    if (!editingRow || !editDate) return;
    if (onEditDate) {
      onEditDate(editingRow, applyScope === 'folder');
    } else {
      const targetFolder = folderOf(editingRow.source);
      const sources =
        applyScope === 'folder'
          ? rows.filter((x) => folderOf(x.source) === targetFolder).map((x) => x.source)
          : [editingRow.source];
      for (const src of sources) store.setDateOverride(src, editDate);
    }
    closeEditor();
  }

  function folderOf(p: string): string {
    const idx = Math.max(p.lastIndexOf('\\'), p.lastIndexOf('/'));
    return idx >= 0 ? p.slice(0, idx) : p;
  }

  function badgeClass(r: PreviewRow): string {
    if (r.date_override) return 'badge-blue';
    if (r.needs_review) return 'badge-amber';
    return 'badge-slate';
  }

  function badgeLabel(r: PreviewRow): string {
    return r.date_override ? 'Manual' : sourceLabel(r.metadata_source);
  }
</script>

<section class="preview panel" aria-label="Pratinjau file">
  <div class="filter-bar">
    <div class="search-wrap">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="11" cy="11" r="7" stroke="currentColor" stroke-width="1.8" />
        <path d="m16.5 16.5 4 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      </svg>
      <input class="input search-input" type="search" placeholder="Cari nama file..." bind:value={search} />
    </div>

    <div class="segmented" role="group" aria-label="Filter tipe file">
      <button class="seg" class:active={kindFilter === 'all'} type="button" onclick={() => (kindFilter = 'all')}>Semua</button>
      <button class="seg" class:active={kindFilter === 'photo'} type="button" onclick={() => (kindFilter = 'photo')}>Foto</button>
      <button class="seg" class:active={kindFilter === 'video'} type="button" onclick={() => (kindFilter = 'video')}>Video</button>
    </div>

    <div class="segmented" role="group" aria-label="Filter review">
      <button class="seg" class:active={reviewFilter === 'all'} type="button" onclick={() => (reviewFilter = 'all')}>Semua</button>
      <button class="seg" class:active={reviewFilter === 'review'} type="button" onclick={() => (reviewFilter = 'review')}>
        Perlu Review
        {#if store.needsReviewCount > 0}
          <span class="mini-count">{store.needsReviewCount}</span>
        {/if}
      </button>
    </div>

    <div class="select-all-wrap">
      <label class="select-all">
        <input
          type="checkbox"
          checked={allSelected}
          indeterminate={someSelected && !allSelected}
          onchange={toggleAll}
        />
        <span>Pilih Semua</span>
      </label>
      <span class="badge badge-blue">{selectedCount} terpilih</span>
    </div>
  </div>

  <div class="table-wrap">
    <table class="preview-table">
      <thead>
        <tr>
          <th class="col-check" scope="col"><span class="sr-only">Pilih</span></th>
          <th scope="col">Nama File</th>
          <th class="col-size" scope="col">Ukuran</th>
          <th class="col-date" scope="col">Tanggal Terdeteksi</th>
          <th scope="col">Folder Tujuan</th>
          <th class="col-action" scope="col"><span class="sr-only">Aksi</span></th>
        </tr>
      </thead>
      <tbody>
        {#if filtered.length === 0}
          <tr>
            <td colspan="6" class="empty-cell">Tidak ada file yang cocok dengan filter.</td>
          </tr>
        {:else}
          {#each filtered as r (r.source)}
            {@const selected = store.selectedRows.has(r.source)}
            <tr class:selected class:editing={editingSource === r.source}>
              <td class="col-check">
                <input
                  type="checkbox"
                  checked={selected}
                  aria-label={`Pilih ${r.name}`}
                  onchange={() => toggleRow(r.source)}
                />
              </td>
              <td class="cell-name">
                <span class="kind-icon" class:video={r.kind === 'video'} aria-hidden="true">
                  {#if r.kind === 'video'}
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                      <rect x="3" y="6" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1.8" />
                      <path d="m15 11 5-3v8l-5-3" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
                    </svg>
                  {:else}
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                      <rect x="3" y="4" width="18" height="16" rx="2" stroke="currentColor" stroke-width="1.8" />
                      <circle cx="8.5" cy="9.5" r="1.6" stroke="currentColor" stroke-width="1.6" />
                      <path d="m4 17 5-4.5 4 3.5 3-2.5 4 3.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                    </svg>
                  {/if}
                </span>
                <span class="name truncate" title={r.source}>{r.name}</span>
              </td>
              <td class="col-size mono-sm">{formatBytes(r.size_bytes)}</td>
              <td class="col-date">
                <span class="date-text">{dateOf(r)}</span>
                <span class="badge {badgeClass(r)}">{badgeLabel(r)}</span>
                {#if r.needs_review && !r.date_override}
                  <span class="review-flag" title="Tanggal perlu diperiksa">⚠</span>
                {/if}
              </td>
              <td class="cell-folder">
                <span class="mono-sm folder-text truncate" title={folderName(r)}>{folderName(r)}</span>
              </td>
              <td class="col-action">
                <div class="action-wrap">
                  <button
                    class="icon-btn"
                    type="button"
                    title="Edit tanggal"
                    aria-label={`Edit tanggal ${r.name}`}
                    aria-expanded={editingSource === r.source}
                    onclick={() => (editingSource === r.source ? closeEditor() : openEditor(r))}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                      <path
                        d="m4 20 4.2-1 10-10a2 2 0 0 0-2.8-2.8l-10 10L4 20Z"
                        stroke="currentColor"
                        stroke-width="1.7"
                        stroke-linejoin="round"
                      />
                      <path d="m13.5 7.5 3 3" stroke="currentColor" stroke-width="1.7" />
                    </svg>
                  </button>

                  {#if editingSource === r.source}
                    <div class="popover" role="dialog" aria-label="Edit tanggal file">
                      <div class="popover-title">Edit Tanggal</div>
                      <input class="input" type="date" bind:value={editDate} />

                      <div class="scope-options">
                        <label class="radio-row">
                          <input type="radio" name="scope" value="file" checked={applyScope === 'file'} onchange={() => (applyScope = 'file')} />
                          <span>File ini saja</span>
                        </label>
                        <label class="radio-row">
                          <input type="radio" name="scope" value="folder" checked={applyScope === 'folder'} onchange={() => (applyScope = 'folder')} />
                          <span>Semua file di folder ini</span>
                        </label>
                      </div>

                      <div class="popover-actions">
                        <button class="btn btn-ghost btn-sm" type="button" onclick={closeEditor}>Batal</button>
                        <button class="btn btn-primary btn-sm" type="button" disabled={!editDate} onclick={applyEdit}>Terapkan</button>
                      </div>
                    </div>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <footer class="table-footer">
    <span>{filtered.length} dari {rows.length} file</span>
    <span class="footer-dot">·</span>
    <span>{formatBytes(filtered.reduce((s, r) => s + r.size_bytes, 0))}</span>
  </footer>
</section>

<style>
  .preview {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: hidden;
  }

  .filter-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  .search-wrap {
    position: relative;
    flex: 1 1 200px;
    min-width: 160px;
  }

  .search-icon {
    position: absolute;
    left: 11px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-3);
    pointer-events: none;
  }

  .search-input {
    padding-left: 34px;
    height: 34px;
  }

  .segmented {
    display: inline-flex;
    padding: 2px;
    gap: 2px;
    background: #f1f5f9;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .seg {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 11px;
    border-radius: 5px;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease;
  }

  .seg:hover:not(.active) {
    color: var(--text-2);
  }

  .seg.active {
    background: var(--card);
    color: var(--text);
    box-shadow: var(--shadow-sm);
  }

  .mini-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 999px;
    background: var(--warning-bg);
    color: var(--warning);
    font-size: 10.5px;
    font-weight: 600;
  }

  .select-all-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-left: auto;
  }

  .select-all {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-2);
    cursor: pointer;
    white-space: nowrap;
  }

  .table-wrap {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .preview-table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    font-size: 13px;
  }

  .preview-table thead th {
    position: sticky;
    top: 0;
    z-index: 2;
    background: #f8fafc;
    text-align: left;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--text-3);
    padding: 9px 12px;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  .preview-table tbody td {
    padding: 9px 12px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }

  .preview-table tbody tr:hover {
    background: #f8fafc;
  }

  .preview-table tbody tr.selected {
    background: var(--primary-soft);
  }

  .preview-table tbody tr.editing {
    background: var(--primary-soft);
  }

  .col-check {
    width: 40px;
    text-align: center;
  }

  .col-size {
    width: 90px;
    white-space: nowrap;
    color: var(--text-2);
  }

  .col-date {
    width: 190px;
    white-space: nowrap;
  }

  .col-action {
    width: 60px;
    text-align: right;
  }

  .cell-name {
    display: flex;
    align-items: center;
    gap: 8px;
    max-width: 320px;
    min-width: 0;
  }

  .kind-icon {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border-radius: 6px;
    background: var(--primary-soft);
    color: var(--primary);
  }

  .kind-icon.video {
    background: #fef3c7;
    color: #b45309;
  }

  .name {
    color: var(--text);
    font-weight: 500;
  }

  .date-text {
    font-family: var(--font-mono);
    font-size: 12.5px;
    margin-right: 6px;
    color: var(--text-2);
  }

  .review-flag {
    margin-left: 4px;
    color: var(--warning);
    font-size: 12px;
  }

  .cell-folder {
    max-width: 260px;
  }

  .folder-text {
    display: block;
    color: var(--text-3);
  }

  .empty-cell {
    text-align: center;
    color: var(--text-3);
    padding: 32px 12px;
  }

  .action-wrap {
    position: relative;
    display: inline-flex;
    justify-content: flex-end;
  }

  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-3);
    transition: background 0.12s ease, color 0.12s ease;
  }

  .icon-btn:hover {
    background: var(--primary-soft);
    color: var(--primary);
  }

  .popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 20;
    width: 250px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow-md);
    text-align: left;
  }

  .popover-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
  }

  .scope-options {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .radio-row {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12.5px;
    color: var(--text-2);
    cursor: pointer;
  }

  .popover-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 2px;
  }

  .table-footer {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    font-size: 12px;
    color: var(--text-3);
    background: var(--card);
  }

  .footer-dot {
    opacity: 0.6;
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
