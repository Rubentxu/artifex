import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  base: './',
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  define: {
    __ARTIFEX_E2E__: JSON.stringify(!!process.env.ARTIFEX_E2E),
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: false,
  },
});
