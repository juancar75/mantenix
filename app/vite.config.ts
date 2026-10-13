// Configuración de Vite para Tauri: puerto fijo y sin limpiar la consola (la usa `tauri dev`).
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
