import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vitest/config'

// Tauri desktop expects a fixed dev-server port (see src-tauri/tauri.conf.json).
const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Prevent vite from obscuring rust errors.
  clearScreen: false,
  server: {
    // Make sure this matches `devUrl` in src-tauri/tauri.conf.json.
    port: 5173,
    // Tauri expects a fixed port — fail instead of silently incrementing.
    strictPort: true,
    // Set by the Tauri CLI when running on a physical device.
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: {
      // Ignore the desktop shell: it has its own cargo watcher.
      ignored: ['**/src-tauri/**'],
    },
  },
  // Variables starting with these prefixes are exposed to the frontend.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS/Linux.
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // Debug builds stay readable with sourcemaps.
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: 'happy-dom',
  },
})
