---
change: mamab-p0-issues
date: 2026-03-03
---

# Spec Clarifications

## Questions

### Q1: General
- **Question**: 
- **Answer**: 1) #629 (EXPECT-ERROR harness) first — unblocks negative tests (#566). 2) Edge-case fixtures (#550-#576) — these are self-contained and can be written immediately. 3) CPython imports (#510-#519) — require downloading and extracting from CPython source.
- **Rationale**: 

### Q2: General
- **Question**: 
- **Answer**: Yes, all 30 issues are independent (no DAG dependencies). Only #566 (negative tests) depends on #629 (harness enhancement) being done first.
- **Rationale**: 

### Q3: General
- **Question**: 
- **Answer**: Each edge-case fixture should have 20-50 test cases covering the syntax construct thoroughly. Aim for ~100-300 lines per fixture file. Stay under 500 lines per file.
- **Rationale**: 

