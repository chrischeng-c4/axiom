---
id: design-resident-light-shell-with-dynamic-bash-fallback
summary: Design and land the first resident light-shell runtime slice for cap run command strings with Bash fallback.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "The resident light shell is an execution path behind cap run and must preserve supervised original-command fallback semantics."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: partial
    rationale: "Agent hooks route Bash Tool payloads into cap run command strings, so the light shell must preserve hook compatibility."
---

# Design Resident Light Shell With Dynamic Bash Fallback

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TB
  AgentHook["Claude/Codex Bash hook"] --> CapRun["cap run '<original command string>'"]
  DirectRun["user cap run '<command string>'"] --> CapRun
  CapRun --> Session["ResidentLightShellSession"]

  Session --> Preserve["Session state: cwd + env snapshot from cap process"]
  Session --> Parse["Parse conservative light-shell subset"]
  Parse -->|single shell-free argv| Planner["command_planner::plan"]
  Parse -->|unsupported syntax or unproven shape| BashFallback["bash -lc original"]

  Planner -->|Native plan| NativeStage["run native stage in process"]
  Planner -->|External Original/Replacement| BashFallback
  NativeStage --> Parity["stdout/stderr/exit parity evidence"]
  BashFallback --> ManagedRun["existing managed_run lease path"]

  ManagedRun --> Daemon["cap daemon resource protection"]
  NativeStage --> Boundary["first slice: in-process native path is observable but does not replace full Bash"]
  Boundary --> Product["cap remains Bash-compatible optimizer and resource governor, not sandbox or full shell"]
```

The resident-light-shell design belongs in this TD because it changes the
`cap run "<command string>"` execution boundary. The first implementation slice
is intentionally narrow: a per-invocation `ResidentLightShellSession` owns the
current cwd/env snapshot, attempts one conservative native stage using the
existing planner/native runner, and returns a structured fallback for every
unsupported form. This keeps the daemon/client resource-protection path intact
for Bash fallback while making an observable in-process native path available
for future resident/session reuse.
```
