---
number: 951
title: "feat(agent): Add CodebaseToSpecAgent — reverse-engineer specs from code"
state: open
labels: [enhancement, crate:agent, P1]
group: "fillback-agents"
---

# #951 — feat(agent): Add CodebaseToSpecAgent — reverse-engineer specs from code

## Summary

Agent that takes codebase context (from ReferenceCodebaseContextAgent) and produces formal specs. Used in fillback flow — onboarding existing repos.

Counterpart to ChangeSpecAgent which writes specs from issues + spec context.

## Differences from ChangeSpecAgent

| | ChangeSpecAgent | CodebaseToSpecAgent |
|---|---|---|
| Input | Issues + spec context | Codebase context |
| System prompt | "write spec for new feature" | "reverse-engineer spec from existing code" |
| CRR review focus | Spec format, coverage | Accuracy — does spec match code? |
| Quality standard | Same format rules (OpenRPC > JSON Schema > Mermaid) | Same format rules |

## Dependencies
- ReferenceCodebaseContextAgent (provides codebase context)
- ReviewAgent + CRR
- Same spec format rules as ChangeSpecAgent
