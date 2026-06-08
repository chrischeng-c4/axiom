---
change: jet-remaining-issues
group: jet-build-aot-pipeline
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Custom Tree-sitter AST — already implemented in mangle.rs/fold.rs/dce.rs/minify.rs. Keep building on this, no new dependencies.

### Q2: General
- **Answer**: Focus on what's NOT done yet: CSS pipeline (#765 R5), asset pipeline (#765 R6), real-world validation (#797). Tree shaking, minification, source maps, code splitting already implemented.

### Q3: General
- **Answer**: Automated CI tests — clone fixtures, run jet build, assert page loads via Playwright. Blocking failures for core functionality, non-blocking for known limitations.

