---
number: 926
title: "feat(agent): Add CRR cycle — generic Create-Review-Revise pattern"
state: open
labels: [enhancement, crate:agent, P1]
group: "agent-crr-core"
---

# #926 — feat(agent): Add CRR cycle — generic Create-Review-Revise pattern

**Parent**: #920

## Summary

Generic CRR (Create-Review-Revise) cycle that can be used by any agent pair. Extracted from cclab-sdd's CRR pattern.

## API

```rust
let crr = CRRCycle::new()
    .creator(spec_agent)
    .reviewer(review_agent)
    .reviser(spec_agent.clone())  // same or different
    .max_revisions(2)
    .on_event(|event| { /* SSE, logging, etc */ })
    .build()?;

let result = crr.run(input).await?;
// result.verdict, result.artifact, result.revision_count
```

## State Machine

```
Create → Review → verdict
  ↑                  ↓
  └── Revise ← NEEDS_REVISION
              APPROVED → done
              REJECTED → error
              max_revisions exceeded → auto-approve or error
```

## Used By
- SpecAgent (spec creation CRR)
- CodeAgent (code creation CRR)
- Any future agent pair

## Dependencies
- #924 ReviewAgent

## Test Plan
- [ ] Unit: CRR approved on first review
- [ ] Unit: CRR 2 revisions then approve
- [ ] Unit: CRR max revisions exceeded
- [ ] Unit: CRR rejected
