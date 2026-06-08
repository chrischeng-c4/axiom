---
change: mamba-string-reverse-slice
group: string-reverse-slice
date: 2026-03-28
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| string-ops | runtime | high | R6: mb_str_slice_full(s, start, stop, step) must support negative step, Absent start defaults to len-1, absent stop defaults to -1 sentinel bypassing clamp_rev_str, Fix applied in bc5921e9, s[::-1] reverses full string, s[4:1:-1] yields chars at indices 4,3,2, Unicode codepoint-based iteration |
| conformance | testing | medium | Golden file conformance: JIT stdout vs .expected byte-for-byte, Xfail: # mamba-xfail directive skips fixture; removal re-enables, Fixture: data_structures/string_edge_cases_xfail.py expects fedcba, Verify: cargo test -p mamba --test conformance_tests |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| string-reverse-slice-fix | modify | crates/mamba/runtime/string-ops.md | overview, requirements, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-string-reverse-slice

**Verdict**: pass

### Summary

PASS with one warning. The reference context correctly covers both affected areas (string-ops runtime and conformance testing). Relevance scores are appropriate. The spec plan is structurally sound: one modify entry on string-ops.md with a valid path, correct subfolder, and appropriate sections. Warning: the 'R6' requirement ID cited in the string-ops key requirements does not yet exist in the spec (verified: string-ops.md has R1-R5 only). R6 is a forward reference that this change will create — acceptable given the spec plan action is 'modify' with 'requirements' as a new section, but the reference context should ideally label it 'planned R6' to avoid ambiguity. Secondary observation: conformance.md (xfail-zero change spec) is the best available but not ideal spec for the xfail mechanism; test-harness.md would be a cleaner reference. Both are informational — the spec writing task can proceed.

### Issues

No issues found.
