import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  base: "/",
  plugins: [vue()],
  build: {
    target: "esnext",
  },
  optimizeDeps: {
    exclude: ["@napi-rs/wasm-runtime", "@emnapi/core", "@emnapi/runtime", "@tybys/wasm-util"],
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
});
