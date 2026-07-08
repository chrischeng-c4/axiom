---
id: sdd-generate-generators-cli-subcommand-helpers
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# CLI Subcommand Helper Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/generate/generators/cli_subcommand.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CliArg` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 18 |  |
| `CliArgKind` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | enum | pub | 44 |  |
| `CliCommand` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 53 |  |
| `CliEmitted` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | struct | pub | 83 |  |
| `emit_cli_subcommand` | apps/agentic-workflow/src/generate/generators/cli_subcommand.rs | function | pub | 150 | emit_cli_subcommand(cmd: &CliCommand) -> CliEmitted |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap missing-generator:struct-with-derives -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/generators/cli_subcommand.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:missing-generator:struct-with-derives>"
    description: "Source template owns CliEmitted and CLI subcommand helper functions."
```
