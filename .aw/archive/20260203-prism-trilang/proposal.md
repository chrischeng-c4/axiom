---
id: prism-trilang
type: proposal
version: 1
created_at: 2026-01-31T10:43:19.342531+00:00
updated_at: 2026-01-31T10:43:19.342531+00:00
author: mcp
status: proposed
iteration: 1
summary: "強化 prism 對 Python/TypeScript/Rust 三語言的一致性支援，實現深度語義分析與重構。"
history:
  - timestamp: 2026-01-31T10:43:19.342531+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-31T10:43:27.918952+00:00
    agent: "codex:deep"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-01-31T10:43:43.971071+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: major
  affected_files: 15
  new_files: 0
affected_specs:
  - id: rust-type-system
    path: specs/rust-type-system.md
    depends: []
  - id: typescript-inference
    path: specs/typescript-inference.md
    depends: []
  - id: unified-semantic-search
    path: specs/unified-semantic-search.md
    depends: [rust-type-system, typescript-inference]
  - id: unified-refactoring-engine
    path: specs/unified-refactoring-engine.md
    depends: [rust-type-system, typescript-inference, unified-semantic-search]
  - id: rust-type-system-spec
    path: specs/rust-type-system-spec.md
    depends: []
  - id: typescript-inference-spec
    path: specs/typescript-inference-spec.md
    depends: []
  - id: unified-semantic-search-spec
    path: specs/unified-semantic-search-spec.md
    depends: [rust-type-system-spec, typescript-inference-spec]
  - id: unified-refactoring-engine-spec
    path: specs/unified-refactoring-engine-spec.md
    depends: [rust-type-system-spec, typescript-inference-spec, unified-semantic-search-spec]---

<proposal>

# Change: prism-trilang

## Summary

強化 prism 對 Python/TypeScript/Rust 三語言的一致性支援，實現深度語義分析與重構。

## Why

目前 Prism 的高級語義分析主要集中在 Python。Rust 和 TypeScript 的支持僅限於基礎解析。為了提供統一代碼分析體驗，Prism 需要對所有支持的語言進行深層語義理解。優先級：Rust > Python > TS。Rust 優先強化工具鏈以實現 dogfooding。所有實現均獨立完成，不依賴 rust-analyzer/pyright，確保行為一致性。

## What Changes

- 實現 Rust 完整類型系統：包括 Trait 解析、Lifetime 分析及符號表擴展。
- 強化 TypeScript 類型推斷：支持複雜泛型、聯合/交叉類型、字面量類型及結構化子類型。
- 統一三語言語義搜索 API：實現跨文件的 Call Hierarchy、Usage Search 及類型特徵搜索。
- 統一三語言重構引擎：支持 Rename、Extract Function/Variable 等核心重構操作，並具備語義校驗能力。

## Impact

- **Scope**: major
- **Affected Files**: ~15
- **New Files**: ~0
- Affected specs:
  - `rust-type-system` (no dependencies)
  - `typescript-inference` (no dependencies)
  - `unified-semantic-search` → depends on: `rust-type-system`, `typescript-inference`
  - `unified-refactoring-engine` → depends on: `rust-type-system`, `typescript-inference`, `unified-semantic-search`
  - `rust-type-system-spec` (no dependencies)
  - `typescript-inference-spec` (no dependencies)
  - `unified-semantic-search-spec` → depends on: `rust-type-system-spec`, `typescript-inference-spec`
  - `unified-refactoring-engine-spec` → depends on: `rust-type-system-spec`, `typescript-inference-spec`, `unified-semantic-search-spec`

</proposal>
