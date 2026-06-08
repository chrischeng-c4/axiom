---
change: jet-hmr-validation
group: jet-hmr-validation
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: import.meta.hot API scope?
- **Answer**: Full Vite-compatible API: accept, dispose, prune, invalidate, data. Enables third-party HMR plugins and full parity with Vite dev experience.

### Q2: General
- **Question**: React Fast Refresh injection method?
- **Answer**: AST-based in transform_tsx.rs. Inject $RefreshReg$ and $RefreshSig$ calls by analyzing the AST to find React component declarations and hooks usage.

