# Task: Revise Reference Context for Group 'class-features' (Change 'mamba-all-p1')

## Instructions

1. **Read artifact + review feedback**:
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/class-features/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/class-features/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `/Users/chris.cheng/cclab/cclab-mamba/cclab/specs/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
cclab sdd artifact revise-reference-context mamba-all-p1 cclab/changes/mamba-all-p1/payloads/revise-reference-context.json
```