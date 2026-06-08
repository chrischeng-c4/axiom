# Task: Revise Reference Context for Group 'jit-refcount-audit' (Change '1129-patrol')

## Instructions

1. **Read artifact + review feedback**:
   `/Users/chrischeng/projects/wt/mamba/cclab/changes/1129-patrol/groups/jit-refcount-audit/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `/Users/chrischeng/projects/wt/mamba/cclab/changes/1129-patrol/groups/jit-refcount-audit/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `/Users/chrischeng/projects/wt/mamba/cclab/specs/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
cclab sdd artifact revise-reference-context 1129-patrol cclab/changes/1129-patrol/payloads/revise-reference-context.json
```