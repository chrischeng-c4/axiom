---
change: jet-aot-build-gaps
group: jet-aot-build-gaps
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Image optimization approach
- **Answer**: Use existing `image` crate, enhance it with actual compression (resize, quality reduction, format-specific optimizations). No new dependencies.

### Q2: General
- **Question**: HTML minification approach
- **Answer**: Custom minimal implementation — jet currently has no HTML minification at all (only dev server HTML serving). Build a simple whitespace/comment removal pass, no new dependency.

### Q3: General
- **Question**: Manual chunks config API shape
- **Answer**: Jet-native simple object format: name → glob patterns mapping. Not Vite-compatible.

