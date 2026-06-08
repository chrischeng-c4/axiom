# Task: Revise Reference Context for Group 'mamba-type-3arg-core' (Change 'mamba-type-3arg')

## Instructions

1. **Read artifact + review feedback**:
   `/Users/chris.cheng/cclab/main/.score/worktrees/mamba-type-3arg/.score/changes/mamba-type-3arg/groups/mamba-type-3arg-core/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `/Users/chris.cheng/cclab/main/.score/worktrees/mamba-type-3arg/.score/changes/mamba-type-3arg/groups/mamba-type-3arg-core/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `/Users/chris.cheng/cclab/main/.score/worktrees/mamba-type-3arg/.score/tech_design/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
score artifact revise-reference-context mamba-type-3arg .score/changes/mamba-type-3arg/payloads/revise-reference-context.json
```