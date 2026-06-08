# Task: Review Reference Context for Group 'stdlib-conformance' (Change 'mamba-p1-lang-features')

## Instructions

1. **Read pre-clarifications** (scope & requirements):
   `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/groups/stdlib-conformance/pre_clarifications.md`
2. **Read the reference context artifact**:
   `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/groups/stdlib-conformance/reference_context.md`
3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `/Users/chrischeng/projects/wt/mamba/cclab/specs/` to verify relevance and key requirements are accurate.
4. **Devil's advocate**: Actively check — what crates/areas from pre-clarifications have NO spec covering them?
5. **Evaluate checklist** (pass/fail each item independently):
   - All affected crates/areas from pre-clarifications are covered by at least one spec
   - Relevance scores are reasonable (high = directly implements, medium = related, low = background)
   - Key requirements listed per spec are accurate (match actual requirement IDs)
   - No irrelevant specs included
   - spec_plan: every entry has main_spec_ref set (not null)
   - spec_plan: sections are reasonable for the requirements
   - spec_plan: modify entries have valid source paths
   - spec_plan: main_spec_ref paths include a subfolder (not root-level under crate)
   - spec_plan: each spec file covers exactly one logical unit (not multiple unrelated concerns)
   - spec_plan: no spec file would require duplicate section types (split into separate files if needed)
   - spec_plan: spec paths mirror source structure (interfaces/, logic/, generate/)
6. **CRITICAL: List ALL issues in a single pass.** You only get ONE review round before auto-approve kicks in. Do NOT hold back issues for a future round — every problem must be reported NOW. Scan the entire artifact exhaustively before writing the verdict.
7. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
8. Write review verdict:

## CLI Commands

```
# Write review artifact (write payload JSON first, then run)
cclab sdd artifact review-reference-context mamba-p1-lang-features cclab/changes/mamba-p1-lang-features/payloads/review-reference-context.json
```