import { resolve } from "node:path";
import { defineConfig } from "vite";

export default defineConfig({
  server: { allowedHosts: [".conductor.show"] },
  preview: { allowedHosts: [".conductor.show"] },
  build: {
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        docs: resolve(import.meta.dirname, "docs.html"),
      },
    },
  },
});
