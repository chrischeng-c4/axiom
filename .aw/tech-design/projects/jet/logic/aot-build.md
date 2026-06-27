---
id: projects-jet-logic-aot-build-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Production Build Pipeline

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/aot-build.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Production Build Pipeline

### Overview

Jet's production-ready AOT build pipeline is comparable to Vite, esbuild, and
Rollup for production output. The pipeline covers tree shaking, code splitting,
minification, source maps, CSS bundling, asset handling, and build-time
configuration.

This spec replaces the old loose root spec
`.aw/tech-design/crates/jet/aot-build.md`. Root files are no longer allowed
under a crate spec directory except `README.md`; the active contract is now this
`logic/` spec.

### Implementation Status

All requirements are implemented. Key files:

| Area | Files |
|------|-------|
| Tree shaking | `bundler/tree_shake.rs`, `bundler/dce.rs`, `bundler/json_shake.rs` |
| Code splitting | `bundler/splitting.rs` |
| Minification | `bundler/minify.rs`, `bundler/mangle.rs`, `bundler/html_minify.rs`, `bundler/fold.rs` |
| Source maps | `bundler/sourcemap.rs` |
| CSS pipeline | `bundler/css_bundle.rs`, `css/mod.rs`, `css/tailwind/` |
| Asset pipeline | `asset/mod.rs`, `asset/image_processor.rs` |
| Scope hoisting | `bundler/scope_hoist.rs` |
| Build config | `bundler/define.rs`, `bundler/types.rs` |

### Requirements

```mermaid
---
id: jet-aot-build-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: ESM tree shaking removes unused exports
        risk: high
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Dynamic imports create async chunks
        risk: high
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Production minification shrinks JS and CSS
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: Source maps preserve original source mapping
        risk: high
        verifymethod: test
    }
    requirement R5 {
        id: R5
        text: CSS imports modules transforms and Tailwind are bundled
        risk: medium
        verifymethod: test
    }
    requirement R6 {
        id: R6
        text: Assets are hashed copied and referenced from output
        risk: medium
        verifymethod: test
    }
    requirement R7 {
        id: R7
        text: Defines and env values are replaced at build time
        risk: high
        verifymethod: test
    }
    requirement R8 {
        id: R8
        text: Static chunks emit modulepreload hints
        risk: medium
        verifymethod: test
    }
    requirement R9 {
        id: R9
        text: Manual chunk globs route matching modules first
        risk: medium
        verifymethod: test
    }
    requirement R10 {
        id: R10
        text: HTML minification preserves sensitive content blocks
        risk: medium
        verifymethod: test
    }
    requirement R11 {
        id: R11
        text: Source map chaining composes multi-pass transforms
        risk: high
        verifymethod: test
    }
    requirement R12 {
        id: R12
        text: Images are optimized when configured and worthwhile
        risk: medium
        verifymethod: test
    }
    requirement R13 {
        id: R13
        text: JSON named imports keep only used top-level keys
        risk: medium
        verifymethod: test
    }
    requirement R14 {
        id: R14
        text: CSS url references are rewritten to hashed assets
        risk: medium
        verifymethod: test
    }
```

### R1: ESM Tree Shaking

ESM-based tree shaking must support the `package.json` `sideEffects` field,
unused export elimination, CommonJS `require()` analysis, and re-export
flattening. See `logic/tree-shaking.md` for the full tree-shaking contract.

### R2: Code Splitting

Dynamic `import()` boundaries must create async chunks. Shared chunk extraction
must handle modules with two or more importers, and the build pipeline must
support multiple entry points.

### R3: Minification

The custom JavaScript minifier must remove whitespace and comments, mangle
identifiers with scope awareness, fold constants, replace booleans with compact
forms, and remove `console.log` and `debugger` statements. CSS minification is
performed through lightningcss. See `logic/variable-mangling.md` for identifier
mangling details.

### R4: Source Maps

Source map generation must use VLQ mappings. Modes include external `.map`
files, inline data URLs, hidden maps without comments, and no maps. Source
content must be included when available.

### R5: CSS Pipeline

The CSS pipeline must resolve `@import` with circular detection, support CSS
Modules, apply lightningcss transforms such as nesting and vendor prefixes, and
run the Tailwind JIT engine. See `logic/postcss-tailwind.md` for the Tailwind
contract.

### R6: Asset Pipeline

Image, font, SVG, and WASM assets must be processed with content-hash output
filenames shaped as `[name].[hash].[ext]`. Static assets from `public/` must be
copied into the distribution directory.

### R7: Define Replacement

Build-time replacement must cover `process.env.NODE_ENV`, `import.meta.env.*`,
custom defines, and `.env` file values with the `VITE_*` or `JET_*` prefixes.

### R8: Preload Hints

HTML output must generate `<link rel="modulepreload">` tags for static chunk
dependencies. Dynamic import chunks are excluded because they load on demand.

### R9: Manual Chunks

`BundleConfig.manual_chunks` maps chunk names to glob patterns. Matching modules
must be routed to named chunks before automatic shared chunk extraction.

### R10: HTML Minification

`minify_html()` must strip HTML comments, collapse whitespace between tags,
preserve whitespace inside `<pre>`, `<code>`, `<script>`, and `<style>`, and
remove unnecessary attribute quotes.

### R11: Source Map Chaining

`compose_source_maps()` must chain input maps through output maps across the
full TypeScript-to-JavaScript-to-bundled-to-minified pipeline. It must decode
and encode VLQ mappings and preserve `sourcesContent` from the original input.

### R12: Image Optimization

The image optimizer must re-encode JPEG at quality 85, optimize PNG bit depth,
strip SVG comments and whitespace, skip images under 1 KB, and honor configured
maximum width and height.

### R13: JSON Tree Shaking

Named imports from JSON files must keep only used top-level keys. Default and
namespace imports must retain the complete JSON object. The implementation
lives in `json_shake.rs`.

### R14: CSS Asset URL Rewriting

The CSS post-processor must rewrite `url()` references by resolving them
relative to the CSS file directory, processing them through the asset pipeline,
and replacing them with hashed output paths. Quoted URLs, unquoted URLs, and
query strings must be supported.

### BundleConfig Schema

```yaml
schema: jet://schemas/bundle-config
type: object
properties:
  entry:
    type: string
    description: Entry point file path
  output_dir:
    type: string
    default: dist
  output_format:
    enum: [esm, cjs, iife]
    default: esm
  minify:
    type: boolean
    default: true
  sourcemap:
    enum: [external, inline, hidden, none]
    default: external
  tree_shake:
    type: boolean
    default: true
  externals:
    type: array
    items:
      type: string
  externalize_all_packages:
    type: boolean
    default: false
  define:
    type: object
    additionalProperties:
      type: string
  manual_chunks:
    type: object
    description: Chunk name to glob patterns for manual chunk routing
    additionalProperties:
      type: array
      items:
        type: string
  hash_filenames:
    type: boolean
    default: true
```

### Scenarios

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: A production build imports only one symbol from an ESM module
    when: The bundler runs tree shaking
    then: Unused exports are removed from the emitted bundle
  - id: S2
    requirement: R2
    given: The graph contains a dynamic import boundary
    when: The bundler emits chunks
    then: The imported module is emitted as a separate async chunk
  - id: S3
    requirement: R3
    given: Production minification is enabled
    when: The build emits JavaScript and CSS
    then: Comments whitespace foldable constants and debug statements are removed safely
  - id: S4
    requirement: R4
    given: Source maps are configured as external inline or hidden
    when: The build writes output files
    then: Mappings point back to original TypeScript or JavaScript sources
  - id: S5
    requirement: R5
    given: CSS imports modules nesting prefixes and Tailwind utilities appear in input
    when: The CSS pipeline runs
    then: The final CSS bundle contains scoped classes transformed rules and generated utilities
  - id: S6
    requirement: R6
    given: Source modules reference image font SVG or WASM assets
    when: The asset pipeline runs
    then: Assets are copied with content hashes and references point to the hashed names
  - id: S7
    requirement: R7
    given: The build config defines environment and custom constants
    when: Define replacement runs
    then: Runtime code contains literal configured values
  - id: S8
    requirement: R8
    given: HTML output has static chunk dependencies
    when: The HTML emitter writes preload hints
    then: Only static chunk dependencies receive modulepreload tags
  - id: S9
    requirement: R9
    given: Manual chunk globs match modules in the graph
    when: Chunk assignment runs
    then: Matched modules are assigned before automatic shared chunk extraction
  - id: S10
    requirement: R10
    given: HTML contains removable comments whitespace and quoted attributes
    when: HTML minification runs
    then: The output is compact while protected content blocks keep required whitespace
  - id: S11
    requirement: R11
    given: Multiple transform passes produce source maps
    when: Source map composition runs
    then: The final map points through all intermediate maps to original sources
  - id: S12
    requirement: R12
    given: Image assets are large enough to optimize
    when: Image processing runs
    then: JPEG PNG and SVG outputs are optimized according to configured limits
  - id: S13
    requirement: R13
    given: A module imports named values from JSON
    when: JSON tree shaking runs
    then: Only referenced top-level keys remain in the emitted JSON module
  - id: S14
    requirement: R14
    given: CSS contains relative asset URLs
    when: CSS URL rewriting runs
    then: URLs resolve through the asset pipeline and point to hashed output paths
```

### Changes

```yaml
files:
  - path: .aw/tech-design/crates/jet/logic/aot-build.md
    action: MODIFY
    section: doc
    impl_mode: hand-written
    desc: Move the AOT build TD under logic and normalize checker-readable sections.
  - path: crates/jet/src/bundler/mod.rs
    action: NONE
    section: doc
    impl_mode: hand-written
    desc: Existing production build orchestration remains the source contract.
```
