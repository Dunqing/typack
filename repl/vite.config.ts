import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/typack/",
  plugins: [vue()],
  build: {
    target: "esnext",
  },
  optimizeDeps: {
    exclude: ["@napi-rs/wasm-runtime"],
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
});
