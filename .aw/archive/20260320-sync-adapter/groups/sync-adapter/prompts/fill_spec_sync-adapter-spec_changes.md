# Task: Fill Section 'changes' for Spec 'sync-adapter-spec' (Change 'sync-adapter')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/sync-adapter/specs/sync-adapter-spec.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec sync-adapter cclab/changes/sync-adapter/payloads/create-change-spec.json`