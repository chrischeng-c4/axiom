# Task: Fill Section 'changes' for Spec 'install-perf-spec' (Change 'install-perf')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/install-perf/specs/install-perf-spec.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec install-perf cclab/changes/install-perf/payloads/create-change-spec.json`