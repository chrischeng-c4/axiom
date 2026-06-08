---
change: e2e-test-reorg
group: e2e-test-reorg
date: 2026-03-26
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Directory structure: e2e/grid/ for RuSheet tests, e2e/jet/ for jet tests?
- **Answer**: Yes. Move existing e2e/*.spec.ts into e2e/grid/, move examples/mini-react/ into e2e/jet/. examples/jet/ and examples/react-bench/ stay.

### Q2: General
- **Question**: Playwright projects: vite-build, jet-build, jet-dev with testMatch filters?
- **Answer**: Yes. vite-build and jet-build run build.spec.ts only. jet-dev runs dev-server.spec.ts, hmr.spec.ts, css.spec.ts.

