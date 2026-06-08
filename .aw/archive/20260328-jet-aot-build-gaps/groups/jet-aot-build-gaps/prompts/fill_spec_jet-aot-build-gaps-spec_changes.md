# Task: Fill Section 'changes' for Spec 'jet-aot-build-gaps-spec' (Change 'jet-aot-build-gaps')

**DO NOT use Write or Edit tools to modify spec files directly. Use the artifact CLI only.**

1. Read spec via `cclab sdd workflow read-artifact jet-aot-build-gaps` with scope="jet-aot-build-gaps-spec"
2. Write content for **changes**: List files that will change:
```yaml
files:
  - path: foo.rs
    action: CREATE
    desc: ...
```
Begin with `<!-- type: changes lang: yaml -->`.
3. Write payload JSON to the change's payloads directory (do NOT write to repo root or CWD), then run: `cclab sdd artifact create-change-spec jet-aot-build-gaps <payload_path>`