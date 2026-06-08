---
change: project-config-and-prism-index
date: 2026-02-10
---

# Clarifications

## Q1: Language field design
- **Question**: Monorepo language 欄位的設計方向？例如 infohub 根目錄有 api/(Python), frontend/(TS)。
- **Answer**: Path mapping: [[project.modules]] path="api/" language="python" — 每個子目錄對應語言
- **Rationale**: Monorepo 需要 per-directory language mapping，flat list 無法區分哪個目錄用什麼語言。Path mapping 讓 task generator 和 Prism 都能根據檔案路徑判斷正確語言。

## Q2: Prism index key
- **Question**: Prism index 搬到 ~/.cclab/projects/ 的 key 用什麼？
- **Answer**: 用 path hash (from root or from home)，支援多個 index（monorepo 可能有多個語言各自的 index）
- **Rationale**: Path hash 避免命名衝突，monorepo 可能需要多個 index 分別對應不同 module。Hash 從 project root 或 home 計算都可以。

## Q3: Git workflow
- **Question**: Git workflow 要用哪種？
- **Answer**: in_place — 直接在 main 上改
- **Rationale**: 直接在 main 上開發，符合目前的工作模式。

