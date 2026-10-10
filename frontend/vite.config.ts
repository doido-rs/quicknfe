import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3001,
    proxy: {
      '/api': 'http://localhost:3000',
      '/users': 'http://localhost:3000',
      '/tenant': 'http://localhost:3000',
      '/api-docs': 'http://localhost:3000',
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
      exclude: [
        'src/main.tsx',
        'src/router/routeTree.tsx',
        'src/vite-env.d.ts',
        'src/test/**',
        'src/pages/ResourcePage.tsx',
        'src/pages/resources/ResourceNewPage.tsx',
        'src/pages/resources/ResourceEditPage.tsx',
        'src/components/ui/**',
        'src/components/resources/**',
        'src/hooks/use-mobile.ts',
      ],
    },
  },
})
