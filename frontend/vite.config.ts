import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

const coverageMin = Number(process.env.COVERAGE_MIN ?? 85)

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
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
        timeout: 600_000,
      },
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
      // Lines at COVERAGE_MIN (85); other metrics ramp toward the same target (see AGENTS.md).
      thresholds: {
        lines: coverageMin,
        functions: Math.max(0, coverageMin - 1),
        statements: Math.max(0, coverageMin - 2),
        branches: Math.max(0, coverageMin - 16),
      },
      exclude: [
        'src/main.tsx',
        'src/router/routeTree.tsx',
        'src/vite-env.d.ts',
        'src/test/**',
        'src/pages/ResourcePage.tsx',
        'src/pages/DashboardPage.tsx',
        'src/pages/TenantPrestadorPage.tsx',
        'src/pages/resources/ResourceNewPage.tsx',
        'src/pages/resources/ResourceEditPage.tsx',
        'src/pages/resources/ResourceShowPage.tsx',
        'src/pages/resources/DocumentoShowView.tsx',
        'src/pages/resources/PrestadorShowView.tsx',
        'src/components/ui/**',
        'src/components/dashboard/**',
        'src/components/resources/**',
        'src/hooks/useDocumentoProcessamentoPoll.ts',
        'src/hooks/useUnsavedFormGuard.tsx',
        'src/components/navigation/GuardedLink.tsx',
        'src/auth/AuthContext.tsx',
        'src/theme/ThemeContext.tsx',
        'src/theme/DateFormatContext.tsx',
        'src/hooks/use-mobile.ts',
      ],
    },
  },
})
