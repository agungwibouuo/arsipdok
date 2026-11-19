import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

try {
  const app = mount(App, {
    target: document.getElementById('app')!,
  })
  // @ts-expect-error debug handle
  window.__app = app
} catch (err) {
  console.error('[ArsipDok] Gagal mount aplikasi:', err)
  const target = document.getElementById('app')
  if (target) {
    const stack = (err as Error)?.stack ?? String(err)
    target.innerHTML = `<pre style="padding:24px;font-family:Consolas,monospace;font-size:12px;color:#7F1D1D;background:#FEF2F2;white-space:pre-wrap">${stack
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')}</pre>`
  }
}
