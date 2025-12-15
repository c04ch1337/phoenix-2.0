import path from 'path';
import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
    // Load env from repo root so `VITE_*` can live alongside the Rust `.env`.
    const repoRoot = path.resolve(__dirname, '..');
    const env = loadEnv(mode, repoRoot, '');
    return {
      server: {
        port: 3000,
        host: '0.0.0.0',
        proxy: {
          // Local dev: proxy API calls to the Rust backend.
          '/api': {
            target: env.VITE_PHOENIX_API_BASE || 'http://127.0.0.1:8888',
            changeOrigin: true,
          },
          '/health': {
            target: env.VITE_PHOENIX_API_BASE || 'http://127.0.0.1:8888',
            changeOrigin: true,
          },
        },
      },
      plugins: [react()],
      resolve: {
        alias: {
          '@': path.resolve(__dirname, '.'),
        }
      }
    };
});
