---
number: 765
title: "feat(jet): AOT production build — tree shaking, code splitting, minification, source maps"
state: open
labels: [enhancement, P1, crate:jet]
group: "aot-build"
---

# #765 — feat(jet): AOT production build — tree shaking, code splitting, minification, source maps

## Goal

Production-quality AOT build pipeline comparable to Vite/esbuild/Rollup: tree shaking, code splitting, minification, source maps, CSS bundling, and asset pipeline.

## Milestone 1: Tree Shaking

| Feature | Description |
|---------|-------------|
| **ESM static analysis** | Detect unused exports via import/export graph |
| **Side-effect detection** | `package.json` `sideEffects` field support |
| **Dead code elimination** | Remove unreachable code paths |
| **Scope analysis** | Track variable references across modules |
| **Re-export flattening** | Inline barrel file re-exports |

## Milestone 2: Code Splitting

| Feature | Description |
|---------|-------------|
| **Dynamic import boundaries** | `import()` creates separate chunks |
| **Shared chunks** | Common modules extracted to shared chunk |
| **Entry points** | Multiple entry point support |
| **Chunk naming** | `[name].[hash].js` output pattern |
| **Preload hints** | Generate `<link rel="modulepreload">` |
| **Manual chunks** | Config-based chunk grouping |

### Splitting strategy

```
Entry A ─┬─► Chunk A (unique to A)
         ├─► Shared Chunk (used by A+B, threshold: 2+ importers)
Entry B ─┤
         └─► Chunk B (unique to B)

Dynamic import → Async Chunk (loaded on demand)
```

## Milestone 3: Minification

| Feature | Description |
|---------|-------------|
| **JS minification** | Identifier mangling, whitespace removal, constant folding |
| **CSS minification** | Shorthand collapsing, duplicate removal |
| **HTML minification** | Whitespace, comment removal |
| **Drop statements** | Remove `console.log`, `debugger` in production |
| **Name mangling** | Shorten variable names (scope-aware) |

### Implementation approach

Option A: Custom minifier using Tree-sitter AST (full control, reuse existing infra)
Option B: Integrate `swc_ecma_minifier` (battle-tested, but adds SWC dependency)
Option C: Integrate `oxc_minifier` (Rust-native, modern)

## Milestone 4: Source Maps

| Feature | Description |
|---------|-------------|
| **Source map generation** | VLQ-encoded mappings per transformation |
| **Source map chaining** | Compose maps: TS → JS → minified → bundled |
| **Inline source maps** | `//# sourceMappingURL=data:...` for dev |
| **External source maps** | `.map` files for production |
| **Original source** | `sourcesContent` for debugger display |

## Milestone 5: CSS Pipeline

| Feature | Description |
|---------|-------------|
| **CSS bundling** | Resolve `@import`, concatenate |
| **CSS Modules** | `.module.css` → scoped class names + JS map |
| **PostCSS integration** | Plugin pipeline (autoprefixer, nesting) |
| **Tailwind support** | JIT compilation integration |
| **Asset URLs** | Rewrite `url()` references, emit assets |

## Milestone 6: Asset Pipeline

| Feature | Description |
|---------|-------------|
| **Image optimization** | Resize, compress (already have `image` crate) |
| **Font handling** | Copy + hash, generate `@font-face` |
| **SVG** | Inline as React component or copy |
| **JSON** | Tree-shake JSON imports |
| **WASM** | Bundle WebAssembly modules (partial support exists) |
| **Static assets** | Copy `public/` dir with hashing |

## Milestone 7: Build Configuration

| Feature | Description |
|---------|-------------|
| **Environment variables** | `process.env.NODE_ENV` → `"production"` replacement |
| **Define** | Compile-time constant replacement |
| **Target** | ES2020, ES2022, ESNext output level |
| **Module format** | ESM, CJS, UMD, IIFE outputs |
| **Library mode** | For publishing npm packages (preserves exports) |
| **Externals** | Exclude deps from bundle (for SSR, libraries) |
| **Aliases** | Path aliases from `tsconfig.json` |

## Architecture Notes

```
Build Pipeline:
  1. Resolve entry points
  2. Build module graph (existing)
  3. Tree shake — mark used exports
  4. Code split — partition into chunks
  5. Transform — TS/JSX strip (existing)
  6. Bundle — concatenate per chunk
  7. Minify — compress output
  8. Source maps — compose chain
  9. Asset emit — copy + hash
  10. HTML inject — script/style tags

Output:
  dist/
  ├── index.html
  ├── assets/
  │   ├── main.[hash].js
  │   ├── vendor.[hash].js
  │   ├── main.[hash].css
  │   └── logo.[hash].svg
  └── main.[hash].js.map
```

## Acceptance Criteria

- [ ] `jet build` produces optimized output with tree shaking
- [ ] Dynamic `import()` creates separate async chunks
- [ ] Minified output is comparable in size to esbuild/Vite
- [ ] Source maps correctly map to original TS/JSX source
- [ ] CSS modules work with scoped class names
- [ ] `process.env.NODE_ENV` replaced at build time
- [ ] Build is reproducible (same input → same output hash)
