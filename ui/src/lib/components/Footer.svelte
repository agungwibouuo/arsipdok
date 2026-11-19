<script lang="ts">
  interface Props {
    leftText?: string;
    rightText?: string;
    busy?: boolean;
  }

  let { leftText = 'Siap', rightText = 'v0.1.0', busy = false }: Props = $props();
</script>

<footer class="footer">
  <div class="left">
    <span class="dot" class:busy aria-hidden="true"></span>
    <span class="label">{leftText}</span>
  </div>
  <div class="right">{rightText}</div>
</footer>

<style>
  .footer {
    height: 32px;
    flex: 0 0 auto;
    background: #f8fafc;
    border-top: 1px solid var(--border);
    padding: 0 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    font-size: 12px;
    color: var(--text-3);
    user-select: none;
  }

  .left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dot {
    width: 8px;
    height: 8px;
    flex: 0 0 auto;
    border-radius: 50%;
    background: var(--success);
  }

  .dot.busy {
    background: var(--primary);
    animation: pulse 1.1s ease-in-out infinite;
  }

  .right {
    font-family: var(--font-mono);
    font-size: 11px;
    white-space: nowrap;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
      box-shadow: 0 0 0 0 rgba(15, 98, 254, 0.35);
    }
    50% {
      opacity: 0.6;
      transform: scale(0.85);
      box-shadow: 0 0 0 4px rgba(15, 98, 254, 0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .dot.busy {
      animation: none;
      opacity: 0.7;
    }
  }
</style>
