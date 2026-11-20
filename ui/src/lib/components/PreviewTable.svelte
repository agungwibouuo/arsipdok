<script lang="ts">
  import { store, formatBytes, sourceLabel } from '../store.svelte';
  import type { PreviewRow, FileKind } from '../types';

  interface Props {
    onEditDate?: (row: PreviewRow, applyScope: boolean) => void;
  }

  let { onEditDate }: Props = $props();

  let search = $state('');
  let kindFilter = $state<'all' | FileKind>('all');

  // Editor popover state
  let editingSource = $state<string | null>(null);
  let editDate = $state('');
  let applyScope = $state<'file' | 'folder'>('file');

  const rows = $derived(store.previewRows);

  const filtered = $derived(
    rows.filter((r) => {
      if (kindFilter !== 'all' && r.kind !== kindFilter) return false;
      const q = search.trim().toLowerCase();
      if (q && !r.name.toLowerCase().includes(q)) return false;
      return true;
    })
  );

  const allSelected = $derived(filtered.length > 0 && filtered.every((r) => store.selectedRows.has(r.source)));
  const someSelected = $derived(filtered.some((r) => store.selectedRows.has(r.source)));

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

  function formatDisplayDate(dateStr: string): string {
    // Input format: YYYY-MM-DD
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
    const d = dateOf(r);
    const dest = store.settings.destinations.find((x) => x.enabled);
    const activity = (dest?.activity || '').trim() || 'Nama Kegiatan';
    const suffix = (dest?.suffix ?? '').trim();
    // In FigCraft: "03-15_Pelatihan Petani" (MM-DD_Activity)
    const parts = d.split('-');
    const mmDd = parts.length === 3 ? `${parts[1]}-${parts[2]}` : d;
    return `${mmDd}_${activity}${suffix ? `_${suffix}` : ''}`;
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
</script>

<section class="preview-panel" aria-label="Preview File dan Pengelompokan">
  <div class="panel-head">
    <h2 class="step-title">3&nbsp; Preview File &amp; Pengelompokan</h2>
  </div>

  {#if rows.length === 0}
    <div class="empty-placeholder">
      <span class="empty-icon" aria-hidden="true">📂</span>
      <span class="empty-title">Belum ada file untuk ditampilkan</span>
      <span class="empty-desc">Tambahkan sumber dokumentasi — metadata dipindai otomatis.</span>
    </div>
  {:else}
    <div class="table-container">
      <div class="table-header">
        <label class="th-check">
          <input
            type="checkbox"
            checked={allSelected}
            indeterminate={someSelected && !allSelected}
            onchange={toggleAll}
          />
        </label>
        <span class="th-name">Nama File</span>
        <span class="th-tipe">Tipe</span>
        <span class="th-ukuran">Ukuran</span>
        <span class="th-tanggal">Tanggal</span>
        <span class="th-sumber">Sumber</span>
        <span class="th-tujuan">Folder Tujuan</span>
      </div>

      <div class="table-body">
        {#each filtered as r (r.source)}
          {@const selected = store.selectedRows.has(r.source)}
          <div class="table-row" class:selected>
            <label class="cell-check">
              <input
                type="checkbox"
                checked={selected}
                aria-label={`Pilih ${r.name}`}
                onchange={() => toggleRow(r.source)}
              />
            </label>
            <span class="cell-name truncate" title={r.name}>{r.name}</span>
            <span class="cell-tipe">{r.kind === 'video' ? '🎬' : '📷'}</span>
            <span class="cell-ukuran">{formatBytes(r.size_bytes)}</span>
            <button class="cell-tanggal" type="button" onclick={() => openEditor(r)} title="Klik untuk ubah tanggal">
              {formatDisplayDate(dateOf(r))}
            </button>
            <span class="cell-sumber badge-{r.metadata_source}">{sourceLabel(r.metadata_source)}</span>
            <span class="cell-tujuan truncate" title={folderName(r)}>{folderName(r)}</span>

            {#if editingSource === r.source}
              <div class="popover">
                <div class="pop-title">Edit Tanggal</div>
                <input class="pop-date-input" type="date" bind:value={editDate} />
                <div class="pop-scope">
                  <label>
                    <input type="radio" name="scope" value="file" checked={applyScope === 'file'} onchange={() => (applyScope = 'file')} />
                    <span>File ini saja</span>
                  </label>
                  <label>
                    <input type="radio" name="scope" value="folder" checked={applyScope === 'folder'} onchange={() => (applyScope = 'folder')} />
                    <span>Semua file di folder ini</span>
                  </label>
                </div>
                <div class="pop-actions">
                  <button class="pop-btn" type="button" onclick={closeEditor}>Batal</button>
                  <button class="pop-btn pop-btn-primary" type="button" disabled={!editDate} onclick={applyEdit}>Terapkan</button>
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</section>

<style>
  .preview-panel {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
    box-sizing: border-box;
    overflow: hidden;
    container-type: inline-size;
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

  /* Empty state inside panel */
  .empty-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--text-3);
  }

  .empty-icon {
    font-size: 46px;
    color: var(--border-strong);
    line-height: 1;
    margin-bottom: 2px;
  }

  .empty-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-3);
  }

  .empty-desc {
    font-size: 12px;
    color: var(--text-3);
  }

  /* Table Grid */
  .table-container {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .table-header,
  .table-row {
    display: grid;
    grid-template-columns: 32px minmax(140px, 3.5fr) 48px 90px 124px 72px minmax(140px, 4fr);
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 10px;
    box-sizing: border-box;
  }

  /* Panel sempit (2-side app / jendela kecil): sembunyikan kolom minor */
  @container (max-width: 720px) {
    .table-header,
    .table-row {
      grid-template-columns: 32px minmax(0, 1fr) 64px 96px minmax(0, 1fr);
    }

    .th-tipe,
    .th-sumber,
    .cell-tipe,
    .cell-sumber {
      display: none;
    }
  }

  @container (max-width: 460px) {
    .table-header,
    .table-row {
      grid-template-columns: 32px minmax(0, 1fr) minmax(0, 1fr);
      gap: 6px;
      padding: 0 8px;
    }

    .th-ukuran,
    .cell-ukuran,
    .th-tanggal,
    .cell-tanggal {
      display: none;
    }
  }

  .table-header {
    background: var(--bg);
    border-radius: 4px;
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
    border-bottom: 1px solid var(--bg);
    background: var(--card);
    font-size: 12px;
    color: var(--text);
    position: relative;
  }

  .table-row:hover {
    background: var(--bg);
  }

  .table-row.selected {
    background: var(--primary-soft);
  }

  .th-check,
  .cell-check {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .th-check input,
  .cell-check input {
    cursor: pointer;
  }

  .cell-name {
    font-weight: 400;
    color: var(--text);
  }

  .cell-tipe {
    color: var(--text-2);
    font-size: 13px;
  }

  .cell-ukuran {
    color: var(--text-2);
    font-family: inherit;
  }

  .cell-tanggal {
    background: transparent;
    border: none;
    padding: 0;
    font-family: inherit;
    font-size: 12px;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    text-decoration: underline dotted var(--border-strong);
  }

  .cell-tanggal:hover {
    color: #0066CC;
    text-decoration-color: #0066CC;
  }

  .cell-sumber {
    font-size: 11px;
    font-weight: 500;
    color: #16A34A;
  }

  .cell-sumber.badge-file_modified,
  .cell-sumber.badge-file_created {
    color: var(--text-3);
  }

  .cell-tujuan {
    color: #0066CC;
  }

  .popover {
    position: absolute;
    top: 32px;
    left: 200px;
    background: var(--card);
    border: 1px solid var(--border-strong);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    padding: 10px 12px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 230px;
  }

  .pop-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-2);
  }

  .pop-date-input {
    height: 26px;
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 0 6px;
    font-size: 11px;
    font-family: inherit;
  }

  .pop-scope {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 11px;
    color: var(--text-2);
  }

  .pop-scope label {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }

  .pop-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .pop-btn {
    height: 24px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    background: var(--bg);
    font-size: 10px;
    cursor: pointer;
    font-family: inherit;
  }

  .pop-btn-primary {
    background: #0F62FE;
    color: #ffffff;
    border-color: #0F62FE;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
</style>
