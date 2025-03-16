import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  server: {
    host: "69.62.65.208",
    port: 80,
    allowedHosts: [
      "srbnfs.dino.icu",
      "srbnfs.antilimit.dev"
    ],
  }
})
