import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  resolve: {
    alias: {
      '$app/navigation': path.resolve('./src/test/__mocks__/navigation.ts'),
      '$app/environment': path.resolve('./src/test/__mocks__/environment.ts'),
      '$app/stores': path.resolve('./src/test/__mocks__/stores.ts'),
    }
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'lcov'],
      reportsDirectory: './coverage',
      include: ['src/lib/**/*.ts'],
      exclude: ['src/test/**']
    }
  }
});
