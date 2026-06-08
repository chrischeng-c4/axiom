---
change: all-issues
group: jet-build-production
date: 2026-03-17
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Custom Tree-sitter AST — build minification on top of existing Tree-sitter parser already used in jet

### Q2: General
- **Answer**: All phases together — implement Phase 1 (Module Concatenation), Phase 2 (Cross-Module DCE), and Phase 3 (Unified Minification) in this change

### Q3: General
- **Answer**: Built-in — integrate PostCSS/Tailwind processing natively in Rust

### Q4: General
- **Answer**: Simple substitution — replace process.env.NODE_ENV and similar with string values, no full Define plugin

