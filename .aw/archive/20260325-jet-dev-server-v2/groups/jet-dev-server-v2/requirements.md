---
change: jet-dev-server-v2
group: jet-dev-server-v2
date: 2026-03-25
---

# Requirements

Overhaul Jet dev server to reliably serve modern React/TS projects:

1. Replace line-based CJS→ESM flattening with bundler-based per-dependency pre-bundling (#1089). Scan package.json deps, create virtual ESM entries, bundle with crate::bundler::Bundler, write to node_modules/.jet/. Cache invalidated by package.json/lockfile changes. Startup < 3s for ~20 CJS deps.

2. Move all TypeScript type stripping into transform_tsx() AST pass (#1090). Handle: export type, import type, inline type in imports, export interface, type aliases, declare statements, satisfies operator. Remove line-based post-filter. No false positives on 'type' as JS identifier.

3. Generate browser-compatible Node.js builtin polyfills during pre-bundling (#1091). Polyfill: crypto, url, buffer, path, events, util, querystring, process, stream. Stub with warning: fs, child_process, cluster, net, tls, etc.

4. Fix .jet-store symlink-based install so Node.js module resolution works (#1092). Packages with optionalDependencies (esbuild, rollup) must resolve. Create nested node_modules within store entries for their deps (pnpm-style).

Acceptance: cclab jet dev on create-react-app renders without errors. All @cclab/ui components serve without syntax errors. cclab jet install + npx vite works. react, react-dom, axios, date-fns pre-bundle correctly.
