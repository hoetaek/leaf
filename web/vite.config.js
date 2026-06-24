import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Dev server proxies the API to a running `leaf serve` (default port 4173),
// so the browser stays same-origin and there is no CORS friction.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/api": "http://127.0.0.1:4173",
    },
  },
});
