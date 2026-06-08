---
change: nebula-rust-aggregate
date: 2026-01-31
---

# Clarifications

## Q1: Scope
- **Question**: 移植範圍：最小可行 vs 完整重構？
- **Answer**: 完整重構 - 將 pipeline 建構移到 Rust，Python 變成 thin wrapper
- **Rationale**: 與 Issue #78 的整體目標一致，最大化效能提升

## Q2: API Style
- **Question**: AggregationBuilder API 設計偏好？
- **Answer**: Fluent Builder - builder.match_stage(filter).group(field, op).build() 鏈式呼叫
- **Rationale**: 更符合 Rust idiom，提供更好的 IDE 支援和錯誤檢查

## Q3: Stages
- **Question**: Pipeline stage 支援範圍？
- **Answer**: $match + $group only - 覆蓋 Issue #80 的需求
- **Rationale**: 簡單快速，先滿足核心需求，後續可擴展

## Q4: Git
- **Question**: Git workflow 偏好？
- **Answer**: In place - 繼續在當前分支開發
- **Rationale**: 與 nebula-rust-bulk-write 使用相同模式

