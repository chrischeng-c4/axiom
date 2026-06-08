---
change_id: project-config-and-prism-index
type: spec_context
created_at: 2026-02-10T06:04:55.902540+00:00
updated_at: 2026-02-10T06:04:55.902540+00:00
iteration: 2
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

- **init** (group: cclab-cli)
  - relevance: high
  - reason: Defines how projects are initialized and their directory structure.
  - key sections: 建立結構, 專案類型偵測
- **prism-init-spec** (group: cclab-server)
  - relevance: high
  - reason: Defines automatic indexing of projects at server startup.
  - key sections: R2 - Background Initialization, Scenario: Persistent Projects Load
- **shield-settings-management** (group: cclab-shield)
  - relevance: medium
  - reason: Provides the base for configuration management.
  - key sections: R1 - Load from Environment Variables, R2 - Support .env files
- **prism-pdg-mcp-tools** (group: cclab-prism)
  - relevance: medium
  - reason: Defines the tools that rely on indexing.
  - key sections: R101 - prism_pdg Tool

## Dependencies

- cclab-server/prism-init-spec depends on cclab-cli/init for project setup
- cclab-server/prism-init-spec uses a registry that could be populated/managed by cclab-cli commands

## Gaps

- cclab-cli/init does not explicitly mention creating a cclab/config.toml file, which is shown in the overall Genesis directory layout.
- Missing spec for the structure and content of cclab/config.toml.
- Lack of clear integration between cclab/config.toml and the server-side prism-init-spec (which currently uses a global registry).
