# Task: Fill Section 'changes' for Spec 'stream-resolver-spec' (Change 'stream-resolver')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/stream-resolver/specs/stream-resolver-spec.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec stream-resolver cclab/changes/stream-resolver/payloads/create-change-spec.json`