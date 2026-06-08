---
number: 946
title: "feat(lens): agent context builder — smart file selection for AI task execution"
state: open
labels: [type:enhancement, priority:p2, crate:sdd, crate:lens]
group: "lens-dissolution"
---

# #946 — feat(lens): agent context builder — smart file selection for AI task execution

## Problem

When an AI agent receives a task like "add caching to the user service", it needs to figure out which files to read. Currently agents either:
1. Read everything (wastes context window)
2. Use grep/glob (misses semantic connections)
3. Guess from file names (unreliable)

Lens has all the pieces — import graph, symbol tables, call hierarchy, type info — but no tool that combines them into "here's what you need to read for this task."

## Proposed: `lens_context` MCP tool

Given a set of target files/symbols + a task description, return the minimal context an agent needs:

```json
// Input
{
  "targets": ["src/services/user.py:get_user"],
  "task": "add Redis caching",
  "depth": 2
}

// Output
{
  "must_read": [
    {"path": "src/services/user.py", "reason": "target file", "symbols": ["get_user", "UserService"]},
    {"path": "src/models/user.py", "reason": "imported by target", "symbols": ["User", "UserQuery"]},
    {"path": "src/db/connection.py", "reason": "called by get_user", "symbols": ["get_session"]}
  ],
  "may_affect": [
    {"path": "src/api/routes.py", "reason": "calls get_user", "symbols": ["user_endpoint"]},
    {"path": "tests/test_user.py", "reason": "tests get_user", "symbols": ["test_get_user"]}
  ],
  "type_context": {
    "User": "dataclass with fields: id(int), name(str), email(str)",
    "get_session": "() -> AsyncSession"
  }
}
```

## Algorithm

1. Start from target symbols
2. Follow import graph forward (what does target depend on?)
3. Follow call graph backward (what calls target?)
4. Follow test file naming conventions (what tests cover target?)
5. Collect type signatures for all referenced symbols
6. Rank by relevance, truncate to configurable depth

## Why This Matters

This is Lens's **highest-value capability for AI agents** — no other tool provides semantic-aware context selection. It directly reduces hallucination (agent sees real types) and missed impacts (agent sees all callers).

## Acceptance Criteria

- [ ] New `lens_context` MCP tool available
- [ ] CLI: `cclab lens context <file:symbol> [--depth N]`
- [ ] Returns must_read (dependencies) and may_affect (reverse dependencies)
- [ ] Includes type signatures for cross-boundary symbols
- [ ] Configurable depth limit (default 2)
- [ ] Works for Python, TypeScript, Rust, Go
