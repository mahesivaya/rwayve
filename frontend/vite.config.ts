import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  server: {
    // Pre-transform the modules every page hits so the first browser
    // request doesn't cascade through them serially.
    warmup: {
      clientFiles: [
        "./src/main.tsx",
        "./src/App.tsx",
        "./src/auth/AuthContext.tsx",
        "./src/auth/Login.tsx",
        "./src/components/Layout.tsx",
        "./src/components/Header.tsx",
        "./src/components/ProtectedRoute.tsx",
      ],
    },

    proxy: {
      "/api": "http://localhost:8080",
      "/gmail": "http://localhost:8080",
      "/oauth": "http://localhost:8080",
      "/ws": "ws://localhost:8080",
    },
  },

  // Pre-bundle deps used by every page so the first cold load doesn't
  // discover them lazily and trigger a re-bundle mid-session.
  optimizeDeps: {
    include: [
      "react",
      "react-dom",
      "react-router-dom",
      "jwt-decode",
    ],
  },

  build: {
    target: "es2022",
    sourcemap: false,
    chunkSizeWarningLimit: 1000,
  },
});