---
change: all-jet-issues
group: jet-build-aot
date: 2026-03-18
---

# Requirements

Implement a full AOT production build pipeline in `jet build`, including tree shaking, code splitting, minification, and source maps. Perform true module flattening (Scope Hoisting Phase 2) to achieve a bundle size of ≤ 196KB for `react-bench`. Audit and fix all bundler components (DCE, minifier, mangle, etc.) to ensure correct handling of multi-byte UTF-8 source code by avoiding char-index-as-byte-offset bugs. Validate the implementation against real-world React projects.
