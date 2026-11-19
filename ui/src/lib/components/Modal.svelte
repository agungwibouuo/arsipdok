<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    subtitle?: string;
    width?: number;
    open: boolean;
    onClose: () => void;
    children?: Snippet;
    footer?: Snippet;
  }

  let { title, subtitle, width = 560, open, onClose, children, footer }: Props = $props();

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onClose();
    }
  }
</script>

<svelte:window on:keydown={onKeydown} />

{#if open}
  <div class="backdrop" onclick={onClose} role="presentation"></div>
  <div class="modal-root" role="dialog" aria-modal="true" aria-label={title}>
    <div class="modal" style={`width: ${width}px`}>
      <header class="modal-header">
        <div class="headings">
          <h1>{title}</h1>
          {#if subtitle}<p class="sub">{subtitle}</p>{/if}
        </div>
        <button class="btn btn-ghost btn-sm close" type="button" onclick={onClose} aria-label="Tutup">✕</button>
      </header>
      <div class="modal-body">
        {@render children?.()}
      </div>
      {#if footer}
        <footer class="modal-footer">
          {@render footer()}
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.45);
    z-index: 100;
    backdrop-filter: blur(2px);
  }

  .modal-root {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 101;
    pointer-events: none;
    padding: 24px;
  }

  .modal {
    pointer-events: auto;
    background: var(--card);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    max-height: 90vh;
    max-width: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    padding: 20px 24px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    flex: 0 0 auto;
  }

  .headings {
    min-width: 0;
  }

  .sub {
    font-size: 13px;
    color: var(--text-3);
    margin-top: 2px;
  }

  .close {
    flex: 0 0 auto;
    width: 30px;
    padding: 0;
    font-size: 13px;
    color: var(--text-3);
  }

  .modal-body {
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .modal-footer {
    padding: 14px 24px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex: 0 0 auto;
  }
</style>
