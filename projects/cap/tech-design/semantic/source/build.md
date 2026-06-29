---
id: cap-source-build
summary: Source coverage for projects/cap/build.rs build provenance stamping.
fill_sections: [source, changes]
capability_refs:
  - id: standard-agent-cli-operations
    role: primary
    gap: shared-standard-cli-commands
    claim: shared-standard-cli-commands
    coverage: full
    rationale: "The build script stamps CAP_TARGET, CAP_GIT_SHA, and CAP_BUILT_AT for cli_std::ToolInfo used by cap llm, upgrade, and issue diagnostics."
---

# Source TD: projects/cap/build.rs

## Source
<!-- type: source lang: rust -->

`````rust
// Build provenance stamping for the standard CLI ops.
// Emits CAP_GIT_SHA, CAP_BUILT_AT, and CAP_TARGET for cli_std::ToolInfo.
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/build.rs"
    action: modify
    section: source
    description: |
      Source coverage for cap build provenance stamping used by the shared
      cli-std ToolInfo release and diagnostics contract.
    impl_mode: hand-written
```
