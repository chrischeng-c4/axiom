# Task: Fill Section 'changes' for Spec 'frontend-codegen-main' (Change 'sdd-codegen-completion')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/sdd-codegen-completion/specs/frontend-codegen-main.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec sdd-codegen-completion cclab/changes/sdd-codegen-completion/payloads/create-change-spec.json`