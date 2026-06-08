---
change: ion-blpop
date: 2026-01-31
---

# Clarifications

## Q1: Architecture
- **Question**: BLPOP 應該在哪一層實現？
- **Answer**: Server 層 (cclab-ion-server)
- **Rationale**: KvEngine 保持 sync API，BLPOP 的等待邏輯由 async server 層處理，使用 tokio::sync::Notify 管理 waiting clients

## Q2: Key Priority
- **Question**: 多個 key 時如何決定優先順序？
- **Answer**: 按參數順序
- **Rationale**: 依 keys 陣列順序檢查，與 Redis 行為一致

## Q3: Git Workflow
- **Question**: Git workflow 偏好？
- **Answer**: In place
- **Rationale**: 留在當前 branch (cclab-ion)

## Q4: Scope
- **Question**: 實現範圍包含哪些？
- **Answer**: Protocol 新增 6 個 commands (LPush, RPush, LPop, RPop, BLPop, BRPop)，Server 層處理 + 等待隊列，Client 新增對應 async methods
- **Rationale**: KvEngine 已有 lpush/rpush/lpop/rpop，無需修改核心引擎

