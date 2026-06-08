---
change: cli-to-mcp
date: 2026-02-05
---

# Clarifications

## Q1: Tool Design
- **Question**: MCP tool 的設計模式要怎麼選擇？
- **Answer**: State-aware dispatcher
- **Rationale**: 像 genesis_decide_change，根據 STATE.yaml 返回下一步 action 和 instructions，讓 mainthread 執行實際操作

## Q2: Scope
- **Question**: 要一次遷移三個 CLI 還是分階段？
- **Answer**: 只做 plan-change
- **Rationale**: 先遷移最重要的 plan-change，impl 和 merge 之後再做，降低風險

## Q3: Git Branch
- **Question**: Git workflow 要在哪層處理？
- **Answer**: In place
- **Rationale**: MCP tool 不處理 git branch，由 mainthread/user 決定，保持 MCP tool 簡單

