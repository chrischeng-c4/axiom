---
change: nebula-rust-link-fetch
date: 2026-02-01
---

# Clarifications

## Q1: PyObject
- **Question**: Python 物件操作策略？
- **Answer**: Rust 透過 PyO3 提供完整物件，Python 只是 thin wrapper
- **Rationale**: 與整體 thin wrapper 架構一致，最大化 Rust 端處理

## Q2: Depth
- **Question**: 遞迴深度處理？
- **Answer**: 支援多層遞迴 - 完整支援 depth > 1
- **Rationale**: 完整功能，避免後續重工

## Q3: Scope
- **Question**: 實作範圍？
- **Answer**: Full pipeline - 包含 ref 收集 + batch query + 分配回 document
- **Rationale**: 完整實作整個 link fetching 流程

