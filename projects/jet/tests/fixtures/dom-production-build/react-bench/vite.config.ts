// <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-tests-fixtures-dom-production-build-react-bench-vite-config-ts" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    preserveSymlinks: true,
    dedupe: ["react", "react-dom"],
  },
});

// </HANDWRITE>
