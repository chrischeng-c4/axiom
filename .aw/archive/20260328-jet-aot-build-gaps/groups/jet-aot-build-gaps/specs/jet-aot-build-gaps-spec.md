---
id: jet-aot-build-gaps-spec
main_spec_ref: crates/cclab-jet/logic/aot-build.md
merge_strategy: extend
---

# AOT Build Pipeline — Remaining Gaps

## Overview

Complete remaining gaps in Jet's AOT production build pipeline. Core features (tree shaking, code splitting, minification, scope hoisting, DCE, source maps, CSS bundling, define replacement) are already implemented. This change adds 7 missing features:

1. **Preload hints** — Generate `<link rel="modulepreload">` tags for chunk dependencies in HTML output
2. **Manual chunks** — Jet-native config for user-defined chunk boundaries (name → glob pattern mapping)
3. **HTML minification** — Custom whitespace/comment removal for HTML output (no new dependency)
4. **Source map chaining** — Compose maps across multi-pass transforms (TS → JS → minified → bundled)
5. **Image optimization** — Implement actual compression using existing `image` crate (currently TODO stub)
6. **JSON tree-shaking** — Dead code elimination for JSON imports (only include used keys)
7. **CSS asset URL rewriting** — Rewrite `url()` references in CSS to point to hashed asset paths

### Current State

- `bundler/splitting.rs`: Dynamic import + shared chunks done. No preload hints, no manual chunks.
- `bundler/minify.rs`: JS + CSS minification done. No HTML minification.
- `bundler/sourcemap.rs`: VLQ + inline/external modes done. No multi-pass chaining.
- `asset/image_processor.rs`: TODO stub — reads file + hashes only.
- `asset/mod.rs`: Generic hashing done. No JSON tree-shake, no CSS URL rewrite.

## Requirements

### R8: Preload Hints for Code-Split Chunks

When `jet build` produces multiple chunks via code splitting, generate `<link rel="modulepreload">` tags in the HTML output for all static chunk dependencies:
- For each entry chunk, trace its static imports to identify preload candidates
- Emit `<link rel="modulepreload" href="assets/chunk-name.[hash].js">` in `<head>`
- Only preload direct static dependencies, not dynamic imports (those load on demand)
- Return chunk dependency metadata from `split_chunks()` for HTML injection

### R9: Manual Chunks Configuration

Add `manual_chunks` field to `BundleConfig` (`types.rs`):
- Jet-native format: `HashMap<String, Vec<String>>` — chunk name → glob patterns
- Example: `{ "vendor": ["node_modules/react/**", "node_modules/react-dom/**"] }`
- During `split_chunks()`, modules matching a manual chunk pattern are routed to that named chunk
- Manual chunks take priority over automatic shared chunk extraction
- Glob matching via existing `globset` dependency

### R10: HTML Minification

Add `minify_html()` function to `minify.rs`:
- Strip HTML comments (`<!-- ... -->`)
- Collapse whitespace between tags (preserve within `<pre>`, `<code>`, `<script>`, `<style>`)
- Remove optional closing tags where safe (e.g., `</li>`, `</p>`)
- Remove unnecessary quotes on attributes with simple values
- No new dependency — custom implementation using string scanning

### R11: Source Map Chaining

Compose source maps across multiple transformation passes in `sourcemap.rs`:
- Accept an input source map (from TS → JS transform)
- Map output positions through the input map to get original positions
- Support the chain: TypeScript source → JS transform → bundled → minified
- Use VLQ decode/encode for each mapping in the chain
- Preserve `sourcesContent` from the original input map

### R12: Image Optimization

Replace TODO stub in `image_processor.rs` with actual compression using the `image` crate:
- JPEG: Re-encode at quality 85 (configurable)
- PNG: Apply basic optimization (reduce bit depth where possible)
- WebP: Pass through (already optimized format)
- SVG: Strip comments and unnecessary whitespace
- Skip optimization for images under 1KB (overhead not worth it)
- Preserve original dimensions unless `max_width`/`max_height` is configured

### R13: JSON Tree-Shaking

When a JS/TS file imports from a `.json` file with destructured import:
- Parse the JSON and identify which top-level keys are used
- Emit only the used keys in the bundled output
- `import data from './data.json'` (default import) → include all keys (namespace usage)
- `import { name, version } from './package.json'` → include only `name` and `version`
- Implementation in `tree_shake.rs` — extend used-export analysis to handle JSON modules

### R14: CSS Asset URL Rewriting

When CSS contains `url()` references to assets (images, fonts):
- Resolve the URL relative to the CSS file's directory
- Process the referenced asset through the asset pipeline (hash, copy to dist/)
- Rewrite the `url()` to point to the hashed output path: `url(../assets/logo.[hash].svg)`
- Handle both quoted and unquoted URLs, and `url()` with query strings
- Implementation in `css_bundle.rs` — post-process CSS output before writing to dist/

## Scenarios

### S9: Preload Hints in HTML Output

**Given** `jet build` splits into `main.[hash].js` and `vendor.[hash].js` (shared chunk)
**When** HTML output is generated
**Then** `<head>` contains `<link rel="modulepreload" href="assets/vendor.[hash].js">`
**And** dynamic import chunks are NOT preloaded

### S10: Manual Chunks Override

**Given** `BundleConfig.manual_chunks = { "vendor": ["node_modules/react/**"] }`
**And** `react` and `react-dom` are dependencies
**When** `split_chunks()` runs
**Then** all react/react-dom modules go into `vendor.[hash].js`
**And** they are NOT in the shared chunk

### S11: HTML Minification

**Given** HTML with comments and extra whitespace: `<!-- comment -->  <div>  <p>text</p>  </div>`
**When** `minify_html()` processes it
**Then** output is `<div><p>text</p></div>` (comments removed, whitespace collapsed)

### S12: Source Map Chain

**Given** TypeScript source with a source map from TS → JS transform
**When** the JS is bundled and minified with source map chaining
**Then** the final source map correctly maps minified output positions to original TypeScript lines

### S13: Image Optimization

**Given** a 50KB JPEG image referenced by the application
**When** `optimize_image()` processes it
**Then** output is re-encoded at quality 85, resulting in smaller file size
**And** the hashed filename is used in the output

### S14: JSON Tree-Shake

**Given** `import { name } from './package.json'` in source code
**And** `package.json` has 20 fields
**When** tree shaking runs
**Then** bundled output contains only `{ "name": "..." }` from the JSON

### S15: CSS URL Rewrite

**Given** CSS contains `background: url(../images/logo.svg)`
**When** CSS is bundled for production
**Then** `logo.svg` is processed (hashed, copied to dist/assets/)
**And** CSS output contains `background: url(assets/logo.[hash].svg)`

## Changes

```yaml
files:
  # Code splitting enhancements
  - path: crates/cclab-jet/src/bundler/splitting.rs
    action: MODIFY
    desc: Add preload hint metadata to SplitResult; implement manual_chunks routing with glob matching

  - path: crates/cclab-jet/src/bundler/types.rs
    action: MODIFY
    desc: Add manual_chunks field to BundleConfig; add PreloadHint struct

  # HTML minification
  - path: crates/cclab-jet/src/bundler/minify.rs
    action: MODIFY
    desc: Add minify_html() function — comment removal, whitespace collapsing, preserve pre/code/script/style

  # Source map chaining
  - path: crates/cclab-jet/src/bundler/sourcemap.rs
    action: MODIFY
    desc: Add compose_source_maps() — chain input map through output map, VLQ decode/encode

  # Image optimization
  - path: crates/cclab-jet/src/asset/image_processor.rs
    action: MODIFY
    desc: Replace TODO stub with actual compression — JPEG quality 85, PNG optimization, SVG minify

  # JSON tree-shaking
  - path: crates/cclab-jet/src/bundler/tree_shake.rs
    action: MODIFY
    desc: Extend used-export analysis to handle JSON modules — emit only used top-level keys

  # CSS URL rewriting
  - path: crates/cclab-jet/src/bundler/css_bundle.rs
    action: MODIFY
    desc: Add post-process pass to rewrite url() references to hashed asset paths

  # Bundler orchestrator
  - path: crates/cclab-jet/src/bundler/mod.rs
    action: MODIFY
    desc: Wire preload hints into HTML output; pass manual_chunks config to split_chunks()
```

## Test Plan

### T12: Preload Hints Generated (R8)

**Given** bundle output with 2 chunks: `main.[hash].js` importing `vendor.[hash].js`
**When** HTML output is generated
**Then** `<link rel="modulepreload" href="assets/vendor.[hash].js">` is present in `<head>`

### T13: Manual Chunks Routing (R9)

**Given** `manual_chunks = { "vendor": ["node_modules/react/**"] }` and react is a dependency
**When** `split_chunks()` runs
**Then** react modules appear in the vendor chunk, not in shared or entry chunks

### T14: HTML Comment Removal (R10)

**Given** input `<!-- todo -->  <div>hello</div>`
**When** `minify_html()` runs
**Then** output is `<div>hello</div>`

### T15: HTML Preserves Pre/Code Content (R10)

**Given** input `<pre>  spaces  matter  </pre>`
**When** `minify_html()` runs
**Then** whitespace inside `<pre>` is preserved

### T16: Source Map Chaining (R11)

**Given** input source map mapping line 5 → original line 10, and output map mapping line 3 → bundled line 5
**When** `compose_source_maps()` chains them
**Then** result maps line 3 → original line 10

### T17: JPEG Optimization (R12)

**Given** a JPEG image file
**When** `optimize_image()` processes it
**Then** output is valid JPEG with smaller or equal file size

### T18: Small Image Skip (R12)

**Given** a 500-byte PNG image
**When** `optimize_image()` processes it
**Then** original bytes are returned unchanged (under 1KB threshold)

### T19: JSON Tree-Shake Named Import (R13)

**Given** `import { name } from './pkg.json'` and pkg.json has `{ "name": "x", "version": "1", "desc": "y" }`
**When** JSON tree-shaking runs
**Then** bundled JSON contains only `{ "name": "x" }`

### T20: JSON Default Import Keeps All (R13)

**Given** `import data from './config.json'`
**When** JSON tree-shaking analyzes the import
**Then** all keys are preserved (default import = use all)

### T21: CSS URL Rewrite (R14)

**Given** CSS `background: url(../img/logo.svg)` and logo.svg exists
**When** CSS URL rewriting runs
**Then** output URL is rewritten to `url(assets/logo.[hash].svg)` and the asset is copied to dist/

# Reviews
