<script lang="ts">
  import { store } from '../store.svelte';
  import logoUrl from '../../assets/logo.svg';

  interface Props {
    onStart?: () => void;
  }

  let { onStart }: Props = $props();

  function handleStart() {
    // Simpan bahwa sudah lihat welcome
    try {
      localStorage.setItem('arsipdok-welcomed', 'true');
    } catch {
      /* ignore */
    }
    store.appState = 'kosong';
    onStart?.();
  }

  function handleTutorial() {
    // TODO: buka tutorial (bisa via link eksternal atau dialog)
    console.log('Tutorial clicked');
  }
</script>

<div class="welcome-container">
  <!-- Background gradient + dekorasi -->
  <div class="welcome-bg">
    <div class="deco-circle deco-1"></div>
    <div class="deco-circle deco-2"></div>
  </div>

  <!-- Content -->
  <div class="welcome-content">
    <!-- Logo + Brand -->
    <div class="logo-section">
      <div class="logo-box">
        <img src={logoUrl} alt="ArsipDok" class="logo-icon" />
      </div>
      <div class="brand-name">
        <span class="arsip">Arsip</span><span class="dok">Dok</span>
      </div>
    </div>

    <!-- Tagline -->
    <p class="tagline">
      Sortir &amp; arsipkan dokumentasi foto/video secara otomatis<br />
      berdasarkan metadata tanggal — tanpa ribet.
    </p>

    <!-- Feature Cards -->
    <div class="features-grid">
      <div class="feature-card">
        <div class="feature-icon">🔍</div>
        <h3>Scan Cerdas</h3>
        <p class="feature-desc">
          Baca metadata EXIF, XMP, dan MP4 untuk menentukan tanggal foto/video yang tepat.
        </p>
      </div>

      <div class="feature-card featured">
        <div class="feature-icon">📁</div>
        <h3>Multi Tujuan</h3>
        <p class="feature-desc">
          Sortir ke banyak folder sekaligus dengan nama kegiatan kustom per tujuan.
        </p>
      </div>

      <div class="feature-card">
        <div class="feature-icon">✅</div>
        <h3>Aman &amp; Terverifikasi</h3>
        <p class="feature-desc">
          Verifikasi ukuran &amp; waktu file setelah transfer. Salin atau pindah — pilihan ada di tangan Anda.
        </p>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="action-buttons">
      <button class="btn-start" onclick={handleStart}>
        ▶ Mulai Sekarang
      </button>
      <button class="btn-tutorial" onclick={handleTutorial}>
        📖 Tutorial
      </button>
    </div>

    <!-- Footer -->
    <div class="welcome-footer">
      <p class="version">v{store.appVersion}</p>
      <p class="footer-text">
        Dibuat dengan ❤ untuk tim dokumentasi lapangan · Berjalan 100% offline
      </p>
    </div>
  </div>
</div>

<style>
  .welcome-container {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    background: var(--bg);
  }

  .welcome-bg {
    position: absolute;
    inset: 0;
    overflow: hidden;
  }

  .deco-circle {
    position: absolute;
    border-radius: 50%;
    opacity: 0.4;
    pointer-events: none;
  }

  .deco-1 {
    width: 600px;
    height: 600px;
    background: #DBEAFE;
    top: -100px;
    left: -100px;
  }

  .deco-2 {
    width: 500px;
    height: 500px;
    background: #BFDBFE;
    opacity: 0.3;
    bottom: 100px;
    right: -50px;
  }

  .welcome-content {
    position: relative;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 32px;
    padding: 60px 40px;
    max-width: 900px;
    margin: 0 auto;
  }

  .logo-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .logo-box {
    width: 200px;
    height: 200px;
    background: linear-gradient(135deg, #0F62FE 0%, #3B82F6 100%);
    border-radius: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 12px 32px rgba(15, 98, 254, 0.33);
    position: relative;
  }

  /* Folder icon illusion inside logo */
  .logo-box::before {
    content: '';
    position: absolute;
    width: 56px;
    height: 24px;
    background: rgba(255, 255, 255, 0.9);
    border-radius: 8px;
    top: 32px;
    left: 48px;
  }

  .logo-box::after {
    content: '';
    position: absolute;
    width: 120px;
    height: 88px;
    background: rgba(255, 255, 255, 0.95);
    border-radius: 10px;
    bottom: 20px;
    left: 40px;
  }

  .logo-icon {
    width: 80%;
    height: 80%;
    object-fit: contain;
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
    z-index: 1;
  }

  .brand-name {
    font-size: 48px;
    font-weight: 700;
    letter-spacing: -0.02em;
    line-height: 1;
  }

  .brand-name .arsip {
    color: var(--text);
  }

  .brand-name .dok {
    color: #0F62FE;
  }

  .tagline {
    text-align: center;
    font-size: 16px;
    line-height: 26px;
    color: var(--text-2);
    max-width: 600px;
    margin: 0;
  }

  .features-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 20px;
    width: 100%;
    max-width: 800px;
  }

  .feature-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 24px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
  }

  .feature-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 24px rgba(15, 23, 42, 0.12);
  }

  .feature-card.featured {
    background: #0F62FE;
    border-color: #0F62FE;
    color: white;
    box-shadow: 0 8px 24px rgba(15, 98, 254, 0.33);
  }

  .feature-card.featured h3 {
    color: white;
  }

  .feature-card.featured .feature-desc {
    color: #BFDBFE;
  }

  .feature-icon {
    font-size: 28px;
    line-height: 1;
  }

  .feature-card h3 {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
    color: var(--text);
  }

  .feature-desc {
    font-size: 12px;
    line-height: 18px;
    color: var(--text-3);
    margin: 0;
  }

  .action-buttons {
    display: flex;
    gap: 16px;
    justify-content: center;
    margin-top: 16px;
  }

  .btn-start,
  .btn-tutorial {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-family: inherit;
    font-size: 15px;
    font-weight: 600;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.12s ease;
    white-space: nowrap;
  }

  .btn-start {
    width: 200px;
    height: 48px;
    background: #0F62FE;
    color: white;
    box-shadow: 0 4px 12px rgba(15, 98, 254, 0.33);
  }

  .btn-start:hover {
    background: #0066CC;
    box-shadow: 0 6px 20px rgba(15, 98, 254, 0.4);
  }

  .btn-tutorial {
    width: 160px;
    height: 48px;
    background: var(--card);
    color: var(--text-2);
    border: 1.5px solid var(--border-strong);
  }

  .btn-tutorial:hover {
    background: var(--bg);
    border-color: var(--text-3);
    color: var(--text);
  }

  .welcome-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
  }

  .version {
    font-size: 12px;
    color: var(--text-3);
    margin: 0;
  }

  .footer-text {
    font-size: 11px;
    color: var(--text-3);
    text-align: center;
    max-width: 600px;
    margin: 0;
  }

  /* ── Responsif ──────────────────────────────────────── */
  @media (max-width: 768px) {
    .welcome-content {
      gap: 24px;
      padding: 40px 20px;
    }

    .logo-box {
      width: 160px;
      height: 160px;
    }

    .brand-name {
      font-size: 40px;
    }

    .tagline {
      font-size: 14px;
      line-height: 22px;
    }

    .features-grid {
      grid-template-columns: 1fr;
      gap: 16px;
    }

    .action-buttons {
      flex-direction: column;
      gap: 12px;
    }

    .btn-start,
    .btn-tutorial {
      width: 100%;
    }
  }

  @media (max-width: 480px) {
    .welcome-content {
      padding: 30px 16px;
      gap: 20px;
    }

    .logo-box {
      width: 140px;
      height: 140px;
    }

    .brand-name {
      font-size: 36px;
    }

    .deco-1 {
      width: 400px;
      height: 400px;
    }

    .deco-2 {
      width: 300px;
      height: 300px;
    }
  }
</style>
