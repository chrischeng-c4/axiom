---
change: mamba-p1-bugfix
group: iter-two-arg
date: 2026-03-28
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| iter | ? | medium | — |
| hir-to-mir | ? | medium | — |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: mamba-p1-bugfix

**Verdict**: REVIEWED

### Summary

Two critical spec omissions: builtins spec (covers directly-affected runtime/builtins.rs) and conformance spec (has the S7/R2 scenario for iter(callable,sentinel)) are both missing from the table and spec_plan. iter relevance is mis-scored as medium when runtime/iter.rs is the primary fix target (should be high). All key requirements fields are empty ('—'). spec_plan is incomplete — only 2 of 4 required specs have entries.

### Checklist

- ❌ All affected crates/areas from pre-clarifications are covered by at least one spec
  - Issue #1113 lists runtime/builtins.rs as an affected file, but the builtins spec is absent. conformance.md (S7, R2) directly tracks the test to fix and is also absent.
- ❌ Relevance scores are reasonable (high = directly implements, medium = related, low = background)
  - iter is scored 'medium' but runtime/iter.rs is the primary fix target per issue Affected Files — should be 'high'. hir-to-mir as 'medium' is defensible given 'codegen error' in issue description.
- ❌ Key requirements listed per spec are accurate (match actual requirement IDs)
  - Both entries show '—' (empty). iter spec has R1 (__iter__/__next__ protocol), R5 (StopIteration) applicable; hir-to-mir spec has R1 (iterator setup in MIR). Requirements must be populated.
- ✅ No irrelevant specs included
  - Both included specs (iter, hir-to-mir) are relevant to this change.
- ✅ spec_plan: every entry has main_spec_ref set (not null)
  - Payload shows main_spec_ref set for both entries: crates/mamba/runtime/iter.md and crates/mamba/lower/hir-to-mir.md.
- ✅ spec_plan: sections are reasonable for the requirements
  - [overview, changes] is appropriate for modify actions on existing specs.
- ✅ spec_plan: modify entries have valid source paths
  - Both source paths exist in cclab/specs/crates/mamba/.
- ✅ spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
  - runtime/ and lower/ subfolders present in both paths.
- ✅ spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
  - iter.md covers iteration protocol; hir-to-mir.md covers HIR-to-MIR lowering — each a single logical unit.
- ✅ spec_plan: no spec file would require duplicate section types (split into separate files if needed)
  - No duplicate section types within either spec entry.
- ✅ spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
  - runtime/iter.md mirrors src/runtime/iter.rs; lower/hir-to-mir.md mirrors src/lower/hir_to_mir.rs.

### Issues

- **[HIGH]** Missing builtins spec — runtime/builtins.rs is listed in issue #1113 Affected Files ('iter() builtin dispatch') but builtins.md is absent from the reference context table and spec_plan.
  - *Recommendation*: Add builtins entry: relevance 'medium', key requirements R5 (Enhanced P1 builtins — iter() dispatch for dynamic 2-arg calls). Add to spec_plan with action: modify, source: crates/mamba/runtime/builtins.md, sections: [overview, changes].
- **[HIGH]** Missing conformance spec — conformance.md has R2 (Lambda passed to map/filter/iter does not SIGABRT) and S7 (iter(callable, sentinel) without SIGABRT) directly mapping to this fix. The blocked xfail test (T9.1) lives here and enabling it is part of the acceptance criteria.
  - *Recommendation*: Add conformance entry: relevance 'high', key requirements R2 (lambda codegen), S7 (iter callable sentinel scenario). Add to spec_plan with action: modify, source: crates/mamba/testing/conformance.md, sections: [overview, changes].
- **[MEDIUM]** iter relevance mis-scored as 'medium'. runtime/iter.rs is the primary fix target — mb_iter_new needs the sentinel variant per the issue. Should be 'high'.
  - *Recommendation*: Change iter relevance from 'medium' to 'high'.
- **[MEDIUM]** Key requirements are empty ('—') for both iter and hir-to-mir entries. This makes the reference context useless as a spec navigation aid.
  - *Recommendation*: Populate: iter → R1 (__iter__/__next__ protocol), R5 (StopIteration); hir-to-mir → R1 (comprehension/iterator setup lowering). Add note about R6 (iter 2-arg callable sentinel variant) being a new requirement to add to iter spec.
- **[LOW]** Group field shows '?' for both entries, indicating group resolution was not completed.
  - *Recommendation*: Resolve group to 'iter-two-arg' for all entries.
