import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },

  // Vite dev server configuration for Tauri
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  // Tauri expects a fixed port, fail if that port is not available
  clearScreen: false,

  // Env variables
  envDir: ".",

  build: {
    outDir: "../dist",
    target: process.env.TAURI_CI ? "es2019" : undefined,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
        },
      },
    },
  },
}));
