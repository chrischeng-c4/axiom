---
change: prompt-registry
date: 2026-02-09
---

# Clarifications

## Q1: Registry 形式
- **Question**: prompt registry 要用什麼形式管理？
- **Answer**: Rust constants/functions — 在 helpers.rs 或新 prompts.rs 裡定義 prompt template functions，flow files 呼叫 function 取得 prompt
- **Rationale**: 保持 compile-time safety，不引入 runtime file loading 複雜度

## Q2: Scope
- **Question**: prompt registry 的 scope？
- **Answer**: Only run_change — 只管 run_change 的 30 個 inline prompt templates
- **Rationale**: 範圍明確，避免過度設計；其他 MCP tools 的 prompts 未來可以再擴展

## Q3: 核心目標
- **Question**: 主要想解決的問題是什麼？
- **Answer**: DRY / 減少重複 — 把重複的 prompt 結構抽出共用，修改一處即全部生效
- **Rationale**: 30 個 prompts 中有大量重複結構（explore x3, review x6, revise x6），修改一處目前需改多處

## Q4: DRY 粒度
- **Question**: DRY 的粒度如何？要合併相似 flow files 還是只 DRY prompt text？
- **Answer**: 只 DRY prompt text — 保留所有檔案結構，只把重複的 prompt 片段抽成 constants/functions
- **Rationale**: 維持 flow files 的獨立性和可讀性，只消除 prompt 文字層面的重複

