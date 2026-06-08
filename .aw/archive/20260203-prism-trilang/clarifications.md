---
change: prism-trilang
date: 2026-01-31
---

# Clarifications

## Q1: Language Priority
- **Question**: 三語言的優先級順序是什麼？這決定功能開發的先後順序。
- **Answer**: Rust > Python > TS
- **Rationale**: Rust 優先強化自己的工具鏈，prism 本身是 Rust 寫的，優先支援 Rust 有 dogfooding 效益

## Q2: Rust Type Depth
- **Question**: Rust 類型推斷要做到什麼程度？
- **Answer**: 完整類型系統
- **Rationale**: 包含 trait 解析、lifetime 分析，這是與 rust-analyzer 競爭的關鍵差異化

## Q3: External Integration
- **Question**: 是否需要整合外部工具？
- **Answer**: 獨立實現
- **Rationale**: 不依賴 rust-analyzer/pyright，自己實現確保一致的 API 和跨語言行為

## Q4: Git Workflow
- **Question**: Git workflow 偏好？
- **Answer**: In place
- **Rationale**: 在當前分支工作，避免 worktree 管理複雜度

