---
number: 81
title: "refactor(titan): Split crud.rs (1190 lines)"
state: open
labels: [enhancement, crate:pg, P0]
---

# #81 — refactor(titan): Split crud.rs (1190 lines)

## Context

cclab-nucleus 是 Python binding hub，用 PyO3 把純 Rust crate 暴露給 Python。目標是讓 binding 代碼可以自動生成（thin wrapper pattern）。

根據專案規範：檔案 ≥1000 行必須拆分。

## Current State

**File**: `crates/cclab-nucleus/src/titan/crud.rs`
**Lines**: 1190 (超過限制)
**Purpose**: PostgreSQL CRUD operations 的 PyO3 binding

這個檔案包含 Titan (PostgreSQL ORM) 的 CRUD 操作綁定，混合了多種操作類型。

## Task

將 `crud.rs` 拆分成多個小檔案，每個檔案 < 500 行。

### Suggested Structure

```
crates/cclab-nucleus/src/titan/
├── crud/
│   ├── mod.rs          # Re-exports + register_crud_module()
│   ├── create.rs       # INSERT operations
│   ├── read.rs         # SELECT operations
│   ├── update.rs       # UPDATE operations
│   ├── delete.rs       # DELETE operations
│   └── bulk.rs         # Bulk operations (如果有)
├── mod.rs              # 更新 import
└── ...
```

### Requirements

1. 保持所有現有的 public API 不變
2. 每個新檔案應該是 thin wrapper（只包裝 cclab-titan crate 的功能）
3. 更新 `titan/mod.rs` 的 imports
4. 確保 `cargo build --features postgres` 編譯通過
5. 保持 `register_module()` 功能正常

### Reference Files

- Pure Rust crate: `crates/cclab-titan/src/`
- Current binding: `crates/cclab-nucleus/src/titan/crud.rs`
- Module entry: `crates/cclab-nucleus/src/titan/mod.rs`
