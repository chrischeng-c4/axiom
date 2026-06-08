---
change: nova-analyst-agent
date: 2026-01-31
---

# Clarifications

## Q1: Output Format
- **Question**: AnalystAgent 的主要輸出格式是什麼？
- **Answer**: Markdown 文件
- **Rationale**: 輸出 requirements.md、questions.md 等標準文件，方便人類閱讀和版本控制

## Q2: Core Tools
- **Question**: Base AnalystAgent 需要哪些核心 tools？
- **Answer**: ask_user + read_file + take_note + web_search + web_fetch
- **Rationale**: 完整的調研能力：問問題、讀現有文件、記錄發現、網路搜尋、讀取網頁內容

## Q3: Integration
- **Question**: GitHub/GitLab/Jira integration 要如何組織？
- **Answer**: 一起編譯，反正最終要暴露給 Python
- **Rationale**: 所有 integrations 都編譯進去，透過 builder pattern 動態啟用，方便 Python bindings

## Q4: Persistence
- **Question**: 是否需要 session/context 持久化？
- **Answer**: Pluggable storage
- **Rationale**: 可插拔 storage backend，支援 memory、file、或自定義實作

