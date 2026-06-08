---
change: sdd-codegen-and-fixes
group: new-spec-types
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Deploy codegen first target
- **Answer**: k8s Deployment + Service manifest as the first codegen target. Production-grade output.

### Q2: General
- **Question**: Deploy section validation blocking behavior
- **Answer**: Soft warning — warn on inconsistencies but don't block spec creation. Cross-ref validation to db-model and rest-api deferred to later iteration.

### Q3: General
- **Question**: Frontend framework target for wireframe codegen
- **Answer**: React as the first target framework.

### Q4: General
- **Question**: Scope of frontend cross-section composition
- **Answer**: Full scope — all 3 phases in this change: wireframe to React scaffold, DTCG design tokens to CSS/Tailwind, CEM to TypeScript interface + component skeleton. Cross-ref to rest-api for data fetching hooks included.

### Q5: General
- **Question**: Shared validator infrastructure
- **Answer**: Introduce a unified validator registration mechanism shared by both deploy and frontend section types.

