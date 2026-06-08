---
change: lens-comprehensive
group: symbol-tables
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: JavaScript symbol depth
- **Answer**: Reuse existing TypeScript symbol extractor. JS is a subset of TS, so TS extractor handles JS files directly.

### Q2: K8s cross-file references
- **Answer**: Support cross-file references (e.g. Service selector → Deployment labels). Requires project-wide index via daemon.

