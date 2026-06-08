---
number: 467
title: "SDD: Verdict label inconsistency — APPROVED vs PASS across all review prompts"
state: open
labels: [bug, P1, crate:sdd]
---

# #467 — SDD: Verdict label inconsistency — APPROVED vs PASS across all review prompts

## Summary

Spec consistently uses `APPROVED` as the positive review verdict, but multiple Rust implementation files emit `PASS` in the prompt sent to agents. The code accepts both (`Some("PASS") | Some("APPROVED")`), but the spec has no knowledge of `PASS`.

## Affected Files

| Spec | Implementation | Line |
|------|---------------|------|
| `review-gap-codebase-spec.md:28` | `gap_codebase_spec.rs:102` | Prompt says `PASS` |
| `review-gap-codebase-knowledge.md:28` | `gap_codebase_knowledge.rs:102` | Prompt says `PASS` |
| `review-gap-spec-knowledge.md:28` | `gap_spec_knowledge.rs:103` | Prompt says `PASS` |
| `review-change-proposal.md:59` | `proposal.rs:143` | Prompt says `PASS` |
| `implement-change.md:388` | `implement.rs:449` | Prompt says `PASS` |

## Decision Needed

Pick one and unify everywhere:
- **Option A**: Standardize on `APPROVED` (matches spec, more readable)
- **Option B**: Standardize on `PASS` (shorter, already in code)

Whichever is chosen, update both spec and implementation to use only one label.

## Risk

If a downstream consumer pattern-matches on `APPROVED` exclusively (per spec), agents emitting `PASS` (per prompt) would fail to trigger the correct state transition.
