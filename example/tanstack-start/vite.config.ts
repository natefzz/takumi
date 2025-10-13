import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import viteTsConfigPaths from "vite-tsconfig-paths";

const config = defineConfig({
  plugins: [viteTsConfigPaths(), tanstackStart(), viteReact()],
  ssr: {
    external: ["@takumi-rs/core"],
  },
  optimizeDeps: {
    exclude: ["@takumi-rs/core"],
  },
});

export default config;
