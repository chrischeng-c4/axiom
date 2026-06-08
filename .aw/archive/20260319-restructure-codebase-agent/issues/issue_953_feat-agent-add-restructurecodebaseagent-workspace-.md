---
number: 953
title: "feat(agent): Add RestructureCodebaseAgent — workspace-aware codebase decomposition"
state: open
labels: [enhancement, crate:agent, P1]
group: "restructure-codebase-agent"
---

# #953 — feat(agent): Add RestructureCodebaseAgent — workspace-aware codebase decomposition

## Summary

Agentic loop that decomposes a large codebase into manageable groups for fillback spec generation. Uses workspace manifests + folder summaries + token budget to drive LLM-based grouping decisions.

## Flow

```
1. Read root manifest (pyproject.toml / package.json / Cargo.toml)
   → workspace members
2. Per-member: list_folder_summary(depth=2) + estimate_tokens
3. LLM decides grouping (token budget constraint)
4. If group > budget → drill down, repeat
5. set_grouping(groups=[...]) → final artifact
```

## Tools

| Tool | Purpose |
|------|---------|
| read_manifest | Parse workspace members from manifest files |
| list_folder_summary | Folder tree + file count + line count at given depth |
| estimate_tokens | Folder → estimated token count (lines * 3) |
| set_grouping | Write final groups artifact (mandatory call) |

## Key Design

- **Workspace-first**: read manifests before listing anything
- **Token budget driven**: LLM gets budget constraint, decides how to split
- **Agentic loop**: multiple LLM calls + tool calls, not one-shot
- **Recursive drill-down**: large groups get further decomposed
- **Output**: groups artifact consumed by per-group RefCodebaseContext → CodebaseToSpec

## Differences from RestructureIssueAgent

| | RestructureIssueAgent | RestructureCodebaseAgent |
|---|---|---|
| Input | Issues (text) | Repo tree (manifests + folder summaries) |
| Style | One-shot LLM call | Agentic loop with tools |
| Grouping by | Spec boundary | Module/feature boundary + token budget |

## Dependencies
- #950 ReferenceCodebaseContextAgent (consumer of groups)
- #951 CodebaseToSpecAgent (consumer of per-group context)
