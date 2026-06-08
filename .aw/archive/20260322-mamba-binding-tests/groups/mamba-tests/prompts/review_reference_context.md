# Task: Review Reference Context for Group 'mamba-tests' (Change 'mamba-binding-tests')

## Instructions

1. **Read pre-clarifications** (scope & requirements):
   `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-binding-tests/groups/mamba-tests/pre_clarifications.md`
2. **Read the reference context artifact**:
   `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-binding-tests/groups/mamba-tests/reference_context.md`
3. **Verify each spec entry**: For each spec listed in the artifact, read the actual spec under `/Users/chrischeng/projects/cclab-sdd/cclab/specs/` to verify relevance and key requirements are accurate.
4. **Devil's advocate**: Actively check — what crates/areas from pre-clarifications have NO spec covering them?
5. **Evaluate checklist** (pass/fail each item independently):
   - All affected crates/areas from pre-clarifications are covered by at least one spec
   - Relevance scores are reasonable (high = directly implements, medium = related, low = background)
   - Key requirements listed per spec are accurate (match actual requirement IDs)
   - No irrelevant specs included
6. **Separate observations from verdict**: First list all findings, then decide verdict based on evidence.
7. Write review verdict:

## CLI Commands

```
# Write review artifact (write payload JSON first, then run)
cclab sdd artifact review-reference-context mamba-binding-tests cclab/changes/mamba-binding-tests/payloads/review-reference-context.json
```