import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// GitHub Pages serves a project site from /<repository>/, so the asset base has to
// match the repository name. Change this when the site moves to its own domain.
const base = '/matlas/'

export default defineConfig({
  base,
  plugins: [react()],
  build: {
    // The wasm module is generated for modern engines only.
    target: 'esnext',
  },
})
