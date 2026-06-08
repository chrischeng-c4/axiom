# Task: Fill Section 'changes' for Spec 'agent-pyo3-spec' (Change 'agent-pyo3')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec: `cclab/changes/agent-pyo3/specs/agent-pyo3-spec.md`
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON then run: `cclab sdd artifact create-change-spec agent-pyo3 cclab/changes/agent-pyo3/payloads/create-change-spec.json`