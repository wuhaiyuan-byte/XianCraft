import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // Proxy requests from /ws to our Rust MUD server
      '/ws': {
        target: 'ws://localhost:3000',
        ws: true, // IMPORTANT: enable websocket proxy
      },
    }
  }
})
