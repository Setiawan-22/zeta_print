import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const API_URL = process.env.VITE_API_URL || "http://localhost:8080";

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": {
        target: API_URL,
        changeOrigin: true,
      },
      "/health": {
        target: API_URL,
        changeOrigin: true,
      }
    }
  }
});
