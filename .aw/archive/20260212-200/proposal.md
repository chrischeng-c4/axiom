---
id: 200
type: proposal
version: 1
created_at: 2026-02-12T08:27:03.619262+00:00
updated_at: 2026-02-12T08:27:03.619262+00:00
author: mcp
status: proposed
iteration: 1
summary: "Rename outdated action names in fetch-issues.md to match run-change conventions"
history:
  - timestamp: 2026-02-12T08:27:03.619262+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: patch
  affected_files: 1
  new_files: 0
affected_specs:
  - id: fetch-issues
    path: specs/fetch-issues.md
    depends: []
---

<proposal>

# Change: 200

## Summary

Rename outdated action names in fetch-issues.md to match run-change conventions

## Why

fetch-issues.md uses create_spec_context and create_knowledge_context as action names, but run-change/README.md uses explore_spec and explore_knowledge. This inconsistency can cause confusion when reading the spec flow and may lead to incorrect tool dispatch.

## What Changes

- Replace create_spec_context with explore_spec in fetch-issues.md
- Replace create_knowledge_context with explore_knowledge in fetch-issues.md

## Impact

- **Scope**: patch
- **Affected Files**: ~1
- **New Files**: ~0
- Affected specs:
  - `fetch-issues` (no dependencies)

</proposal>
