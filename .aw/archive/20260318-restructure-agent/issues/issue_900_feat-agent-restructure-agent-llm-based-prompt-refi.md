---
number: 900
title: "feat(agent): Restructure Agent — LLM-based prompt refinement with structured I/O"
state: open
labels: [enhancement, crate:agent, P0]
group: "restructure-agent-core"
---

# #900 — feat(agent): Restructure Agent — LLM-based prompt refinement with structured I/O

## Summary

Core agent that takes a user's vague intent, enriches it with spec context, clarifies ambiguities, and produces well-structured issues (meta-prompts for downstream agents).

**Parent**: #899

## What It Does

The agent is thin — all intelligence is in the LLM call:

1. **LLM judges**: intent + specs → is information sufficient?
2. **LLM produces**: insufficient → `questions[]` / sufficient → `structured issues[]`
3. That's it.

Agent code only:
- Assembles prompt (intent + clarifications + specs)
- Calls LLM with structured output (JSON Schema enforced)
- Returns result

```rust
async fn run(input: RestructureInput) -> RestructureOutput {
    let specs = spec_store.search(&input.intent).await?;
    let prompt = build_prompt(&input, &specs);
    let result = provider.complete_structured(prompt, schema).await?;
    result
}
```

## Standard Input

```json
{
  "intent": "加 OAuth2 支援，要 Google 和 GitHub",
  "project_id": "cclab",
  "clarifications": [
    { "question": "需要 token refresh 嗎?", "answer": "要" }
  ]
}
```

## Standard Output (discriminated union)

```json
// Type 1: need_clarification
{
  "type": "need_clarification",
  "questions": [
    {
      "id": "q1",
      "question": "現有 AuthProvider trait 有 login/logout，OAuth2 需要 authorize_url 和 token_exchange，要擴充還是建新的？",
      "why": "影響是修改現有 spec 還是新增 spec",
      "suggestions": ["擴充現有 trait", "建新的 OAuth2Provider trait"]
    }
  ]
}

// Type 2: create_issues
{
  "type": "create_issues",
  "issues": [
    {
      "title": "feat(auth): add OAuth2 provider trait",
      "description": "...",
      "issue_type": "feature",
      "priority": "P1",
      "labels": ["auth", "oauth2"],
      "acceptance_criteria": [
        "OAuth2Provider trait with authorize_url, exchange_code, refresh_token",
        "Unit tests for trait contract"
      ],
      "depends_on": [],
      "scope": "medium"
    }
  ],
  "summary": "拆成 3 個 issues: 1 個 trait 抽象 + 2 個 provider 實作"
}
```

## Internal Pipeline

```
Input { intent, clarifications, project_id }
    │
    ├─ 1. Parse ─────── Extract goal, scope, constraints
    ├─ 2. Explore ───── SpecStore.search() + read()
    ├─ 3. Analyze ───── Gap analysis: intent vs existing specs
    ├─ 4. Decide ────── Sufficient info?
    │     ├─ No  → 5a. Clarify (questions with spec context)
    │     └─ Yes → 5b. Structure (issues with AC, deps, labels)
    │
Output { need_clarification | create_issues }
```

All steps are LLM reasoning — no business logic in agent code.

## Key Principle

Questions must be **spec-informed**, not generic:
```
❌ "請問你需要什麼功能？"
✅ "現有 AuthProvider trait 有 login/logout，OAuth2 需要 authorize_url + token_exchange，要擴充還是建新的？"
```

## Dependencies

- SpecStore interface (#901)
- Existing `LLMProvider` trait + `complete_structured()`

## Test Plan

- [ ] Unit: mock LLM → returns need_clarification
- [ ] Unit: mock LLM → returns create_issues
- [ ] Unit: specs context correctly assembled in prompt
- [ ] Integration: real LLM call with sample intent
