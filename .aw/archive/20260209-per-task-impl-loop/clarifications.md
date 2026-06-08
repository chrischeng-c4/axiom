---
change: per-task-impl-loop
date: 2026-02-09
---

# Clarifications

## Q1: Per-task review depth
- **Question**: Per-task review 要用什麼等級？輕量 auto-review 還是完整 code review？
- **Answer**: Full review per task，但要防止無限 review loop
- **Rationale**: 每個 task 都做完整 code review 確保品質，但需要 revision limit 機制（跟 spec review 一樣 max 2 revisions）避免無限 loop

## Q2: Git workflow
- **Question**: Git workflow 要用哪種？
- **Answer**: in_place — 直接在 main branch 上做
- **Rationale**: This is an internal refactor of the genesis workflow engine itself

