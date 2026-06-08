---
change: all-issues
group: jet-build-production
date: 2026-03-17
---

# Requirements

Implement a full AOT production build pipeline (jet build) with the following capabilities:
- Tree shaking (ESM analysis, side-effects).
- Code splitting (dynamic imports, shared chunks, multiple entry points).
- Minification (JS/CSS/HTML with mangling and constant folding).
- Source map generation (VLQ-encoded, chained, external/inline).
- CSS pipeline (bundling, modules, PostCSS/Tailwind support).
- Asset pipeline (hashing, optimization for images, fonts, WASM).
- Configuration (env vars, target levels, library mode, externals, aliases).
- Optimization: Implement scope hoisting (module concatenation) to achieve bundle sizes comparable to Webpack/Vite (target <= 196KB).
- Validation: Build and verify at least 2 real-world React applications (e.g., TodoMVC, Realworld App) using Playwright smoke tests.
