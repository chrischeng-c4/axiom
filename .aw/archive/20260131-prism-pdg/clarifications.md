---
change: prism-pdg
date: 2026-01-31
---

# Clarifications

## Q1: Language Support
- **Question**: PDG 應該支援哪些語言？
- **Answer**: Python only
- **Rationale**: 先專注 Python，與現有 type system 整合最完整，Rust/TS 可後續擴展

## Q2: Granularity
- **Question**: PDG 分析的粒度層級？
- **Answer**: Statement-level
- **Rationale**: 每個 statement 作為 PDG node，標準做法，平衡精確度與效能

## Q3: Features
- **Question**: 需要哪些 PDG 功能？
- **Answer**: Program slicing, Impact analysis, Taint tracking, Dead code detection
- **Rationale**: 四項核心功能：slicing 是 LLM 理解程式碼的基礎，impact analysis 支援變更分析，taint tracking 用於安全分析，dead code detection 幫助程式碼清理

## Q4: Analysis Scope
- **Question**: Inter-procedural 分析範圍？
- **Answer**: Cross-file inter-procedural
- **Rationale**: 完整跨檔案分析，支援複雜專案的依賴追蹤，與現有 ImportGraph 整合

