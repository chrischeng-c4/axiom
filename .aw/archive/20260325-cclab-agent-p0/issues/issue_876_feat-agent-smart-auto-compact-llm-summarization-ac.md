---
number: 876
title: "feat(agent): smart auto-compact — LLM summarization, accurate token counting, tool-call pairing"
state: open
labels: [enhancement, crate:agent, P0]
dependencies: [786]
group: "token-counting-and-compact"
---

# #876 — feat(agent): smart auto-compact — LLM summarization, accurate token counting, tool-call pairing

## Summary

`ContextManager` 目前的 auto-compact 機制過於簡陋，只做 FIFO 刪除最舊訊息。需要升級為智慧壓縮策略，類似 Claude Code 的 auto compact 行為。

## Current Behavior

- `compress()` 從頭部逐一刪除舊訊息，僅保留最後 4 條
- Token 計數用 `text.len() / 4` 粗估
- 可能破壞 tool_use / tool_result 配對
- 無法區分訊息重要性

## Proposed Improvements

### 1. Accurate token counting (depends on #786)
- 使用 tiktoken 或 API-reported usage 精準計算 token
- 替換現有 `len/4` 粗估

### 2. LLM-based summarization
- 超過 threshold 時，用 LLM 將舊訊息摘要為精簡版本，而非直接刪除
- 保留關鍵上下文（決策、變數、目標）
- 可配置摘要模型（用低成本模型如 Haiku）

### 3. Tool call pairing protection
- 確保 tool_use 和 tool_result 成對保留或成對刪除/摘要
- 避免壓縮後產生不合法的 message sequence

### 4. Priority-aware retention
- System prompt: 永遠保留
- 最近 N 條訊息: 永遠保留
- Tool call 結果: 可摘要但保留 call 結構
- 中間對話: 優先壓縮

## Reference

- Claude Code auto compact: 接近 context limit 時自動用 LLM 摘要歷史訊息
- 現有程式碼: `crates/cclab-agent/src/context.rs`

## Related

- #786 — accurate token counting (前置工作)
- #792 — structured output
