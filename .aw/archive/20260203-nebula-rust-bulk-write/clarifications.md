---
change: nebula-rust-bulk-write
date: 2026-01-31
---

# Clarifications

## Q1: API 相容性
- **Question**: Python API 應該維持向後相容還是可以做破壞性變更？
- **Answer**: 簡化 API - 允許破壞性變更，採用更簡潔的 API 設計
- **Rationale**: 這是 Phase 1 的重構，趁機會簡化 API 設計，減少不必要的複雜度。用戶可以接受 migration 成本。

## Q2: Rust 結構
- **Question**: Rust 端的 BulkOperation 該如何設計？
- **Answer**: Enum - enum BulkOp { UpdateOne{..}, InsertOne{..}, DeleteOne{..} }
- **Rationale**: Enum 提供完整的類型安全和 pattern matching，是 Rust 最佳實踐。編譯時期就能捕捉到錯誤的操作類型。

## Q3: PyO3 策略
- **Question**: PyO3 轉換策略？
- **Answer**: FromPyObject trait - 實作 FromPyObject 讓 Python dict 自動轉換為 Rust struct
- **Rationale**: FromPyObject 是 PyO3 推薦的方式，可以讓 Python 端傳入簡單的 dict，Rust 端自動驗證和轉換，減少手動解析的樣板程式碼。

## Q4: Git 流程
- **Question**: Git 工作流程偏好？
- **Answer**: In place - 在目前分支直接開發
- **Rationale**: 用戶偏好簡單的工作流程，避免 branch 切換的額外複雜度。

