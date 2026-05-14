import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { viteSingleFile } from 'vite-plugin-singlefile'

export default defineConfig({
  plugins: [svelte(), viteSingleFile()],
  server: {
    proxy: {
      '/api': 'http://localhost:8089'
    }
  },
  build: {
    outDir: '../static',
    emptyOutDir: true,
    assetsInlineLimit: Infinity,
  },
})
