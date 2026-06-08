# Task: Fill Section 'changes' for Spec 'fix-monorepo-bundler-spec' (Change 'fix-monorepo-bundler')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/fix-monorepo-bundler/specs/fix-monorepo-bundler-spec.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec fix-monorepo-bundler cclab/changes/fix-monorepo-bundler/payloads/create-change-spec.json`