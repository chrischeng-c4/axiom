---
change: fix
group: jet-bundler-fixes
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: Dynamic import strategy
- **Answer**: Eager resolution: transform import('./path') to Promise.resolve(__jet__.require(id)). Simple and sufficient for now, no runtime changes needed.

