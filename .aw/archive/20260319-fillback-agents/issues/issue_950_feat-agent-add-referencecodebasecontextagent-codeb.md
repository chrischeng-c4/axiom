---
number: 950
title: "feat(agent): Add ReferenceCodebaseContextAgent — codebase exploration for fillback"
state: open
labels: [enhancement, crate:agent, P1]
group: "fillback-agents"
---

# #950 — feat(agent): Add ReferenceCodebaseContextAgent — codebase exploration for fillback

## Summary

Agent that explores codebase (not specs) to build context for CodebaseToSpecAgent. Used in fillback flow when onboarding an existing repo that has no specs.

Counterpart to ReferenceSpecContextAgent (renamed from ReferenceContextAgent) which explores specs.

## Two Pipelines

```
SDD flow:      RestructureIssue → RefSpecContext → ChangeSpec → CodeAgent
Fillback flow: RefCodebaseContext → CodebaseToSpec
```

## Differences from ReferenceSpecContextAgent

| | RefSpecContext | RefCodebaseContext |
|---|---|---|
| Explores | SpecStore (specs/) | Codebase (src/) via Grep, Glob, Read, AST |
| System prompt | "find related specs, score relevance" | "read code, understand architecture, API, data models" |
| Output | Spec references + relevance | Code references + architecture findings |
| Consumer | ChangeSpecAgent | CodebaseToSpecAgent |

## Dependencies
- Existing tools: Grep, Glob, Read
- ReviewAgent + CRR for quality review
