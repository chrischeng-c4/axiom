---
number: 477
title: "SDD: Merge begin/resume prompts have undocumented codebase_paths/knowledge_refs enrichment"
state: open
labels: [enhancement, P2, crate:sdd]
---

# #477 — SDD: Merge begin/resume prompts have undocumented codebase_paths/knowledge_refs enrichment

## Summary

The implementation adds context-enrichment steps to `begin_merge` and `resume_merge` that are not in the spec. These steps read `codebase_context.md` and `knowledge_context.md`, then embed `codebase_paths` and `knowledge_refs` into merged spec frontmatter.

## Spec (merge-change.md:196-216)

Only lists:
1. List specs in the change directory
2. For each spec, read and merge via `sdd_write_artifact`

No mention of `codebase_context`, `knowledge_context`, `codebase_paths`, or `knowledge_refs`.

## Implementation (merge.rs:98-122, 130-150)

Adds:
1. Read `codebase_context.md` via `sdd_read_artifact(scope="codebase_context")`
2. Read `knowledge_context.md` via `sdd_read_artifact(scope="knowledge_context")`
3. For each spec, identify relevant codebase files and knowledge docs
4. Include `codebase_paths` and `knowledge_refs` in YAML frontmatter

The `sdd_write_artifact` payload also differs: spec uses `{path, content}`, code uses `{spec_id, spec_group}`.

## Additional Merge Inconsistencies

- `review_merge` artifact: spec says `artifact="implementation"`, code says `artifact="merge"` (`merge-change.md:61` vs `merge.rs:172`)
- `merge_complete` has "SDD SDD" typo in both spec and code (likely a typo)

## Action

Update spec to document the enrichment behavior (it's a useful feature). Also fix the `artifact` name inconsistency.
