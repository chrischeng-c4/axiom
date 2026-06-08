---
id: 199
type: proposal
version: 1
created_at: 2026-02-12T08:24:13.157866+00:00
updated_at: 2026-02-12T08:24:13.157866+00:00
author: mcp
status: proposed
iteration: 1
summary: "Fix delegate-agent.md action enum coverage and verification table artifact names"
history:
  - timestamp: 2026-02-12T08:24:13.157866+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: patch
  affected_files: 1
  new_files: 0
affected_specs:
  - id: delegate-agent
    path: specs/delegate-agent.md
    depends: []
---

<proposal>

# Change: 199

## Summary

Fix delegate-agent.md action enum coverage and verification table artifact names

## Why

delegate-agent.md action enum is missing gap-create, merge, and per-task implementation actions. The verification table also has incorrect artifact names for spec actions (spec.md instead of specs/{spec_id}.md). This causes agent dispatch failures and false verification failures.

## What Changes

- Extend action enum to include gap_codebase_spec, gap_codebase_knowledge, gap_spec_knowledge, implement_task, review_implementation, begin_merge, resume_merge, review_merge, fix_merge
- Fix verification table artifact names: create_spec expected_artifact from spec.md to specs/{spec_id}.md, review_spec expected_artifact from review_spec.md to review_spec.md (unchanged but add note about per-spec naming)

## Impact

- **Scope**: patch
- **Affected Files**: ~1
- **New Files**: ~0
- Affected specs:
  - `delegate-agent` (no dependencies)

</proposal>
