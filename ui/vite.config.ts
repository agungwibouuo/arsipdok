import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Tauri menyajikan file dari custom protocol (tauri://localhost atau https://tauri.localhost).
  // Path asset HARUS relatif (./assets/...) agar ter-resolve ke folder dist yang di-bundle.
  base: './',

  // Vite options tailored for Tauri development
  //
  // 1. prevent Vite from obscuring Rust errors
  clearScreen: false,
  // 2. Tauri expects a fixed port; fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
})
