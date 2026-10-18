import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: process.env.ARENA_API_TARGET ?? 'http://127.0.0.1:3000',
        ws: true,
      },
    },
  },
});
