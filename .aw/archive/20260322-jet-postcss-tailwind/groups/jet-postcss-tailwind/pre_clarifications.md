---
change: jet-postcss-tailwind
group: jet-postcss-tailwind
date: 2026-03-22
status: answered
---

# Pre-Clarifications

### Q1: execution-model
- **Answer**: Native Rust. No Node.js dependency — that defeats the purpose of Jet replacing the JS toolchain. Use lightningcss (Parcel's CSS parser/transformer in Rust) as the foundation. Implement Tailwind utility class generation natively by scanning source files for class names and generating corresponding CSS rules.

### Q2: config-format
- **Answer**: Support both: (1) parse tailwind.config.js via a minimal JS expression evaluator (the config is usually a simple object literal), (2) native jet.config.yaml as alternative. JS config takes precedence for compatibility. For postcss.config.js — don't parse it, Jet replaces PostCSS entirely.

### Q3: js-plugin-execution
- **Answer**: Option (c) — out of scope for JS plugin execution. For tailwindcss-animate: implement the keyframe/animation utilities natively (it's a small set). For @tailwindcss/typography: implement prose class generation natively. These are well-defined CSS output — we don't need to run JS to produce them.

### Q4: phased-delivery
- **Answer**: All in one change. The phases are tightly coupled — Tailwind needs the CSS pipeline, and the pipeline is useless without Tailwind for Conductor. Ship it as one coherent feature.

### Q5: css-minifier
- **Answer**: lightningcss — it handles both parsing, transforming AND minification. Use it as the single CSS engine. Jet already has a JS minifier (tree-sitter based) so CSS minification via lightningcss is the natural parallel.

### Q6: dev-mode-watch-integration
- **Answer**: Jet already has file-watch via the notify crate (dev_server module). Integrate CSS watch into the same watcher — when .css or source files change, trigger CSS rebuild + HMR reload. Single watcher, not separate process.

