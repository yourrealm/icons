import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Dev: the Rust server renders the marks; ICONS_PORT moves it (default 3000).
const port = process.env.ICONS_PORT ?? "3000";
const target = `http://127.0.0.1:${port}`;

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: Object.fromEntries(
      ["/api", "/char", "/fa", "/ph", "/shape", "/healthz"].map((p) => [p, { target }]),
    ),
  },
});
