---
change: all-jet-issues
group: jet-build-aot
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Custom Tree-sitter AST — already implemented in mangle.rs/fold.rs/dce.rs, continue building on this approach

### Q2: General
- **Answer**: ESM first — most npm packages ship ESM, CJS modules (like React) should fall back to wrapper-based approach. Focus on ESM-only modules for flattening

### Q3: General
- **Answer**: Known limitations: CSS modules not fully supported, dynamic import() splits not verified end-to-end, no HMR in production builds. Start with TodoMVC (mini-react) and react-bench as validation targets

