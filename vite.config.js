import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { join } from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  resolve: {
    alias: {
      '@lib': join(__dirname, 'src/lib'),
      '@compose': join(__dirname, 'src/routes/compose'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    hmr: {
      protocol: 'ws',
      host: 'localhost',
      port: 5173,
    },
    fs: {
      allow: ['.'],
    },
  },
  build: {
    outDir: join(__dirname, 'dist'),
    emptyOutDir: true,
  },
});