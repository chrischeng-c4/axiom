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
- verified: `cargo test -p workbench --test desktop_launch_smoke -- --nocapture`
- verified: `cargo test -p workbench --test folder_shell_journey -- --nocapture`
- verified: `cargo test -p workbench --test pty_agent_adapters -- --nocapture`
- verified: `cargo test -p workbench --test pty_cwd_context -- --nocapture`
- verified: `cargo test -p workbench --test generic_context_renderers -- --nocapture`
- verified: `cargo test -p workbench --test aw_typed_renderer -- --nocapture`
- planned: `cargo test -p workbench --test production_journey -- --nocapture`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| desktop-application-bootstrap | change | #2191 | complete | verified | prototype | `desktop_launch_smoke` |
| three-column-folder-shell | change | #2192 | complete | verified | prototype | `folder_shell_journey` plus retained viewport evidence |
| native-agent-pty | change | #2193 | complete | verified | prototype | `pty_agent_adapters` real-shell fixture |
| authoritative-cwd-context | change | #2194 | complete | verified | prototype | `pty_cwd_context` OSC 7 real-PTY fixture |
| generic-context-renderers | change | #2195 | complete | verified | prototype | `generic_context_renderers` non-AW Git fixture |
| optional-aw-typed-renderer | change | #2196 | complete | verified | prototype | `aw_typed_renderer` four-kind fixtures and byte identity |
| context-provenance | change | #2198 | planned | planned | none | `context_provenance` |
| optional-graph-adapter | change | #2199 | planned | planned | none | `graph_context_adapter` |
| optional-derived-page-adapter | change | #2200 | planned | planned | none | `derived_page_context_adapter` |
| production-journey | change | #2201 | planned | planned | none | `production_journey` |
