---
change: jet-browser-console-errors
group: browser-console-error-relay
date: 2026-04-09
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| ? | - | high | HMR WebSocket endpoint /__jet_hmr — server-to-client only; recv_task at mod.rs:485-492 drops all non-Close frames — extension point for receiving browser console messages, HMR Protocol JSON schema (oneOf) defines existing message types — new client-to-server console-message type must be added, HMR Client Runtime section documents injected JS — browser-side hooks must be added here, ServerConfig schema — no new fields needed (always-on) |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| jet-console-error-relay | create | crates/cclab-jet/logic/console-error-relay.md | overview, schema, changes |

