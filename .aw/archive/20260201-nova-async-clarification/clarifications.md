---
change: nova-async-clarification
date: 2026-02-01
---

# Clarifications

## Q1: Platform Support
- **Question**: Comment 格式要支援哪些平台？
- **Answer**: GitHub + GitLab + Jira - 三個平台都支援 checkbox 格式，完整覆蓋
- **Rationale**: 已有三個平台的 integration，應該一併支援 post_comment 功能

## Q2: Storage Backend
- **Question**: Session 暫停時的狀態儲存位置？
- **Answer**: 用 Storage trait 抽象，使用者可注入 DB 實作（MongoDB/PostgreSQL）
- **Rationale**: FileStorage 在 K8s 環境不好管理，透過 trait 抽象讓使用者自己注入 DB 實作更彈性

## Q3: Reply Parsing
- **Question**: 如何解析用戶的回覆？
- **Answer**: Checkbox + Reply text - 解析 [x] checkbox，也接受 reply comment 的文字
- **Rationale**: 支援兩種方式更彈性：打勾選擇或自由文字回覆

## Q4: Resume Scope
- **Question**: Resume 對話恢復的範圍？
- **Answer**: 完整 LLM messages - 儲存完整 messages，resume 時 LLM 能看到之前的對話
- **Rationale**: 保持完整上下文才能讓 Agent 理解之前的分析進度和用戶需求

