import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const isDebug = !!process.env.TAURI_ENV_DEBUG;
const minify: "esbuild" | false = isDebug ? false : "esbuild";

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "chrome105",
    minify,
    sourcemap: isDebug,
  },
});
