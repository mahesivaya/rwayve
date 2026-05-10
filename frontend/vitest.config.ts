import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    // The repo currently has stale compiled `.js` siblings next to every
    // `.tsx`. Without this override Vite's default extension order would
    // load the stale `.js` file for transitive imports, e.g.
    // `import { AuthProvider } from "./AuthContext"`.
    extensions: [".mjs", ".tsx", ".ts", ".jsx", ".js", ".json"],
  },
  test: {
    environment: "jsdom",
    pool: "forks",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
    css: false,
  },
});
