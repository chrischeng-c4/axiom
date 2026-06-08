---
number: 786
title: "feat(agent): add accurate token counting"
state: open
labels: [enhancement, crate:agent, P0]
group: "token-counting-and-compact"
---

# #786 — feat(agent): add accurate token counting

## Context
Token counting is essential for context management, cost estimation, and prompt optimization. Currently requires `pip install tiktoken`.

## Scope
- `cclab.agent.count_tokens(text, model="gpt-4o") -> int`
- `cclab.agent.count_message_tokens(messages, model) -> int`
- Support tokenizers for:
  - OpenAI models (cl100k_base, o200k_base)
  - Claude models (claude tokenizer)
  - Gemini models
- `cclab.agent.truncate(text, max_tokens, model) -> str` — safe truncation
- `cclab.agent.estimate_cost(tokens, model) -> float` — cost estimation

## Replaces
- `tiktoken`
- Manual character-based estimation (currently in cclab-agent context.rs)

## Implementation
Rust BPE tokenizer or integrate `tiktoken-rs` crate → expose via cclab-agent-pyo3
