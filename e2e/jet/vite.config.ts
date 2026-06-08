import { defineConfig } from "vite";

export default defineConfig({
  // Use classic JSX runtime so createElement calls match our mini-react
  esbuild: {
    jsx: "transform",
    jsxFactory: "createElement",
    jsxFragment: "Fragment",
  },
  build: {
    outDir: "dist-vite",
    minify: false, // No minify so we can compare output behavior
  },
});
