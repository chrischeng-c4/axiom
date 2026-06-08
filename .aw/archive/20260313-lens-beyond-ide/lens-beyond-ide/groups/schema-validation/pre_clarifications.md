---
change: lens-beyond-ide
group: schema-validation
date: 2026-03-13
status: answered
---

# Pre-Clarifications

### Q1: Schema size
- **Answer**: Full API schemas (~15MB total). Embed raw JSON via include_bytes!() for fastest startup. Binary size increase is acceptable.

### Q2: Schema scope
- **Answer**: Full K8s API schemas — all resource types, not a subset.

