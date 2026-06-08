---
change: 178-grid-db-refactor
date: 2026-02-05
---

# Clarifications

## Q1: CRDT 處理
- **Question**: CRDT 模組如何處理？
- **Answer**: 重構為 yrs snapshot storage - 保留模組但改為儲存 yrs update/snapshot，不自己實作 CRDT 邏輯
- **Rationale**: yrs 已經是成熟的 CRDT library，grid-server 已在使用。保留模組結構但改變用途，避免重複實作且維持程式碼組織性

## Q2: WAL 架構
- **Question**: WAL (Write-Ahead Log) 實作方式？
- **Answer**: 抽出共用 cclab-wal crate，ion 和 grid-db 都依賴它
- **Rationale**: WAL 是通用的持久化機制，建立共用 crate 可以避免重複實作，並讓不同儲存引擎共享經過驗證的 WAL 實作

## Q3: 重構策略
- **Question**: 重構策略？
- **Answer**: 一次到位 - 一次完成所有重構
- **Rationale**: grid-db 目前幾乎全是 stub，沒有需要維護的生產程式碼，可以直接重構到目標架構

## Q4: Git Flow
- **Question**: Git 工作流程？
- **Answer**: 原地開發 - 留在當前分支 (tslibs)
- **Rationale**: 這是架構重構，與當前開發分支一致，不需要獨立分支

