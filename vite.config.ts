import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const binpatchWasmPath = fileURLToPath(
  new URL("./crates/binpatch-wasm/pkg/binpatch_wasm.js", import.meta.url),
);

export default defineConfig({
  base: "./",
  plugins: [react()],
  resolve: {
    alias: {
      "binpatch-wasm": binpatchWasmPath,
    },
  },
  server: {
    host: "127.0.0.1",
    port: 65173,
    strictPort: true,
  },
  preview: {
    host: "127.0.0.1",
    port: 65175,
    strictPort: true,
  },
});
