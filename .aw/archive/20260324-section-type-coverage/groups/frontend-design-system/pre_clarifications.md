---
change: section-type-coverage
group: frontend-design-system
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Where should tech_stack config live?
- **Answer**: tech_stack should NOT be a separate config. SDD infers it from existing project files: Cargo.toml (Rust + framework), package.json (JS/TS + framework + design system e.g. @mui/material, antd), pyproject.toml (Python + framework). No duplication — read what's already there.

### Q2: General
- **Question**: Should the UX pattern library extension point be spec'd now?
- **Answer**: Yes, spec the extension point interface (pattern registry, pattern format) so future implementation has a clear target. Implementation is deferred.

