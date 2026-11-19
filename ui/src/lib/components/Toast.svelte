<script lang="ts">
  interface Props {
    message: string;
    type?: 'info' | 'success' | 'warning' | 'error';
    onClose?: () => void;
  }

  let { message, type = 'info', onClose }: Props = $props();
</script>

<div class="toast {type}" role="status" aria-live="polite">
  <span class="icon" aria-hidden="true">
    {#if type === 'success'}
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <path d="m5 12.5 4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    {:else if type === 'error'}
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.8" />
        <path d="M12 7.5v5.5M12 16.4v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
    {:else if type === 'warning'}
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <path d="M12 4 2.8 19.5h18.4L12 4Z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
        <path d="M12 10v4M12 17v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
    {:else}
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.8" />
        <path d="M12 11v5.5M12 7.6v.1" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
      </svg>
    {/if}
  </span>

  <span class="message">{message}</span>

  {#if onClose}
    <button class="close" type="button" onclick={onClose} aria-label="Tutup notifikasi">✕</button>
  {/if}
</div>

<style>
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 16px;
    max-width: 380px;
    border-radius: 8px;
    border: 1px solid transparent;
    box-shadow: var(--shadow-md);
    font-size: 13px;
    line-height: 1.45;
    animation: slide-in 0.18s ease-out both;
    pointer-events: auto;
  }

  .icon {
    display: flex;
    align-items: center;
    flex: 0 0 auto;
    padding-top: 1px;
  }

  .message {
    flex: 1;
    min-width: 0;
    word-break: break-word;
  }

  .close {
    flex: 0 0 auto;
    padding: 0 2px;
    font-size: 12px;
    line-height: 18px;
    opacity: 0.6;
    border-radius: 4px;
  }

  .close:hover {
    opacity: 1;
  }

  .toast.info {
    background: var(--primary-soft);
    border-color: #c7d7fe;
    color: #1e40af;
  }

  .toast.success {
    background: var(--success-bg);
    border-color: #bbf7d0;
    color: #15803d;
  }

  .toast.warning {
    background: var(--warning-bg);
    border-color: var(--warning-border);
    color: var(--warning);
  }

  .toast.error {
    background: var(--danger-bg);
    border-color: #fecaca;
    color: var(--danger);
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(24px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast {
      animation: none;
    }
  }
</style>
