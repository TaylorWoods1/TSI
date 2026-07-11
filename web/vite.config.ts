import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/health": "http://127.0.0.1:7700",
      "/venue": "http://127.0.0.1:7700",
      "/ik": "http://127.0.0.1:7700",
      "/fk": "http://127.0.0.1:7700",
      "/jacobian": "http://127.0.0.1:7700",
      "/feasible": "http://127.0.0.1:7700",
      "/workspace": "http://127.0.0.1:7700",
      "/traj": "http://127.0.0.1:7700",
      "/scene": "http://127.0.0.1:7700",
      "/calibration": "http://127.0.0.1:7700",
      "/run": "http://127.0.0.1:7700",
    },
  },
});
