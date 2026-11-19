<script lang="ts">
  interface Props {
    onSettings?: () => void;
    onHelp?: () => void;
  }

  let { onSettings, onHelp }: Props = $props();
</script>

<header class="titlebar">
  <div class="brand">
    <div class="logo" aria-hidden="true">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
        <path
          d="M3 7.5A1.5 1.5 0 0 1 4.5 6h4.379a1.5 1.5 0 0 1 1.06.44l1.622 1.621a1.5 1.5 0 0 0 1.06.439H19.5A1.5 1.5 0 0 1 21 10v7.5A1.5 1.5 0 0 1 19.5 19h-15A1.5 1.5 0 0 1 3 17.5v-10Z"
          fill="#fff"
        />
      </svg>
    </div>
    <div class="brand-text">
      <span class="app-name">ArsipDok</span>
      <span class="tagline">Pencatatan Dokumentasi Foto &amp; Video</span>
    </div>
  </div>

  <div class="actions">
    <button class="btn btn-ghost btn-sm" type="button" onclick={onSettings}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"
          stroke="currentColor"
          stroke-width="1.6"
        />
        <path
          d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <span>Pengaturan</span>
    </button>

    <button class="btn btn-ghost btn-sm" type="button" onclick={onHelp}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.6" />
        <path
          d="M9.6 9.2a2.5 2.5 0 1 1 3.7 2.2c-.8.5-1.3 1-1.3 1.9v.3"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        />
        <circle cx="12" cy="16.8" r="0.95" fill="currentColor" />
      </svg>
      <span>Bantuan</span>
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: 56px;
    flex: 0 0 auto;
    background: var(--card);
    border-bottom: 1px solid var(--border);
    padding: 0 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .logo {
    width: 32px;
    height: 32px;
    flex: 0 0 auto;
    border-radius: 8px;
    background: var(--primary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .brand-text {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 0;
    line-height: 1.25;
  }

  .app-name {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
  }

  .tagline {
    font-size: 12px;
    color: var(--text-3);
    white-space: nowrap;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .actions .btn-ghost {
    color: var(--text-2);
  }
</style>
