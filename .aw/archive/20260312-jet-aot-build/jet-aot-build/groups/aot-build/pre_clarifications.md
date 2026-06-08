---
change: jet-aot-build
group: aot-build
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: Minification strategy
- **Answer**: Tree-sitter AST — reuse existing infra, whitespace removal + console.log/debugger drop only. No identifier mangling in this phase.

### Q2: TodoMVC scope
- **Answer**: Mini React stub first — write a minimal createElement/render implementation without npm dependencies. Real React verification deferred to a later phase.

### Q3: Code splitting deferral
- **Answer**: Include in Phase 2 — implement dynamic import() → async chunks alongside tree shaking and minification.

### Q4: CSS handling
- **Answer**: Basic CSS bundling included — resolve @import and concatenate. No CSS Modules, no PostCSS, no Tailwind in this phase.

