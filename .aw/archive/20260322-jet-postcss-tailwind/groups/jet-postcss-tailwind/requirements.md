---
change: jet-postcss-tailwind
group: jet-postcss-tailwind
date: 2026-03-22
---

# Requirements

Add a PostCSS plugin pipeline and Tailwind CSS JIT integration to the Jet bundler (`crate:jet`) to unblock Conductor frontend CSS compilation.

Three phases as described in the issue:

**Phase 1 — PostCSS Plugin System**
- Parse `postcss.config.js` (or a Jet-native config equivalent)
- Execute a CSS → plugin₁ → plugin₂ → … → output transformation chain
- `@import` resolution within the pipeline

**Phase 2 — Tailwind JIT Integration**
- Content path scanning via glob (`src/**/*.{ts,tsx}`)
- Utility class extraction from scanned source files
- JIT compilation: only emit CSS for classes actually used
- Directive processing: `@tailwind base/components/utilities`, `@apply`, `@layer`
- CSS variable theme extension (`hsl(var(--*))` pattern)
- Dark mode `class` strategy

**Phase 3 — Plugin & Tool Support**
- `tailwindcss-animate` — keyframe/animation utilities
- `@tailwindcss/typography` — prose styling
- `autoprefixer` — vendor prefix generation

**Dev & Production**
- Dev mode: watch source files + CSS for changes → incremental rebuild
- Production: minified CSS output

Acceptance criteria from issue:
- PostCSS plugin system architecture in place
- Tailwind JIT compilation within Jet build pipeline
- `@tailwind`, `@apply`, `@layer` directive processing
- Content path scanning + tree-shaking unused utilities
- Dark mode `class` strategy support
- CSS variable theme extension
- Autoprefixer for vendor prefixes
- Dev mode watch + rebuild on CSS/source changes
- Production minified CSS output
