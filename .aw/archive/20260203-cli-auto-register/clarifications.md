---
change: cli-auto-register
date: 2026-01-31
---

# Clarifications

## Q1: Migration Strategy
- **Question**: 如何處理現有的 CLI commands？要一次全部遷移還是漸進式？
- **Answer**: 漸進式遷移
- **Rationale**: 先建立機制，保留舊 pattern 相容，逐步遷移各 crate，降低風險

## Q2: Macro Crate
- **Question**: 是否需要自定義 proc macro crate？
- **Answer**: 不需要，直接用 linkme
- **Rationale**: 各 crate 直接寫 #[distributed_slice] 註冊即可，簡單明確，不需要額外的 proc macro 包裝

## Q3: Collection Mechanism
- **Question**: 選擇 distributed slice 的實現方式？
- **Answer**: linkme
- **Rationale**: 較新、維護積極、支援更多平台，比 inventory 更活躍

## Q4: Scope
- **Question**: MCP tools 和 CLI 是否都要改？
- **Answer**: 只改 CLI
- **Rationale**: MCP tools 維持現有 list() pattern，只將 CLI 改用 linkme 自動收集

