---
change_id: envfile-support
type: spec_context
created_at: 2026-02-10T02:27:31.807946+00:00
updated_at: 2026-02-10T02:27:31.807946+00:00
iteration: 1
complexity: medium
stage: spec
scanned_groups:
  - cclab-aurora
  - cclab-cli
  - cclab-core
  - cclab-genesis
  - cclab-grid
  - cclab-grid-db
  - cclab-ion
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-titan
  - genesis
  - nebula
---

# Spec Context

## Relevant Specs

- **shield-settings-management** (group: cclab-shield)
  - relevance: high
  - reason: Defines the standard pattern for .env support and environment variable loading in the project using dotenvy.
  - key sections: Requirements R2 (Support .env files), Diagrams (Settings Loading Flow)
- **orchestrator** (group: cclab-genesis)
  - relevance: high
  - reason: Describes how agents are configured and executed, which is where the envfile support needs to be integrated.
  - key sections: Requirements R2 (Workflow Stage Configuration), Architecture (AgentRunner), Configuration (config.toml structure)
- **run-change** (group: cclab-genesis)
  - relevance: medium
  - reason: Main workflow entry point that uses the configuration and orchestrator; needs to be aware of the new config fields.
  - key sections: Requirements R5 (Modular code structure), Action ↔ Phase ↔ Agent Mapping

## Dependencies

- cclab-shield/shield-settings-management
- cclab-genesis/orchestrator
- cclab-genesis/run-change

## Gaps

- GenesisConfig (src/models/config.rs) lacks 'envfile' fields in [workflow], [gemini], [codex], and [claude] sections.
- AgentRunner (src/orchestrator/agent_runner.rs) does not load environment variables from configured .env files before spawning subprocesses.
- Variable substitution support as defined in clarifications.md is not yet implemented in the configuration loading logic.
