import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: '.',
  build: {
    outDir: 'dist',
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
      },
    },
  },
  server: {
    port: 3000,
    host: true,
  },
  optimizeDeps: {
    exclude: ['@your-org/rust-wasm-package'], // Replace with your actual WASM package name
  },
  plugins: [],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  // WASM support
  assetsInclude: ['**/*.wasm'],
  // Enable WASM streaming
  experimental: {
    asyncWebAssembly: true,
  },
});
