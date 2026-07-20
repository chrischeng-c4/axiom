# Workbench Capabilities

<!-- aw:meta:project-capabilities:start -->
## Brief

Machine-readable capability contract for Workbench.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
<!-- aw:meta:project-capabilities:end -->

### Terminal-first Agent Workbench

ID: terminal-first-agent-workbench
Type: RuntimeTool
Root WI: #2171
Status: confirmed
Surfaces: Native desktop host, registered launch folders, real agent PTY, active cwd, context renderer registry, provenance, and production evidence.
EC Dimensions: behavior: pending folder-to-agent-to-artifact journey
Required Verification: smoke, integration, e2e
Promise:
Workbench launches Claude Code, Codex, or AGY through its native CLI in a real
PTY and adds optional, read-only context without owning vendor sessions, AW
lifecycle state, or inferred knowledge as canonical repository truth.
Gate Inventory:
- in_progress: `cargo test -p workbench --test desktop_launch_smoke -- --nocapture`
- planned: `cargo test -p workbench --test production_journey -- --nocapture`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| desktop-application-bootstrap | change | #2191 | in_progress | in_progress | prototype | `desktop_launch_smoke` |
| three-column-folder-shell | change | #2192 | planned | planned | none | `folder_shell_journey` |
| native-agent-pty | change | #2193 | planned | planned | none | `pty_agent_adapters` |
| authoritative-cwd-context | change | #2194 | planned | planned | none | `pty_cwd_context` |
| generic-context-renderers | change | #2195 | planned | planned | none | `generic_context_renderers` |
| optional-aw-typed-renderer | change | #2196 | planned | planned | none | `aw_typed_renderer` |
| context-provenance | change | #2198 | planned | planned | none | `context_provenance` |
| optional-graph-adapter | change | #2199 | planned | planned | none | `graph_context_adapter` |
| optional-derived-page-adapter | change | #2200 | planned | planned | none | `derived_page_context_adapter` |
| production-journey | change | #2201 | planned | planned | none | `production_journey` |
