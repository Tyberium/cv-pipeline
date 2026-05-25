import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react()],
  server: {
    // Proxy API calls to Axum during development so the frontend
    // makes real fetch calls without CORS issues
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
  build: {
    // Output to ../static so Axum serves it from ./static at runtime
    outDir: '../static',
    emptyOutDir: true,
  },
});
