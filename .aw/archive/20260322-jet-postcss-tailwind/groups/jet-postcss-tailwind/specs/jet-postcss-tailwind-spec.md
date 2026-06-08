---
id: jet-postcss-tailwind-spec
main_spec_ref: "cclab-jet/postcss-tailwind.md"
merge_strategy: new
filled_sections: [overview, requirements, scenarios, interaction, logic, state-machine, dependency, schema, config, test-plan, changes]
fill_sections: [overview, requirements, scenarios, interaction, logic, state-machine, dependency, schema, config, test-plan, changes]
create_complete: true
---

# Jet Postcss Tailwind Spec

## Overview

Add a native Rust PostCSS pipeline and Tailwind CSS JIT engine to Jet (`crates/cclab-jet`), unblocking Conductor frontend CSS compilation (issue #1029, part of #1014).

### Key Decisions (from pre-clarifications)

- **Execution model**: Native Rust only — no Node.js. `lightningcss` (Parcel's Rust CSS parser/transformer) is the single CSS engine for parsing, transforming, and minifying.
- **Config**: `tailwind.config.js` parsed via minimal JS object literal evaluator; `jet.config.yaml` `css.tailwind` section as alternative. `postcss.config.js` is not parsed — Jet replaces PostCSS entirely. JS config takes precedence when both exist.
- **JS plugins**: Out of scope. `tailwindcss-animate` and `@tailwindcss/typography` are implemented as native Rust emitters; `autoprefixer` is handled via lightningcss vendor prefix transforms.
- **Dev mode**: CSS rebuild integrated into Jet's existing `notify` watcher (dev_server module) — single watcher, no separate process.
- **Minification**: `lightningcss` handles CSS minification in production builds (mirrors JS minification via `oxc_minifier`).

### Scope

Extends `aot-build.md` R5 (CSS Pipeline) with full Tailwind JIT support. Integrates with `jit-runner.md` dev_server/watch infrastructure and respects `tree-shaking.md` R4 (CSS imports treated as side effects — always preserved).

### Current State

Jet CSS handling today:
- Basic CSS injection (wraps in `<style>` tag)
- `@import` resolution (partial)
- No PostCSS plugin system
- No Tailwind scanning/JIT

### Target State

```
index.css → CssPipeline:
  1. @import resolution (inline all @import statements)
  2. @tailwind directive detection
  3. ContentScanner: glob scan src/**/*.{ts,tsx} → used class set
  4. TailwindEmitter: generate base/components/utilities layers
  5. @apply expansion → CSS rules
  6. @layer custom rules routing
  7. lightningcss transform (nesting, vendor prefixes)
  8. lightningcss minify (production only)
  → dist/[name].[hash].css
```
## Requirements

### R1: lightningcss CSS Transform Engine

Integrate `lightningcss` crate as the CSS parse/transform/minify engine:
- Parse CSS source into lightningcss AST
- Apply transforms: nesting, custom media queries, vendor prefixing
- Minify in production mode (`minify: true`)
- Output: transformed CSS string + optional source map

### R2: @import Resolution

Within the CSS pipeline, resolve and inline `@import` statements:
- Resolve relative paths from the importing file's directory
- Resolve node_modules CSS (e.g. `@import "normalize.css"`)
- Detect and error on circular imports via visited-path set
- Process imported files recursively through the same pipeline

### R3: Tailwind Content Scanning

Scan source files for Tailwind utility class names:
- Load content glob patterns from `tailwind.config.js` or `jet.config.yaml`
- Walk matching files using the glob patterns
- Extract class names from: `className="..."`, `class="..."`, template literals, conditional class patterns (clsx/cn)
- Build a used-class `HashSet<String>` for JIT emission

### R4: Tailwind JIT Utility Emission

Generate CSS only for classes present in the scanned content:
- Map each used class to its CSS rule (e.g. `flex` → `display: flex`)
- Handle responsive prefixes: `sm:`, `md:`, `lg:`, `xl:`, `2xl:`
- Handle variant prefixes: `hover:`, `focus:`, `active:`, `dark:`, `group-*`, `peer-*`
- Handle arbitrary values: `w-[300px]`, `text-[#ff0000]`
- Emit only used rules — never emit unused utilities

### R5: Tailwind Directive Processing

Process Tailwind CSS directives in input CSS:
- `@tailwind base` → inject Preflight (CSS reset + base styles)
- `@tailwind components` → inject component-layer CSS
- `@tailwind utilities` → inject generated utility CSS
- `@apply <classes>` → expand into corresponding CSS property declarations at point of use
- `@layer base|components|utilities { ... }` → route custom rules to the correct layer

### R6: CSS Variable Theme Extension

Support `hsl(var(--*))` pattern for theme color extension:
- Parse `theme.extend.colors` in `tailwind.config.js` / `jet.config.yaml`
- Generate CSS custom property declarations in `:root { --color-name: ... }`
- Reference via `hsl(var(--color-name))` in generated utility classes

### R7: Dark Mode Class Strategy

Support Tailwind's `class` dark mode strategy:
- `dark:` variant generates CSS rules scoped to `.dark <selector>` (HTML class toggle)
- Only `class` strategy is supported (no `prefers-color-scheme` media query mode)

### R8: Native Plugin Emitters

Implement fixed plugin outputs as native Rust emitters (no JS execution):
- **tailwindcss-animate**: emit keyframe + animation utility CSS (enter/exit/spin/ping/bounce/pulse)
- **@tailwindcss/typography**: emit `prose` class CSS (typography baseline, size variants, dark:prose-invert)
- **autoprefixer**: handled via lightningcss vendor prefix transforms (enabled by default)

### R9: Dev Mode Watch Integration

Integrate CSS rebuild into Jet's existing `notify` watcher (`dev_server` module):
- Watch `.css` files for changes → trigger CSS pipeline rebuild
- Watch content-scanned source files (`.ts`, `.tsx`) for class name changes → trigger CSS pipeline rebuild
- On rebuild: emit updated CSS → trigger HMR reload (CSS hot replacement)
- Single watcher instance — no separate CSS watcher process

### R10: Production CSS Minification

In production builds (`jet build`):
- Pass `minify: true` to `lightningcss` transform
- Strip comments, normalize whitespace, shorten values
- Output single minified CSS file in `dist/` with content hash (`[name].[hash].css`)

### R11: Config Parsing

Load Tailwind configuration from:
- `tailwind.config.js` (primary): parse via minimal JS object literal evaluator (handles most real-world configs)
- `jet.config.yaml` `css.tailwind` section (alternative): YAML-native config
- JS config takes precedence when both exist
- Supported config fields: `content`, `darkMode`, `theme`, `theme.extend`, `plugins` (string names only)
## Scenarios

### S1: Tailwind Utilities in Production Build

**Given** Conductor frontend with `index.css` containing `@tailwind base`, `@tailwind components`, `@tailwind utilities`
**And** source files using classes `flex`, `text-blue-500`, `dark:bg-gray-900`
**When** `jet build` runs
**Then** output CSS contains only rules for `flex`, `text-blue-500`, `dark:bg-gray-900`
**And** unused utilities are absent from output
**And** output CSS is minified in `dist/[name].[hash].css`

### S2: @apply Directive Expansion

**Given** CSS source `.btn { @apply flex items-center px-4 py-2 rounded; }`
**When** CSS pipeline processes the file
**Then** `.btn` is expanded to the corresponding property declarations (`display`, `align-items`, `padding`, `border-radius`)
**And** no `@apply` remains in output

### S3: @layer Custom Rules

**Given** CSS source `@layer base { *, ::before, ::after { box-sizing: border-box; } }`
**When** CSS pipeline processes the file
**Then** the custom rule is injected into the base layer
**And** it appears before component and utility layers in output

### S4: Dark Mode Class Strategy

**Given** source file contains classes `dark:bg-slate-800 dark:text-white`
**And** `tailwind.config.js` has `darkMode: "class"`
**When** CSS pipeline generates utilities
**Then** output contains `.dark .dark\:bg-slate-800 { background-color: rgb(30 41 59); }`
**And** toggling `.dark` class on `<html>` switches the theme

### S5: CSS Variable Theme Colors

**Given** `tailwind.config.js` has `theme.extend.colors = { primary: "hsl(var(--primary))" }`
**And** source uses `text-primary`, `bg-primary`
**When** CSS pipeline generates output
**Then** output CSS contains `--primary` referenced in `.text-primary { color: hsl(var(--primary)); }`
**And** `.bg-primary { background-color: hsl(var(--primary)); }` is emitted

### S6: Dev Mode Watch + Rebuild

**Given** `jet dev` is running with CSS pipeline active
**When** a `.tsx` source file is saved with a new Tailwind class `mt-8`
**Then** CSS pipeline rescans content and emits updated CSS within 500ms
**And** browser receives HMR CSS update without full page reload

### S7: tailwindcss-animate Keyframes

**Given** plugin `tailwindcss-animate` is listed in Tailwind config
**When** source uses class `animate-spin`
**Then** output contains `@keyframes spin { to { transform: rotate(360deg) } }`
**And** `.animate-spin { animation: spin 1s linear infinite; }` is emitted

### S8: @tailwindcss/typography Prose

**Given** plugin `@tailwindcss/typography` is listed in Tailwind config
**And** source uses `prose prose-lg dark:prose-invert`
**When** CSS pipeline generates output
**Then** typography baseline CSS for `.prose` and `.prose-lg` is emitted
**And** dark mode prose overrides are scoped to `.dark .dark\:prose-invert`

### S9: @import Inlining

**Given** `index.css` contains `@import "./tokens.css"` and `@import "normalize.css"`
**When** CSS pipeline processes `index.css`
**Then** both imports are inlined at the `@import` positions
**And** the final output is a single CSS string with no remaining `@import` statements

### S10: tailwind.config.js Parsing

**Given** `tailwind.config.js` with:
```js
module.exports = {
  content: ["./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: { extend: { colors: { primary: "hsl(var(--primary))" } } },
  plugins: [require("tailwindcss-animate"), require("@tailwindcss/typography")]
}
```
**When** `TailwindConfig::from_js` parses the file
**Then** `config.content = ["./src/**/*.{ts,tsx}"]`
**And** `config.dark_mode = DarkMode::Class`
**And** `config.plugins` includes `tailwindcss-animate` and `@tailwindcss/typography`
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan

### T1: lightningcss Parse + Nesting Transform (R1)

**Given** input CSS `a { color: red; &:hover { color: blue; } }`
**When** `CssPipeline::process` transforms with nesting enabled
**Then** output contains `a:hover { color: blue; }` (nesting flattened)

### T2: lightningcss Production Minification (R1, R10)

**Given** CSS with comments and extra whitespace
**When** `CssPipeline::process` runs in production mode
**Then** output contains no comments and minimal whitespace
**And** `CssOutput.hash` is populated

### T3: @import Relative Resolution (R2)

**Given** `index.css` has `@import "./tokens.css"` and `tokens.css` exists with `:root { --bg: white; }`
**When** `CssPipeline::process("index.css")` runs
**Then** output contains `:root { --bg: white; }` inlined at the `@import` position

### T4: @import Circular Detection (R2)

**Given** `a.css` imports `b.css` which imports `a.css`
**When** `CssPipeline::process("a.css")` runs
**Then** an `Err` is returned with a message indicating circular import

### T5: Content Scanning Extracts Classes (R3)

**Given** source file `src/App.tsx` contains `className="flex items-center text-blue-500"`
**And** content glob is `["./src/**/*.{ts,tsx}"]`
**When** `ContentScanner::scan` runs
**Then** result set contains `"flex"`, `"items-center"`, `"text-blue-500"`

### T6: JIT Utility Emission — Used Only (R4)

**Given** used classes `{"flex", "items-center"}`
**When** `TailwindEmitter::generate` produces utilities
**Then** output contains `.flex { display: flex; }` and `.items-center { align-items: center; }`
**And** `.hidden { display: none; }` is absent

### T7: Responsive Prefix Emission (R4)

**Given** used class `"md:text-lg"`
**When** `TailwindEmitter::generate` produces utilities
**Then** output contains `@media (min-width: 768px) { .md\:text-lg { font-size: 1.125rem; line-height: 1.75rem; } }`

### T8: @tailwind Directive Injection (R5)

**Given** CSS input `@tailwind base;\n@tailwind utilities;`
**When** CSS pipeline processes it
**Then** output at `@tailwind base` position contains Preflight CSS
**And** output at `@tailwind utilities` position contains generated utilities

### T9: @apply Expansion (R5)

**Given** CSS `.btn { @apply flex rounded px-4; }`
**When** CSS pipeline processes the file
**Then** output contains `.btn { display: flex; border-radius: 0.25rem; padding-left: 1rem; padding-right: 1rem; }`
**And** no `@apply` remains in output

### T10: @layer Custom Base (R5)

**Given** CSS `@layer base { h1 { @apply text-2xl font-bold; } }`
**When** CSS pipeline processes
**Then** the custom h1 rule is injected into the base layer output
**And** appears before component and utility layers

### T11: CSS Variable Theme Color (R6)

**Given** config `theme.extend.colors.primary = "hsl(var(--primary))"`
**And** source uses `text-primary`
**When** CSS pipeline generates output
**Then** `.text-primary { color: hsl(var(--primary)); }` is emitted

### T12: Dark Mode Class Strategy (R7)

**Given** `darkMode: "class"` in config
**And** source uses class `dark:bg-slate-800`
**When** CSS pipeline generates output
**Then** output contains `.dark .dark\:bg-slate-800 { background-color: rgb(30 41 59); }`

### T13: tailwindcss-animate Keyframes (R8)

**Given** plugin `tailwindcss-animate` in config
**And** source uses `animate-spin`
**When** `AnimateEmitter::emit` runs
**Then** output contains `@keyframes spin { to { transform: rotate(360deg); } }`
**And** `.animate-spin { animation: spin 1s linear infinite; }` is emitted

### T14: @tailwindcss/typography Prose (R8)

**Given** plugin `@tailwindcss/typography` in config
**And** source uses `prose prose-lg`
**When** `TypographyEmitter::emit` runs
**Then** output contains `.prose` and `.prose-lg` with typography CSS rules

### T15: Dev Watch CSS Rebuild (R9)

**Given** `jet dev` running with CSS pipeline
**When** a `.tsx` file is saved with a new class `mt-8`
**Then** within 500ms CSS pipeline rescans and emits updated CSS
**And** HMR sends CSS update to the connected browser

### T16: tailwind.config.js Parsing (R11)

**Given** `tailwind.config.js` with `module.exports = { content: ["./src/**/*.tsx"], darkMode: "class" }`
**When** `TailwindConfig::from_js` parses the file
**Then** `config.content = ["./src/**/*.tsx"]`
**And** `config.dark_mode = DarkMode::Class`

### T17: Conductor End-to-End Build

**Given** Conductor frontend project with real `tailwind.config.js` and `index.css`
**When** `jet build` runs
**Then** `dist/index.[hash].css` is produced
**And** the CSS file contains all utility classes used in the Conductor source
**And** no `@tailwind` or `@apply` directives remain in the output
## Changes

```yaml
files:
  # CSS pipeline core
  - path: crates/cclab-jet/src/css/mod.rs
    action: CREATE
    desc: CssPipeline struct — orchestrates parse → import-resolve → directive-detect → emit → transform → (minify)

  - path: crates/cclab-jet/src/css/import_resolver.rs
    action: CREATE
    desc: Inline @import directives recursively; circular import detection via visited PathBuf set

  - path: crates/cclab-jet/src/css/directives.rs
    action: CREATE
    desc: Detect @tailwind/@apply/@layer directives; expand @apply to CSS rules; route @layer custom content

  - path: crates/cclab-jet/src/css/output.rs
    action: CREATE
    desc: CssOutput struct — css String, source_map Option<String>, hash String (first 8 hex of SHA-256)

  # Tailwind JIT engine
  - path: crates/cclab-jet/src/css/tailwind/mod.rs
    action: CREATE
    desc: TailwindEmitter — generates base/components/utilities TailwindLayers from used class HashSet

  - path: crates/cclab-jet/src/css/tailwind/scanner.rs
    action: CREATE
    desc: ContentScanner — walk glob patterns via globset + walkdir; extract class names from .ts/.tsx source

  - path: crates/cclab-jet/src/css/tailwind/config.rs
    action: CREATE
    desc: TailwindConfig — from_js (minimal JS object literal parser) and from_yaml; DarkMode enum; ThemeConfig

  - path: crates/cclab-jet/src/css/tailwind/utilities.rs
    action: CREATE
    desc: Utility class → CSS rule table; responsive prefix (sm:/md:/lg:/xl:/2xl:) expansion; arbitrary value parsing

  - path: crates/cclab-jet/src/css/tailwind/variants.rs
    action: CREATE
    desc: Variant prefix handling — hover:/focus:/active:/dark: + responsive breakpoints; selector wrapping

  - path: crates/cclab-jet/src/css/tailwind/preflight.rs
    action: CREATE
    desc: Tailwind Preflight CSS (base reset) as embedded static &str (compile-time include_str!)

  # Plugin emitters
  - path: crates/cclab-jet/src/css/plugins/mod.rs
    action: CREATE
    desc: PluginEmitter trait (emit + name); plugin registry; dispatch by name to concrete emitters

  - path: crates/cclab-jet/src/css/plugins/animate.rs
    action: CREATE
    desc: AnimateEmitter — native Rust emit for tailwindcss-animate keyframes and animation utilities

  - path: crates/cclab-jet/src/css/plugins/typography.rs
    action: CREATE
    desc: TypographyEmitter — native Rust emit for @tailwindcss/typography prose classes and dark variants

  # Integration with existing build pipeline
  - path: crates/cclab-jet/src/bundler/mod.rs
    action: MODIFY
    desc: Detect CSS entry file; invoke CssPipeline::process; write dist/[name].[hash].css to output

  - path: crates/cclab-jet/src/dev_server/mod.rs
    action: MODIFY
    desc: Register CSS entry + content glob patterns with existing notify watcher; on change trigger CssPipeline rebuild + HMR CSS update

  - path: crates/cclab-jet/src/lib.rs
    action: MODIFY
    desc: Export pub mod css

  - path: crates/cclab-jet/Cargo.toml
    action: MODIFY
    desc: Add dependencies — lightningcss, globset, walkdir; sha2 for content hash
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
