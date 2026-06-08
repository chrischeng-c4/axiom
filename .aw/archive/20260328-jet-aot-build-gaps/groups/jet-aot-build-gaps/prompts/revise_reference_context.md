# Task: Revise Reference Context for Group 'jet-aot-build-gaps' (Change 'jet-aot-build-gaps')

## Instructions

1. **Read artifact + review feedback**:
   `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/reference_context.md`
   Focus on the `# Reviews` section — list each issue to address.
2. **Read pre-clarifications** (confirm scope):
   `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/pre_clarifications.md`
3. **Address each issue one by one**: For each review issue:
   - Identify what needs to change (add spec? fix relevance? update key requirements?)
   - If a missing spec is mentioned, read it from `/Users/chris.cheng/cclab/cclab-jet/cclab/specs/`
   - Apply the fix to your specs array
4. **Self-verify**: Walk through each original review issue — is it resolved in the new specs array?
5. **Scope re-check**: Do the revised specs still cover all crates/areas from pre-clarifications?
6. Rewrite via artifact tool:

## CLI Commands

```
# Write revised artifact (write payload JSON first, then run)
cclab sdd artifact revise-reference-context jet-aot-build-gaps cclab/changes/jet-aot-build-gaps/payloads/revise-reference-context.json
```