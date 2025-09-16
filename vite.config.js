import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { join } from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: true,
    hmr: {
      protocol: 'ws',
      host: 'localhost',
      port: 5173
    }
  },
  build: {
    outDir: join(__dirname, 'dist'),
    emptyOutDir: true
  }
});