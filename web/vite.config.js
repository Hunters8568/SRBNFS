import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    host: "127.0.0.1",
    port: 5500,
    allowedHosts: [
      "srbnfs.dino.icu",
      "srbnfs.antilimit.dev"
    ],
  }
})
