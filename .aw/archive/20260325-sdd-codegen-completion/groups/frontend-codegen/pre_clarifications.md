---
change: sdd-codegen-completion
group: frontend-codegen
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Primary frontend framework target
- **Answer**: React (TSX). ReactGenerator already partially implemented from previous change.

### Q2: General
- **Question**: Wireframe YAML DSL schema source
- **Answer**: Use the wireframe YAML DSL from #897's proposal. WireframeSpec already defined in SpecIR types.

### Q3: General
- **Question**: Design-token output priority
- **Answer**: Both CSS custom properties and Tailwind config from a single pass. CSS custom properties first, then Tailwind config object.

