---
change: jet-aot-build-gaps
group: jet-aot-build-gaps
date: 2026-03-24
---

# Requirements

Complete remaining AOT production build gaps in cclab-jet bundler. Core features (tree shaking, code splitting, minification, scope hoisting, DCE, source maps, CSS bundling, define replacement) are already implemented. This change fills the remaining gaps: (1) preload hints — generate <link rel="modulepreload"> for chunk dependencies, (2) manual chunks config — user-configurable chunk boundaries beyond dynamic imports, (3) HTML minification — whitespace/comment removal for HTML output, (4) source map chaining — compose maps across multi-pass transforms (TS → JS → minified → bundled), (5) image optimization — implement actual compression in the existing TODO stub, (6) JSON tree-shaking — dead code elimination for JSON imports, (7) CSS asset URL rewriting — rewrite url() references to hashed asset paths.
