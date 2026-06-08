---
id: nebula-phase2
type: proposal
version: 1
created_at: 2026-02-04T06:48:55.597733+00:00
updated_at: 2026-02-04T06:48:55.597733+00:00
author: mcp
status: proposed
iteration: 1
summary: "将 cclab.nebula 的 aggregation pipeline 与 batched link fetching 核心逻辑迁移到 Rust，并让 Python 层成为 PyO3 薄封装。"
history:
  - timestamp: 2026-02-04T06:48:55.597733+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-04T06:49:04.479908+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-04T06:49:25.879578+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: minor
  affected_files: 9
  new_files: 0
affected_specs:
  - id: aggregation
    path: specs/aggregation.md
    depends: []
  - id: link-fetching
    path: specs/link-fetching.md
    depends: []
  - id: query-builder
    path: specs/query-builder.md
    depends: []---

<proposal>

# Change: nebula-phase2

## Summary

将 cclab.nebula 的 aggregation pipeline 与 batched link fetching 核心逻辑迁移到 Rust，并让 Python 层成为 PyO3 薄封装。

## Why

当前 aggregation 的单值聚合与 link batched fetching 仍由 Python 组装与执行，导致性能与一致性受限，也与“thin wrapper”目标不一致。将 pipeline 构建/执行与批次 link 分发收敛到 Rust，可减少 Python 逻辑与 GIL 负担，并统一安全校验路径。

## What Changes

- 在 `crates/cclab-nebula` 增加/暴露 Rust 侧 aggregation helper（avg/sum/min/max/count 等单值聚合），由 Rust 负责 pipeline 构建与执行，Python 仅传入参数。
- 在 `crates/cclab-nebula` 的 PyO3 bindings 中完善/扩展 batched link fetching 接口（复用现有 `fetch_links_batched`，补齐所需元数据与返回格式），并在 Python `_engine` 添加薄封装调用。
- 在 Python `QueryBuilder` 与 `Document.fetch_all_links` 中移除批次 link 的 Python 实现，改为调用 Rust batched fetch；BackLink 仍沿用 Python 路径（Rust 端暂不支持）。
- 新增链接字段元数据构建与结果回填的轻量逻辑（如 `links.py` 中生成 `LinkField` 列表、按 collection 名解析目标类型、处理 list Link），保证返回仍是 `Link` 对象且行为与现有测试一致。
- 补充/调整 `python/tests/mongo` 下的 fetch_links 与 aggregation 相关用例，以覆盖 Rust 路径与回归行为。

## Impact

- **Scope**: minor
- **Affected Files**: ~9
- **New Files**: ~0
- Affected specs:
  - `aggregation` (no dependencies)
  - `link-fetching` (no dependencies)
  - `query-builder` (no dependencies)
- Affected code: `python/cclab/nebula/_engine.py`, `python/cclab/nebula/query.py`, `python/cclab/nebula/document.py`, `python/cclab/nebula/links.py`, `crates/cclab-nebula/src/aggregation.rs`, `crates/cclab-nebula/src/pyo3_bindings/document.rs`, `crates/cclab-nebula/src/pyo3_bindings/link.rs`, `crates/cclab-nebula/src/pyo3_bindings/mod.rs`, `python/tests/mongo/unit/test_relations.py`
- **Breaking Changes**: 无预期对外 API 破坏；内部实现切换为 Rust。

</proposal>
