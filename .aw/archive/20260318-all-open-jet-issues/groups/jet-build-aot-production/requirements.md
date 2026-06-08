---
change: all-open-jet-issues
group: jet-build-aot-production
date: 2026-03-18
---

# Requirements

Implement the full AOT production build pipeline. This includes true module flattening (merging module bodies into a single scope), variable renaming to avoid collisions, and cross-module optimizations (constant inlining and DCE). Support tree shaking, code splitting (dynamic imports), and JS minification. Implement source map generation/chaining, a CSS pipeline (modules, imports), an asset pipeline (images, fonts, public/), and build-time configuration (env vars, define constants). Target: react-bench bundle size ≤ 196KB.
