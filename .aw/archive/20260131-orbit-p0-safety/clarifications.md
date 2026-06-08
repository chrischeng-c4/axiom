---
change: orbit-p0-safety
date: 2026-01-31
---

# Clarifications

## Q1: Focus Area
- **Question**: P0 有 16 個 issues，你想先聚焦在哪個類別？
- **Answer**: Safety First - #101 unsafe audit, #102 Send+Sync, #108 error handling, #109 graceful shutdown
- **Rationale**: Safety is foundational. Auditing unsafe code and ensuring thread safety must be done before adding new features.

## Q2: Git Flow
- **Question**: 你想用什麼 git workflow？
- **Answer**: In place - stay on current branch (cclab-orbit)
- **Rationale**: User prefers working directly on the current branch without creating new branches or worktrees.

## Q3: PyO3 Strategy
- **Question**: 目前 orbit 的 PyO3 bindings 要怎麼處理？
- **Answer**: Keep temporarily - 保留現有 bindings，先強化 Rust core，之後再移除
- **Rationale**: Incremental approach: strengthen the Rust core first, then refactor bindings later. This reduces risk and allows gradual migration.

