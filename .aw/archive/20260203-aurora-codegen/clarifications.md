---
change: aurora-codegen
date: 2026-02-02
---

# Clarifications

## Q1: Output Mode
- **Question**: Code generation 時要生成完整的 project 結構還是只生成單一檔案？
- **Answer**: Full project - 生成完整的 project 結構 (main.py, routes/, models/, etc.)
- **Rationale**: 完整 project 結構更實用，使用者可以直接執行

## Q2: Template Engine
- **Question**: Template engine 要用哪個？
- **Answer**: Tera - Rust-native, Jinja2 相容
- **Rationale**: 專案中已使用 Tera，保持一致性

## Q3: Test Framework
- **Question**: Test generation 要支援哪些 test frameworks？
- **Answer**: 使用 cclab-probe 作為 test generation 的基礎
- **Rationale**: Probe 是專案內部的測試工具，可以更好地整合

## Q4: Git Workflow
- **Question**: Git workflow 偏好？
- **Answer**: New branch - genesis/aurora-codegen
- **Rationale**: 獨立分支方便追蹤變更

---

## Post-Planning Notes

### Bug: Task Generator Duplicate Tasks
- **Issue**: `template-engine` 有 root 和 nested 兩個 spec，導致 Task 2.5/2.6 和 4.7/4.8 重複建立同一檔案
- **Root cause**: Task generator 沒有 deduplicate nested specs
- **Action**: 在此 change 中一起修正 task generator 的 nested spec handling
