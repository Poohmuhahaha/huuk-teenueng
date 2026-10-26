import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vitest/config'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_'],
  build: {
    target: 'es2022',
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules/zod')) return 'validation'
          if (
            id.includes('node_modules/vue') ||
            id.includes('node_modules/@tanstack/vue-query') ||
            id.includes('node_modules/@tanstack/vue-table')
          ) {
            return 'vendor'
          }
          if (id.includes('node_modules/motion-v')) {
            return 'motion'
          }
        },
      },
    },
  },
  test: {
    environment: 'happy-dom',
  },
})
