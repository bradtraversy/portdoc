import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      // npm run dev talks to a locally running `cargo run` backend
      '/api': 'http://127.0.0.1:7788',
    },
  },
})
