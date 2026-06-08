# Task: Fill Section 'changes' for Spec 'change-spec-logic' (Change 'sdd-codegen-and-fixes')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/sdd-codegen-and-fixes/specs/change-spec-logic.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec sdd-codegen-and-fixes cclab/changes/sdd-codegen-and-fixes/payloads/create-change-spec.json`