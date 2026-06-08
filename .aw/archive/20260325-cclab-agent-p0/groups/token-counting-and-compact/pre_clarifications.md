---
change: cclab-agent-p0
group: token-counting-and-compact
date: 2026-03-16
status: answered
---

# Pre-Clarifications

### Q1: tokenizer-strategy
- **Answer**: Use tiktoken-rs crate. Battle-tested, supports cl100k_base/o200k_base for OpenAI models. Minimal maintenance burden.

### Q2: claude-tokenizer
- **Answer**: Use API-reported input_tokens/output_tokens from responses for accurate tracking. For pre-request estimation, use chars/4 heuristic as fallback.

### Q3: compact-threshold
- **Answer**: Trigger auto-compact at 80% of model's context window.

### Q4: summarization-model
- **Answer**: Configurable per-agent with default to Haiku (low cost, fast). Agent config field: compact_model.

