---
number: 135
title: "feat(titan): 補齊關鍵測試覆蓋以達到 asyncpg + SQLAlchemy 替代標準"
state: open
labels: [crate:pg, P0]
---

# #135 — feat(titan): 補齊關鍵測試覆蓋以達到 asyncpg + SQLAlchemy 替代標準

## 背景

cclab-titan 目標是取代 Python 生態中的 asyncpg + SQLAlchemy。目前測試覆蓋約 310 個測試，在 query building、security、transactions 等核心功能有良好覆蓋，但有幾個關鍵領域完全沒有測試。

## 現況分析

### ✅ 測試覆蓋良好的領域

| 類別 | 測試數 | 狀態 |
|------|--------|------|
| Query Building (SELECT/INSERT/UPDATE/DELETE) | 87 | ✅ 良好 |
| Security / SQL Injection Prevention | 29 | ✅ 優秀 |
| Transactions + Savepoints + Isolation Levels | 15 | ✅ 優秀 |
| Migrations | 12 | ✅ 優秀 |
| Schema Introspection | 7 | ✅ 良好 |
| Type Mapping | 25 | ✅ 良好 |
| Fuzzing | 25 | ✅ 良好 |

### ❌ 關鍵缺失（阻擋 production-ready）

| 類別 | 測試數 | 問題 |
|------|--------|------|
| **Connection Pool Lifecycle** | 0 | 完全沒有測試 |
| **Prepared Statement Caching** | 0 | 完全沒有測試 |
| **Constraint Violation Errors** | 0 | 完全沒有測試 |
| **Cascade Operations** | 0 | 完全沒有測試 |
| Relationships / Eager Loading | 5 | 不足 |
| Error Handling | 15 | 部分 |

---

## P0 - 必須補齊的測試

### 1. Connection Pool Lifecycle (~20 tests)

asyncpg 的 `asyncpg.create_pool()` 提供完整的 pool 管理。我們需要測試：

```rust
// 需要新增的測試案例
#[tokio::test]
async fn test_pool_min_max_connections() { }

#[tokio::test]
async fn test_pool_overflow_queue_behavior() { }

#[tokio::test]
async fn test_pool_connection_timeout() { }

#[tokio::test]
async fn test_pool_health_check_pruning() { }

#[tokio::test]
async fn test_pool_exhaustion_under_load() { }

#[tokio::test]
async fn test_pool_connection_leak_detection() { }

#[tokio::test]
async fn test_pool_graceful_shutdown() { }

#[tokio::test]
async fn test_pool_reconnect_on_connection_drop() { }
```

**驗證項目：**
- [ ] min_connections 參數正確運作
- [ ] max_connections 限制有效
- [ ] 超過 max 時排隊或拒絕的行為
- [ ] idle timeout 後連線被回收
- [ ] connection max_lifetime 後強制關閉
- [ ] health check 失敗時移除壞連線
- [ ] pool shutdown 時所有連線正確關閉

---

### 2. Constraint Violation Error Handling (~20 tests)

SQLAlchemy 提供豐富的 exception hierarchy。我們需要測試錯誤處理：

```rust
// 需要新增的測試案例
#[tokio::test]
async fn test_unique_constraint_violation() { }

#[tokio::test]
async fn test_not_null_constraint_violation() { }

#[tokio::test]
async fn test_check_constraint_violation() { }

#[tokio::test]
async fn test_foreign_key_constraint_violation() { }

#[tokio::test]
async fn test_primary_key_violation() { }

#[tokio::test]
async fn test_error_message_extraction() { }

#[tokio::test]
async fn test_constraint_name_in_error() { }
```

**驗證項目：**
- [ ] UNIQUE violation 返回正確錯誤類型
- [ ] NOT NULL violation 包含 column 名稱
- [ ] CHECK constraint 包含 constraint 名稱
- [ ] FK violation 包含 table 和 column 資訊
- [ ] 錯誤訊息可以被 parse 取得結構化資訊
- [ ] Python 端收到對應的 exception 類型

---

### 3. Cascade Operations (~15 tests)

```rust
// 需要新增的測試案例
#[tokio::test]
async fn test_on_delete_cascade() { }

#[tokio::test]
async fn test_on_delete_set_null() { }

#[tokio::test]
async fn test_on_delete_set_default() { }

#[tokio::test]
async fn test_on_delete_restrict() { }

#[tokio::test]
async fn test_on_update_cascade() { }

#[tokio::test]
async fn test_cascade_with_multiple_fks() { }

#[tokio::test]
async fn test_cascade_depth_limit() { }
```

**驗證項目：**
- [ ] DELETE parent 時 child 正確被刪除 (CASCADE)
- [ ] DELETE parent 時 child FK 設為 NULL (SET NULL)
- [ ] DELETE parent 時有 child 會被阻擋 (RESTRICT)
- [ ] UPDATE PK 時 FK 正確更新 (CASCADE)
- [ ] 多層 cascade 正確傳播
- [ ] cascade 操作在 transaction 中正確 rollback

---

### 4. UPSERT 和進階查詢 (~15 tests)

```rust
// 需要新增的測試案例
#[test]
fn test_upsert_on_conflict_do_nothing() { }

#[test]
fn test_upsert_on_conflict_do_update() { }

#[test]
fn test_upsert_with_where_clause() { }

#[test]
fn test_returning_clause_insert() { }

#[test]
fn test_returning_clause_update() { }

#[test]
fn test_returning_clause_delete() { }

#[test]
fn test_recursive_cte() { }
```

**驗證項目：**
- [ ] INSERT...ON CONFLICT DO NOTHING
- [ ] INSERT...ON CONFLICT DO UPDATE SET
- [ ] RETURNING * 返回插入的資料
- [ ] RETURNING specific columns
- [ ] Recursive CTE 語法正確

---

## P1 - 重要但非阻擋

### 5. Concurrency & Locking (~10 tests)

```rust
#[tokio::test]
async fn test_select_for_update() { }

#[tokio::test]
async fn test_deadlock_detection() { }

#[tokio::test]
async fn test_serializable_conflict() { }
```

### 6. Bulk Operations Performance (~10 tests)

```rust
#[tokio::test]
async fn test_bulk_insert_1k_rows() { }

#[tokio::test]
async fn test_bulk_insert_10k_rows() { }

#[tokio::test]
async fn test_bulk_update_batching() { }
```

### 7. Error Recovery (~10 tests)

```rust
#[tokio::test]
async fn test_connection_retry_on_timeout() { }

#[tokio::test]
async fn test_transaction_retry_on_serialization_failure() { }
```

---

## 實作建議

### 測試檔案結構

```
crates/cclab-titan/tests/
├── test_row_crud.rs          # 現有
├── test_transaction.rs       # 現有
├── test_migration.rs         # 現有
├── test_schema.rs            # 現有
├── test_security.rs          # 現有
├── test_pool.rs              # 🆕 P0
├── test_constraints.rs       # 🆕 P0
├── test_cascade.rs           # 🆕 P0
├── test_upsert.rs            # 🆕 P0
├── test_concurrency.rs       # 🆕 P1
└── test_bulk.rs              # 🆕 P1
```

### 估算工時

| Priority | 測試數 | 預估時間 |
|----------|--------|----------|
| P0 | 70 tests | 10-13 天 |
| P1 | 30 tests | 5-7 天 |
| **Total** | **100 tests** | **15-20 天** |

---

## 驗收標準

- [ ] 所有 P0 測試通過
- [ ] 測試覆蓋率達到 80%+
- [ ] Python binding 測試可從 Rust 測試自動萃取
- [ ] CI 包含所有新測試

---

## 相關

- 此 issue 來自 prism pyo3 binding 自動生成的測試萃取分析
- Python 測試將使用 `cclab prism gen-pyo3-test` 從 Rust 測試萃取
