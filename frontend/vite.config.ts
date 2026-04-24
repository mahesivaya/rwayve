import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'

// https://vite.dev/config/
export default {
  server: {
    proxy: {
      "/api": "http://localhost:8080",
      "/gmail": "http://localhost:8080",
      "/oauth": "http://localhost:8080",
      "/ws": "ws://localhost:8080",
    },
  },
};