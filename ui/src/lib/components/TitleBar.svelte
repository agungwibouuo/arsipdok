<script lang="ts">
  import { store } from '../store.svelte';
  import logoUrl from '../../assets/logo.svg';

  interface Props {
    onSettings?: () => void;
  }

  let { onSettings }: Props = $props();

  type ThemePref = 'light' | 'dark' | 'system';

  // Sistem Windows saat ini (reaktif)
  let systemDark = $state(false);

  function resolveTheme(pref: ThemePref): 'light' | 'dark' {
    if (pref === 'system') return systemDark ? 'dark' : 'light';
    return pref;
  }

  function applyTheme(pref: ThemePref) {
    const root = document.documentElement;
    root.setAttribute('data-theme', resolveTheme(pref));
    root.setAttribute('data-theme-pref', pref);
    try {
      localStorage.setItem('arsipdok-theme', pref);
    } catch {
      /* ignore */
    }
  }

  // Inisialisasi: baca preferensi tersimpan + pantau tema sistem
  $effect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    systemDark = mq.matches;

    const saved = (localStorage.getItem('arsipdok-theme') as ThemePref | null) ?? 'system';
    store.theme = saved;
    applyTheme(saved);

    const onChange = (e: MediaQueryListEvent) => {
      systemDark = e.matches;
      // hanya ikut berubah otomatis bila preferensi = sistem
      if (store.theme === 'system') applyTheme('system');
    };
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  });

  // Terapkan ulang setiap kali preferensi berubah lewat tombol
  let lastApplied: ThemePref | null = null;
  $effect(() => {
    const pref = store.theme;
    if (pref !== lastApplied) {
      lastApplied = pref;
      applyTheme(pref);
    }
  });

  const cycle: Record<ThemePref, ThemePref> = {
    light: 'dark',
    dark: 'system',
    system: 'light',
  };

  function cycleTheme() {
    store.theme = cycle[store.theme];
  }

  const isDark = $derived(resolveTheme(store.theme) === 'dark');

  const themeLabel = $derived(
    store.theme === 'system'
      ? `Tema: Ikut sistem (${systemDark ? 'gelap' : 'terang'})`
      : store.theme === 'dark'
        ? 'Tema: Gelap'
        : 'Tema: Terang'
  );
</script>

<!-- Top Action Bar dengan Logo Resmi ArsipDok -->
<header class="action-bar" data-tauri-drag-region>
  <div class="brand-group" data-tauri-drag-region>
    <!-- Logo resmi ArsipDok (aset dari design/ArsipDok.figcraft #1130) -->
    <div class="logo-box" aria-hidden="true">
      <img class="logo-icon" src={logoUrl} alt="" draggable="false" />
    </div>

    <div class="brand-text">
      <div class="brand-row">
        <span class="brand">ArsipDok</span>
        <span class="version-tag">v{store.appVersion}</span>
      </div>
      <span class="tagline">Sortir &amp; Pemindahan Otomatis Berbasis Metadata</span>
    </div>
  </div>

  <div class="action-right">
    <button
      class="theme-btn"
      class:system={store.theme === 'system'}
      type="button"
      onclick={cycleTheme}
      title="{themeLabel} — klik untuk berganti"
      aria-label={themeLabel}
    >
      {#if store.theme === 'system'}
        <!-- Ikon monitor untuk mode ikut sistem -->
        <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2.5" y="4" width="19" height="13" rx="2" />
          <path d="M8 21h8M12 17v4" />
          {#if systemDark}
            <path d="M12 13.5a3 3 0 0 0 3-3c0-.4.5-.6.8-.3a4 4 0 1 1-4.5-4.5c.3-.3.7.1.7.5a3 3 0 0 0 0 7.3Z" fill="currentColor" stroke="none" />
          {:else}
            <circle cx="12" cy="10.5" r="2.6" />
            <path d="M12 6v.6M12 14.4v.6M7.5 10.5H7M18 10.5h-.5M8.8 7.3l-.4-.4M15.6 13.7l-.4-.4M15.6 7.3l.4-.4M8.8 13.7l.4-.4" />
          {/if}
        </svg>
      {:else if isDark}
        <!-- Ikon bulan -->
        <svg viewBox="0 0 24 24" width="17" height="17" fill="currentColor">
          <path d="M20.6 14.3A8.6 8.6 0 0 1 9.7 3.4a.7.7 0 0 0-.9-.9 9.8 9.8 0 1 0 12.7 12.7.7.7 0 0 0-.9-.9Z" />
        </svg>
      {:else}
        <!-- Ikon matahari -->
        <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round">
          <circle cx="12" cy="12" r="4" fill="currentColor" stroke="none" />
          <path d="M12 2.5v2M12 19.5v2M4.6 4.6l1.4 1.4M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4 6 18M18 6l1.4-1.4" />
        </svg>
      {/if}
    </button>

    <button class="settings-btn" type="button" onclick={onSettings} aria-label="Buka Pengaturan">
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" />
      </svg>
      <span>Pengaturan</span>
    </button>
  </div>
</header>

<style>
  .action-bar {
    height: 60px;
    background: var(--card);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    flex: 0 0 auto;
    user-select: none;
  }

  .action-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 0 0 auto;
  }

  .theme-btn {
    width: 34px;
    height: 34px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
    color: var(--text-2);
    line-height: 1;
    flex-shrink: 0;
  }

  .theme-btn:hover {
    background: var(--border);
    color: var(--text);
  }

  .theme-btn.system {
    position: relative;
  }

  .theme-btn.system::after {
    content: '';
    position: absolute;
    right: 4px;
    bottom: 4px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--primary);
    border: 1.5px solid var(--card);
  }

  .brand-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-box {
    width: 42px;
    height: 42px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .logo-icon {
    width: 100%;
    height: 100%;
    display: block;
    filter: drop-shadow(0 2px 4px rgba(15, 98, 254, 0.2));
    user-select: none;
    pointer-events: none;
  }

  .brand-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .brand-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .brand {
    font-size: 20px;
    font-weight: 700;
    color: var(--text);
    line-height: 1;
    letter-spacing: -0.01em;
  }

  .version-tag {
    font-size: 11px;
    font-weight: 600;
    color: #0F62FE;
    background: var(--primary-soft);
    padding: 1px 6px;
    border-radius: 4px;
    line-height: 1.2;
  }

  .tagline {
    font-size: 11px;
    font-weight: 400;
    color: var(--text-3);
    line-height: 1;
  }

  .settings-btn {
    height: 34px;
    padding: 0 14px;
    background: var(--bg);
    border: 1px solid var(--border-strong);
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-2);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    font-family: inherit;
    transition: all 0.12s ease;
  }

  .settings-btn:hover {
    background: var(--border);
    color: var(--text);
  }
</style>