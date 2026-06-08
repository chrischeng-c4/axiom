---
change: sdd-unified-frontend
group: sdd-unified-frontend
date: 2026-04-07
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Router choice for @score/app?
- **Answer**: React Router — consistent with Conductor FE which already uses it.

### Q2: General
- **Question**: Should axum API expand to full SddDataSource interface?
- **Answer**: Yes, expand to match SddDataSource. Rename 'specs' to 'tech-design' in the API paths to match the .score/tech_design/ naming convention.

### Q3: General
- **Question**: React build strategy?
- **Answer**: Use cclab-jet to build @score/app. Jet already builds Conductor FE. Build pipeline: `cclab jet build -p @score/app` → dist/ → Rust include_str!() embeds into score binary. Pre-built + committed to avoid requiring Node.js at cargo build time.

