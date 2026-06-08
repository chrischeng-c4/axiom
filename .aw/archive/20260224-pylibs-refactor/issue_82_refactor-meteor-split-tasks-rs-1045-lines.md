---
number: 82
title: "refactor(meteor): Split tasks.rs (1045 lines)"
state: open
labels: [enhancement, crate:queue, P0]
---

# #82 — refactor(meteor): Split tasks.rs (1045 lines)

## Context

cclab-nucleus 是 Python binding hub，用 PyO3 把純 Rust crate 暴露給 Python。目標是讓 binding 代碼可以自動生成（thin wrapper pattern）。

根據專案規範：檔案 ≥1000 行必須拆分。

## Current State

**File**: `crates/cclab-nucleus/src/tasks.rs`
**Lines**: 1045 (超過限制)
**Purpose**: Task queue (Celery-like) 的 PyO3 binding，包裝 cclab-meteor crate

這個檔案包含多個 PyO3 class：Task, Chain, Group, Chord, Signature 等。

## Task

將 `tasks.rs` 拆分成多個小檔案，每個檔案 < 500 行。

### Suggested Structure

```
crates/cclab-nucleus/src/
├── tasks/
│   ├── mod.rs          # Re-exports + register_module()
│   ├── task.rs         # Task class (#[pyclass])
│   ├── chain.rs        # Chain class
│   ├── group.rs        # Group class  
│   ├── chord.rs        # Chord class
│   └── signature.rs    # Signature handling
└── lib.rs              # 更新 mod tasks → mod tasks (directory)
```

### Requirements

1. 保持所有現有的 public API 不變
2. 每個新檔案應該是 thin wrapper（只包裝 cclab-meteor crate 的功能）
3. 更新 `lib.rs` 的 module 聲明
4. 確保 `cargo build --features tasks` 編譯通過
5. 保持 `register_module()` 功能正常

### Reference Files

- Pure Rust crate: `crates/cclab-meteor/src/`
- Current binding: `crates/cclab-nucleus/src/tasks.rs`
- Main entry: `crates/cclab-nucleus/src/lib.rs`
