---
number: 1142
title: "feat(sdd): check-alignment workflow integration — Phase 3"
state: open
labels: [type:enhancement, priority:p2, crate:sdd]
group: "check-alignment-phase3"
---

# #1142 — feat(sdd): check-alignment workflow integration — Phase 3

## Summary

Integrate `check-alignment` into the SDD workflow so alignment is checked automatically, not manually.

### Integration Points

| Point | Behavior | Blocking? |
|-------|----------|-----------|
| `sdd merge` — after archive, before git commit | Auto-run `check-alignment` on affected specs | Warn only, do not block |
| Review agent prompt | Inject alignment report into reviewer context | N/A |
| `sdd run-change` response | `alignment_warnings` field in response JSON | No |
| Artifact tools (write-time) | Call `check()` after writing spec section | Block on format violations (Phase 1 rules); warn on coverage gaps (Phase 2 rules) |

### Write-time vs Post-hoc Behavior

Same `spec_alignment::check()` function, different strictness:

| Caller | Format violations (Phase 1) | Coverage gaps (Phase 2) |
|--------|---------------------------|------------------------|
| Artifact tools (write-time) | Error — block write | Warning — allow write |
| CLI (post-hoc) | Report all | Report all |
| Merge workflow (pre-commit) | Warning | Warning |

### Acceptance Criteria

- `sdd merge` output includes alignment warnings when gaps exist
- Artifact tools reject specs with missing section annotations
- `cclab sdd check-alignment` works standalone for CI/manual use
- No change to existing merge behavior when alignment is clean

Depends on: #1140 (Phase 1), #1141 (Phase 2)
