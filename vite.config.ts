// SPDX-License-Identifier: GPL-3.0-or-later
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri, geliştirme sunucusunu sabit bir portta bekler.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || '0.0.0.0',
    watch: { ignored: ['**/src-tauri/**'] },
    // Önizleme tünelleri kendi ana makine adını kullanır.
    allowedHosts: process.env.VITE_ADDITIONAL_ALLOWED_HOSTS
      ? process.env.VITE_ADDITIONAL_ALLOWED_HOSTS.split(',').map((h) => h.trim())
      : undefined,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Android WebView (Chromium) ve min SDK 26 hedefi.
    target: process.env.TAURI_ENV_PLATFORM === 'android' ? 'chrome105' : 'es2022',
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    chunkSizeWarningLimit: 1500,
  },
});
