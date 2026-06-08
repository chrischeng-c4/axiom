---
change: all-open-jet-issues
group: jet-build-aot-production
date: 2026-03-18
status: answered
---

# Pre-Clarifications

### Q1: Minifier Strategy
- **Answer**: Extend the existing Tree-sitter infrastructure (Option A) to maintain consistency and full control over transformations for the first production release, as suggested in #765.

### Q2: Side Effects Analysis
- **Answer**: Follow the sideEffects field strictly in accordance with standard npm/bundler conventions to enable aggressive tree shaking, as this is standard practice in libraries like React and MUI.

### Q3: CSS Pipeline Depth
- **Answer**: Standard CSS bundling and CSS Modules are the priority for the first release. PostCSS integration should be architected to be pluggable but can be finalized once core bundling is stable.

### Q4: Source Map Defaults
- **Answer**: Disable source maps by default in production builds, but provide a --sourcemap flag (or similar config) to enable them manually.

