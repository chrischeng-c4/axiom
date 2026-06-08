---
change: jet-aot-build
group: aot-build
date: 2026-03-12
---

# Requirements

Production AOT build pipeline for jet. Scoped to Phase 1 (verification) and Phase 2 (core optimizations).

### Phase 1: Build Correctness Verification
- Create a React TodoMVC test app as build fixture
- Build same source with both `jet build` and Vite
- Compare outputs via Playwright DOM snapshot tests
- Verify functional equivalence: rendering, event handling, state updates

### Phase 2: Core Build Optimizations
- **Tree shaking**: ESM static analysis to detect unused exports, remove dead code from bundle
- **Environment replacement**: `process.env.NODE_ENV` → `"production"` at build time, compile-time constant replacement via `define` config
- **Minification**: Whitespace removal, comment stripping, `console.log`/`debugger` drop (no identifier mangling in this phase)
- **External source maps**: Generate `.map` files for production builds with correct mappings back to original TS/JSX source

### Existing Infrastructure to Build On
- `bundler/` module: module graph, import resolution, concatenation
- `transform/` module: Tree-sitter TS/JSX stripping
- `resolver/` module: node_modules resolution

### Acceptance Criteria (Phase 1+2)
- `jet build` produces working output for a React TodoMVC app
- Playwright tests pass identically for jet and Vite builds
- Unused exports are eliminated from the bundle
- `process.env.NODE_ENV` replaced at build time
- External `.map` files generated
- Build is reproducible (same input → same output hash)
