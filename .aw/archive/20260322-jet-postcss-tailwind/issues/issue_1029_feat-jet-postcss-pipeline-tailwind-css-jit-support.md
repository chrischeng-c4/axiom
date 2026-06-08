---
number: 1029
title: "feat(jet): PostCSS pipeline + Tailwind CSS JIT support"
state: open
labels: [type:enhancement, priority:p1, crate:jet, project:conductor]
group: "jet-postcss-tailwind"
---

# #1029 — feat(jet): PostCSS pipeline + Tailwind CSS JIT support

## Context

Part of #1014. Conductor frontend uses TailwindCSS extensively. Jet currently has **no PostCSS pipeline**, making Tailwind integration impossible without external workarounds.

This is the #1 blocker for Conductor frontend migration.

## Current State

Jet CSS handling today:
- Basic CSS injection (wraps in `<style>` tag)
- `@import` resolution (partial)
- No PostCSS plugin system
- No Tailwind scanning/JIT

## Conductor's CSS Pipeline

```
postcss.config.js:
  tailwindcss → autoprefixer

index.css:
  @tailwind base;
  @tailwind components;
  @tailwind utilities;
  @layer base { ... }

tailwind.config.js:
  content: ["./src/**/*.{ts,tsx}"]
  darkMode: "class"
  theme.extend: { colors: hsl(var(--*)) }
  plugins: [tailwindcss-animate, @tailwindcss/typography]
```

## Requirements

### Phase 1: PostCSS Plugin System
- Parse `postcss.config.js` (or `jet.config.yaml` equivalent)
- Plugin pipeline: CSS → plugin1 → plugin2 → ... → output
- Support `@import` resolution within pipeline

### Phase 2: Tailwind Integration
- Content path scanning (glob `src/**/*.{ts,tsx}`)
- Utility class extraction from source files
- JIT compilation of used classes only
- `@tailwind` directive processing
- `@apply` directive processing
- `@layer` directive processing
- Theme extension via CSS variables (`hsl(var(--*))`)
- Dark mode class-based toggle (`class` strategy)

### Phase 3: Tailwind Plugin Support
- `tailwindcss-animate` — keyframe/animation utilities
- `@tailwindcss/typography` — prose class styling
- `autoprefixer` — vendor prefix generation

## Workaround (Until Native Support)

```bash
# Standalone Tailwind CLI as a build step
npx tailwindcss -i src/index.css -o src/tailwind-compiled.css --watch
# Jet handles JS/TS bundling only
cclab jet dev --port 3201
```

## Acceptance Criteria

- [ ] PostCSS plugin system architecture
- [ ] Tailwind JIT compilation within Jet build pipeline
- [ ] `@tailwind`, `@apply`, `@layer` directive processing
- [ ] Content path scanning + tree-shaking unused utilities
- [ ] Dark mode `class` strategy support
- [ ] CSS variable theme extension (`hsl(var(--*))`)
- [ ] Autoprefixer for vendor prefixes
- [ ] Dev mode: watch + rebuild on CSS/source changes
- [ ] Production: minified CSS output
