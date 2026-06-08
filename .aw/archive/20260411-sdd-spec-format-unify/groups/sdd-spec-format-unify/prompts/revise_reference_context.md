# Task: Revise Reference Context for Group 'sdd-spec-format-unify' (Change 'sdd-spec-format-unify')

## Instructions

1. **Read artifact + review feedback**:
   `/Users/chris.cheng/cclab/main/.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `/Users/chris.cheng/cclab/main/.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `/Users/chris.cheng/cclab/main/.score/tech_design/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
score artifact revise-reference-context sdd-spec-format-unify .score/changes/sdd-spec-format-unify/payloads/revise-reference-context.json
```