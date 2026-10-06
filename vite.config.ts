import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The port is fixed because tauri.conf.json names it as the dev address.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true, host: "127.0.0.1" },
  build: { target: "safari15", outDir: "dist" }
});
