---
number: 898
title: "feat(sdd): support user-facing doc as change artifact"
state: open
labels: [enhancement, crate:sdd]
group: "sdd-frontend-doc-artifacts"
---

# #898 — feat(sdd): support user-facing doc as change artifact

## Summary

SDD workflow tracks code specs but not user-facing documentation. When code behavior changes, corresponding user docs (guides, CLI help, API reference, tutorials) are not part of the change lifecycle, causing docs to drift from code.

## Proposal

Treat user-facing doc updates as a spec/artifact within a change, sharing the same per-spec lifecycle:

1. **Create** — agent senses user-facing impact → generates doc spec skeleton
2. **Fill** — writes doc content based on code spec requirements/scenarios
3. **Review** — verifies doc accurately reflects code changes
4. **Implement** — writes doc content to target path
5. **Merge** — doc changes archived alongside code

### Doc spec frontmatter

```yaml
---
id: doc-theme-toggle
doc_target: docs/guide/settings.md
merge_strategy: append | replace
fill_sections: [overview, content, examples]
---
```

### Trigger conditions

Auto-create doc spec when change involves:
- CLI command changes
- New/modified public API
- Behavior changes
- Config format changes
