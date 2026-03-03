import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';
import path from 'node:path';

export default defineConfig({
  plugins: [svelte({ hot: false })],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
    globals: true,
    setupFiles: [],
  },
  resolve: {
    alias: {
      $lib: path.resolve('./src/lib'),
    },
  },
});
