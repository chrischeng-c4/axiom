---
number: 949
title: "feat(lens): agent-optimized output — structured JSON for AI consumption"
state: open
labels: [type:enhancement, priority:p3, crate:sdd, crate:lens]
group: "lens-dissolution"
---

# #949 — feat(lens): agent-optimized output — structured JSON for AI consumption

## Problem

Current output formats (JSON, SARIF, Markdown, GitHub, GitLab) are designed for **humans or CI dashboards**, not AI agents.

An AI agent consuming Lens output needs:
1. **Actionable context**, not diagnostic lists — "what do I need to know?" not "what rules are violated"
2. **Compact representation** — every token in the output costs context window
3. **Structured relationships** — symbol X depends on Y, not flat lists

### Current JSON output
```json
[{"file": "a.py", "diagnostics": [{"code": "PY101", "message": "...", "range": ...}]}]
```
This is lint-focused. An agent contributing code needs:
- Symbol map (what's defined where, what type)
- Dependency edges (who imports whom)
- Impact scope (changing X affects Y, Z)

## Proposed: Agent JSON format

New `--format agent` output mode optimized for LLM consumption:

```json
{
  "symbols": {"get_user": {"type": "(int) -> User", "file": "db.py", "line": 42}},
  "imports": {"handler.py": ["db.get_user", "models.User"]},
  "issues": [{"severity": "error", "symbol": "get_user", "message": "..."}],
  "impact": {"db.get_user": ["handler.py:15", "api.py:33"]}
}
```

Key differences from current JSON:
- Symbol-centric, not file-centric
- Includes type signatures inline
- Includes dependency edges
- Compact (no redundant rule metadata)

## Acceptance Criteria

- [ ] New `--format agent` output mode for `cclab lens check`
- [ ] Output includes: symbol map, import edges, issues, impact scope
- [ ] Compact: no redundant rule descriptions, no SARIF boilerplate
- [ ] Works with `lens_check` MCP tool via format parameter
