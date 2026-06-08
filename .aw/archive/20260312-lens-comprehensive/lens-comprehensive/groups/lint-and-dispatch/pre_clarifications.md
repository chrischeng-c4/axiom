---
change: lens-comprehensive
group: lint-and-dispatch
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: K8s schema size
- **Answer**: Bundle all 3 versions (1.28, 1.29, 1.30). Binary size increase acceptable for full version support.

### Q2: Python security rules scope
- **Answer**: Default to Warning severity. Users can override to Error via config.

### Q3: GL006 cycle detection
- **Answer**: Report full cycle path (A→B→C→A) for maximum debugging clarity.

