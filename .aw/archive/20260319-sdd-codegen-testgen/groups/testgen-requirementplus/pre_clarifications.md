---
change: sdd-codegen-testgen
group: testgen-requirementplus
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: Standalone CLI: `cclab sdd gen test <spec-path>`. Can also be called internally during implement phase.

### Q2: General
- **Answer**: Comments + todo!() only. Safe default: `// Then: <text>` + `todo!("implement")`. No NLP heuristics.

### Q3: General
- **Answer**: Warnings by default. Add `--strict` flag to make uncovered requirements (no Verifies relationship) into hard errors for CI gating.

### Q4: General
- **Answer**: Defer cclab-probe integration to a follow-up issue. This change focuses on test scaffold generation + coverage validation only.

