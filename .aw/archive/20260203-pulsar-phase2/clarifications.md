---
change: pulsar-phase2
date: 2026-01-31
---

# Clarifications

## Q1: Array Priority
- **Question**: pulsar-array 要優先擴充哪些功能到 50%?
- **Answer**: 統計函數優先 - mean, std, var, sum, min, max, percentile
- **Rationale**: 統計函數是資料分析最常用的基礎功能，優先實作可快速提升覆蓋率和實用性

## Q2: Frame Priority
- **Question**: pulsar-frame 要優先擴充哪些功能?
- **Answer**: 缺失值處理 - fillna, dropna, isna, interpolate
- **Rationale**: 真實世界資料經常有缺失值，資料清洗是任何分析的第一步

## Q3: Stats Scope
- **Question**: pulsar-stats (scipy) 要包含哪些模組?
- **Answer**: 完整 scipy.stats - 包含更多分佈和進階檢驗
- **Rationale**: 用戶需要完整的統計工具集，包括多種機率分佈、假設檢驗、相關分析等

