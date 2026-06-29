---
id: cap-source-resident-shell
summary: Source coverage for projects/cap/src/resident_shell.rs resident command-string execution.
fill_sections: [source, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The resident shell is part of cap run command-string execution and preserves supervised Bash fallback for unsupported shapes."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "Agent hooks route original Bash payloads into cap run command strings, which the resident shell optimizes or falls back without changing hook semantics."
---

# Source TD: projects/cap/src/resident_shell.rs

## Source
<!-- type: source lang: rust -->

`````rust
// ResidentLightShellSession captures cwd/env context for command-string mode.
// It runs only proven native CommandPlan::Native stages in-process and returns
// a bash -lc fallback for shell syntax, external plans, and unproven shapes.
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/src/resident_shell.rs"
    action: modify
    section: source
    description: |
      Source coverage for the resident light-shell boundary behind cap run
      command strings, including native-stage execution and Bash fallback.
    impl_mode: hand-written
```
