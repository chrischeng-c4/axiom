---
change: prism-rust-symbols
date: 2026-02-06
---

# Clarifications

## Q1: Symbol Scope
- **Question**: Rust symbol extraction 的範圍要涵蓋到哪些 symbol kinds？
- **Answer**: Comprehensive — 所有可提取的 symbols，包括 module declarations、use statements、lifetimes 等
- **Rationale**: Complete coverage ensures Prism can serve as a full Rust code intelligence backend, not just basic navigation

## Q2: Language Coverage
- **Question**: 需要同時實作 TypeScript symbol extraction 嗎？還是先只做 Rust？
- **Answer**: Rust + TypeScript — 一起實作兩種語言的 symbol extraction
- **Rationale**: Both languages already have tree-sitter grammars loaded; implementing together ensures consistent architecture and avoids revisiting the same code paths later

## Q3: Symbol Hierarchy
- **Question**: Symbol extraction 需要支援 scope/hierarchy 嗎？例如 impl block 內的 method 要標記 parent？
- **Answer**: Hierarchical — 支援 parent-child 關係（e.g. struct → impl → methods）
- **Rationale**: Hierarchical symbols enable richer IDE features like outline view, breadcrumbs, and scoped search. This may require extending the existing SymbolTable data model if it currently only supports flat lists

