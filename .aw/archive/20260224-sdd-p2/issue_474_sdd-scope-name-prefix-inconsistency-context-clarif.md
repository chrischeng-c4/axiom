---
number: 474
title: "SDD: Scope name prefix inconsistency — context_clarifications vs clarifications"
state: open
labels: [bug, P2, crate:sdd]
---

# #474 — SDD: Scope name prefix inconsistency — context_clarifications vs clarifications

## Summary

Multiple places in the implementation use shortened scope names that don't match the spec, potentially causing `sdd_read_artifact` to read the wrong artifact.

## Affected Scopes

| Spec scope | Implementation scope | Files |
|-----------|---------------------|-------|
| `context_clarifications` | `clarifications` | `clarify.rs:336,247,358-359` |
| `review_context_clarifications` | `review_clarifications` | `clarify.rs:359` |
| `spec_clarifications` | `post_clarifications` | `clarify.rs:451` (review_post_clarifications artifact name) |
| `spec_{{spec_id}}` | `{sid}` (bare) | `spec.rs:308` |
| `review_spec` | `review_spec_{sid}` | `spec.rs:310` vs `revise-change-spec.md:51` |

## Action Names

| Spec action | Implementation action | Files |
|------------|----------------------|-------|
| `review_context_clarifications` | `review_clarifications` | `clarify.rs:310` |
| `revise_context_clarifications` | `revise_clarifications` | `clarify.rs:345` |

## Impact

If `sdd_read_artifact` uses the scope string to locate files, mismatched scope names could cause reads to fail or read wrong artifacts.
