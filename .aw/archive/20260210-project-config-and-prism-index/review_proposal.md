---
verdict: NEEDS_REVISION
file: proposal
iteration: 1
---

# Review: proposal (Iteration 1)

**Change ID**: project-config-and-prism-index

## Summary

Strong problem framing and clear value, but key implementation decisions are still ambiguous (index key derivation and migration behavior), and impact/risk accounting is likely understated for cross-crate persistence changes.

## Checklist

- ✅ Clarity
  - Problem statement is specific (monorepo mis-detection and non-persistent Prism index), and the target direction is understandable.
- ✅ Value
  - Proposed change creates clear user value: correct language inference in monorepos and reduced re-indexing cost after restarts.
- ❌ Completeness
  - Proposal does not define deterministic path-hash/canonicalization rules or module-to-index mapping behavior; migration behavior for uncertain detection is also underspecified.
- ✅ Feasibility
  - Technical approach is feasible with existing config/init/migrate/task generation touchpoints.
- ❌ Impact Accuracy
  - Estimated impact likely undercounts supporting work (tests/docs/error handling/backward compatibility paths) for multi-crate config and storage changes.

## Issues

- **[medium]** Path-hash strategy is ambiguous ("from root or from home") and may produce inconsistent index keys across environments.
  - *Recommendation*: Define one canonical algorithm in the proposal (e.g., absolute canonical project root path + stable normalization + SHA-256 prefix) and state collision handling expectations.
- **[medium]** Persistence design does not specify whether Prism uses one index per project or per module/language, which is important for monorepo behavior and storage layout.
  - *Recommendation*: Add explicit storage topology in proposal scope (single project index vs per-module/per-language sub-indexes) and expected lookup precedence.
- **[medium]** Migration plan for existing projects is too high-level and does not define safe behavior when language detection is partial/unknown.
  - *Recommendation*: Specify non-destructive migration rules, fallback behavior (e.g., empty modules + warning), and whether existing manual config entries are preserved or merged.

## Verdict

- [ ] PASS
- [x] NEEDS_REVISION
- [ ] REJECTED

