import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
      },
      '/auth': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
        changeOrigin: true,
      }
    }
  },
  build: {
    // Enable code splitting and chunk optimization
    rollupOptions: {
      output: {
        // Manual chunk splitting for better caching
        manualChunks: {
          // Core React libraries
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          
          // Data visualization libraries
          'viz-vendor': ['d3', 'cytoscape', 'cytoscape-dagre', 'cytoscape-cose-bilkent', 'recharts'],
          
          // UI and form libraries
          'ui-vendor': ['framer-motion', 'lucide-react', 'react-hook-form', 'react-select', 'react-datepicker'],
          
          // State management and data fetching
          'state-vendor': ['@tanstack/react-query', 'zustand', 'axios', 'socket.io-client'],
          
          // Large utility libraries
          'utils-vendor': ['react-virtualized']
        },
        // Optimize chunk file names
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId
          if (facadeModuleId) {
            // Create meaningful chunk names based on the module path
            if (facadeModuleId.includes('components/dashboard')) return 'chunks/dashboard-[hash].js'
            if (facadeModuleId.includes('components/explorer')) return 'chunks/explorer-[hash].js'
            if (facadeModuleId.includes('components/traceability')) return 'chunks/traceability-[hash].js'
            if (facadeModuleId.includes('components/analytics')) return 'chunks/analytics-[hash].js'
            if (facadeModuleId.includes('features/')) return 'chunks/features-[hash].js'
          }
          return 'chunks/[name]-[hash].js'
        },
        // Optimize asset file names
        assetFileNames: 'assets/[name]-[hash].[ext]'
      }
    },
    // Optimize chunk size warnings
    chunkSizeWarningLimit: 1000,
    // Enable source maps for production debugging
    sourcemap: true,
    // Optimize minification
    minify: 'esbuild',
  },
  // Optimize dependencies
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'react-router-dom',
      'axios',
      'socket.io-client',
      '@tanstack/react-query',
      'zustand'
    ],
    exclude: [
      // Exclude large libraries that should be loaded on demand
      'd3',
      'cytoscape',
      'recharts'
    ]
  },
  // Performance optimizations
  esbuild: {
    // Remove unused imports
    treeShaking: true,
  }
})
