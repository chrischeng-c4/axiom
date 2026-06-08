---
change: cclab-agent-p0
group: token-counting-and-compact
date: 2026-03-16
---

# Requirements

## Token Counting (#786)
- Implement `count_tokens(text, model) -> int` and `count_message_tokens(messages, model) -> int`
- Support tokenizers: OpenAI (cl100k_base, o200k_base), Claude, Gemini
- Implement `truncate(text, max_tokens, model) -> str` for safe truncation
- Implement `estimate_cost(tokens, model) -> float` for cost estimation
- Use tiktoken-rs or Rust BPE tokenizer, expose via PyO3
- Replace existing `text.len() / 4` estimation in context.rs

## Smart Auto-Compact (#876)
- Upgrade ContextManager compress() from FIFO deletion to intelligent summarization
- Use accurate token counting from #786 (replaces len/4 estimation)
- LLM-based summarization: summarize old messages instead of deleting, configurable model (e.g. Haiku)
- Tool call pairing protection: ensure tool_use/tool_result are kept or removed as pairs
- Priority-aware retention: system prompt always kept, recent N messages kept, middle conversation compressed first
- Preserve key context (decisions, variables, goals) during compression
