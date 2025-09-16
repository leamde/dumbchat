import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';

export default defineConfig({
  plugins: [sveltekit()],
  build: {
    outDir: 'dist',
    target: 'esnext'
  },
  server: {
    port: 5173,
    strictPort: true,
    fs: {
      allow: ['..']
    }
  },
  resolve: {
    alias: {
      '@sveltejs/kit': '@sveltejs/kit',
      '@sveltejs/kit/vite': '@sveltejs/kit/vite'
    }
  },
  optimizeDeps: {
    exclude: ['@sveltejs/kit']
  }
});