# Task: Review Reference Context for Group 'mamba-py312-conformance' (Change 'mamba-conformance-p0')

## Instructions

1. **Read pre-clarifications** (scope & requirements):
   `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/groups/mamba-py312-conformance/pre_clarifications.md`
2. **Read the reference context artifact**:
   `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/groups/mamba-py312-conformance/reference_context.md`
3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/specs/` to verify relevance and key requirements are accurate.
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
6. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
7. Write review verdict:

## CLI Commands

```
# Write review artifact (write payload JSON first, then run)
cclab sdd artifact review-reference-context mamba-conformance-p0 cclab/changes/mamba-conformance-p0/payloads/review-reference-context.json
```