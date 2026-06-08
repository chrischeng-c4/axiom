---
change: toolchain-reorg
date: 2026-01-28
---

# Clarifications

## Q1: Env Tools
- **Question**: 關於 Environment tools (5個)，你想要怎麼處理？
- **Answer**: 移除 from list - 保持 intentionally excluded，不暴露這些工具
- **Rationale**: 這些工具會修改 pyproject.toml，有安全考量，且目前沒有使用需求

## Q2: Git Workflow
- **Question**: 要使用哪種 Git workflow 進行這個 change？
- **Answer**: In place - 留在目前的 branch (genesis/mermaid-plus)
- **Rationale**: 這個 change 是 mermaid-plus 的延續，邏輯上相關

## Q3: Scope
- **Question**: 這個 change 的範圍要多大？
- **Answer**: Phase 1-2 only - 先修 router + 加 state machine tools
- **Rationale**: 聚焦核心問題，降低風險。後續 phases 可以另開 change

## Q4: Shared Crate
- **Question**: 是否建立 cclab-mermaid 共享 crate？
- **Answer**: 不建立 - 直接用 Prism API
- **Rationale**: Codex review 指出 Mermaid parser 依賴 Prism IR，建立 shared crate 會造成循環依賴

## Q5: Router Design
- **Question**: Router 如何處理不同類型的 Prism tools？
- **Answer**: 分三條路徑：Daemon (9) / MCP Handler (3+2) / 移除 (5)
- **Rationale**: Daemon 只支援分析工具，spec-gen 和 state machine 需要走 MCP handler

