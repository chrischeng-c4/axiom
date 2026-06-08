---
change: fillback-main-specs-workflow
date: 2026-02-06
---

# Clarifications

## Q1: 觸發方式
- **Question**: 用戶使用時，需要指定哪些參數？
- **Answer**: mainthread 初步掃一下，用 AskUserQuestion 問一下用戶要 fillback 哪些部分
- **Rationale**: 自動偵測後讓用戶確認，兼顧便利性與控制

## Q2: Chunking 策略
- **Question**: 對於非 mono-repo 的大型 codebase，chunking 策略偏好？
- **Answer**: 根據專案動態評估，不預設固定策略
- **Rationale**: 不同專案結構差異大，需要彈性評估

## Q3: 產出位置
- **Question**: Fillback 產出的 spec 應該放在哪裡？
- **Answer**: 直接寫入 cclab/specs/{group}/，不走 change workflow
- **Rationale**: 因為是記錄現有行為，不需要 proposal/review 流程

## Q4: 詳細程度
- **Question**: Spec 的詳細程度？
- **Answer**: 非常詳細，因為目標是 spec-to-code。需要完整的 Mermaid diagrams、OpenAPI、OpenRPC、AsyncAPI、JSON Schema
- **Rationale**: Code-to-spec 是 spec-to-code 的反向操作，需要足夠詳細才能反向生成

