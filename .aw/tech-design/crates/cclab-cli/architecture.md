---
id: cli-architecture
type: spec
title: "CLI Architecture"
version: 1
spec_type: utility
created_at: 2026-01-31T12:45:00+00:00
updated_at: 2026-01-31T12:45:00+00:00
design_elements:
  has_mermaid: true
  diagrams:
    - type: flowchart
      title: "CLI Command Structure"
---

<spec>

# CLI Architecture

## Overview

The `cclab` command provides unified CLI for all cclab operations. Modules can register themselves via the linkme `#[distributed_slice(CLI_MODULES)]` pattern for automatic discovery.

## Command Structure

```mermaid
flowchart TB
    cclab[cclab] --> api[api]
    cclab --> titan[titan]
    cclab --> probe[probe]
    cclab --> server[server]
    cclab --> gen[gen]
    cclab --> talos[talos]

    api --> api_serve[serve]
    api --> api_routes[routes]

    titan --> titan_migrate[migrate]
    titan --> titan_revision[revision]

    probe --> probe_run[run]
    probe --> probe_collect[collect]
    probe --> probe_migrate[migrate]
    probe --> probe_generate[generate]

    probe_generate --> probe_gen_test[test]
    probe_generate --> probe_gen_bench[bench]

    server --> server_start[start]
    server --> server_stop[stop]

    gen --> gen_plan[plan-change]
    gen --> gen_impl[impl-change]
    gen --> gen_merge[merge-change]
    gen --> gen_knowledge[knowledge]
    gen --> gen_spec[spec]

    talos --> talos_build[build]
    talos --> talos_dev[dev]
```

## CliModule Registration

Modules auto-register via linkme distributed slices. Registered modules are dispatched first before falling back to legacy handling.

```rust
// Example: ProbeCli registration (crates/cclab-cli/src/probe/mod.rs)
#[distributed_slice(CLI_MODULES)]
static PROBE_CLI: &dyn CliModule = &ProbeCli;
```

**Registered CliModules**:
| Module | Status | Location |
|--------|--------|----------|
| ion | Registered | `crates/cclab-cli/src/ion.rs` |
| probe | Registered | `crates/cclab-cli/src/probe/mod.rs` |

## Command Dispatch Flow

```mermaid
flowchart TB
    Input[User Input] --> Parse[Parse Args]
    Parse --> Registered{Try Registered<br/>CliModules}
    Registered -->|ion, probe| CliModule[CliModule.execute]
    Registered -->|not found| Match{Match Legacy<br/>Subcommand}
    Match -->|api| API[API Module]
    Match -->|titan| Titan[Titan Module]
    Match -->|server| Server[Server Module]
    Match -->|gen| Gen[SDD Module]
    Match -->|talos| Talos[Warp Module]
    Match -->|unknown| Help[Show Help]

    CliModule --> Execute[Execute Command]
    API --> Execute
    Titan --> Execute
    Server --> Execute
    Gen --> Execute
    Talos --> Execute
    Execute --> Output[Output Result]
```

**Dispatch Priority**:
1. **Registered modules** (via `try_dispatch_registered()`) - ion, probe
2. **Legacy subcommands** - api, titan, server, gen, talos
3. **Help** - unknown commands

</spec>
