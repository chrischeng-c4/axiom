---
number: 85
title: "refactor(nebula): Split document.rs (863 lines)"
state: open
labels: [enhancement, crate:mongo, P1]
---

# #85 — refactor(nebula): Split document.rs (863 lines)

## Context

cclab-nucleus 是 Python binding hub，用 PyO3 把純 Rust crate 暴露給 Python。目標是讓 binding 代碼可以自動生成（thin wrapper pattern）。

根據專案規範：檔案 ≥500 行建議拆分。

## Current State

**File**: `crates/cclab-nucleus/src/nebula/document.rs`
**Lines**: 863 (超過建議值)
**Purpose**: MongoDB Document 的 PyO3 binding，包裝 cclab-nebula crate

這是 `RustDocument` class 的主要實現，包含 CRUD、查詢、聚合等操作。

## Task

將 `document.rs` 拆分成多個小檔案，每個檔案 < 400 行。

### Suggested Structure

```
crates/cclab-nucleus/src/nebula/
├── document/
│   ├── mod.rs          # Re-exports RustDocument + register
│   ├── core.rs         # RustDocument struct + basic methods
│   ├── crud.rs         # save(), delete(), reload()
│   ├── query.rs        # find(), find_one(), count()
│   ├── aggregate.rs    # aggregate(), pipeline operations
│   └── bulk.rs         # bulk_write(), bulk operations
├── mod.rs              # 更新 import
└── ...
```

### Requirements

1. 保持 `RustDocument` class 的所有 public API 不變
2. 每個新檔案應該是 thin wrapper（只包裝 cclab-nebula crate 的功能）
3. 更新 `nebula/mod.rs` 的 imports
4. 確保 `cargo build --features mongodb` 編譯通過
5. 保持 `register_module()` 功能正常

### Reference Files

- Pure Rust crate: `crates/cclab-nebula/src/`
- Current binding: `crates/cclab-nucleus/src/nebula/document.rs`
- Module entry: `crates/cclab-nucleus/src/nebula/mod.rs`
- Conversion utils: `crates/cclab-nucleus/src/nebula/conversion.rs`
