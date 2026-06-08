---
change_id: genesis-fetch-issues
type: spec_context
created_at: 2026-02-11T17:04:59.919533+00:00
updated_at: 2026-02-11T17:04:59.919533+00:00
iteration: 2
complexity: high
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

- **fetch-issues** (group: cclab-genesis)
  - relevance: high
  - reason: Core specification for the genesis_fetch_issues tool and its integration into run_change.
  - key sections: OpenRPC, Fetch Flow, Dependency Extraction, STATE.yaml DAG Section, Per-Issue Clarification Loop
- **delegate-agent** (group: cclab-genesis)
  - relevance: high
  - reason: Defines the verified agent dispatch mechanism used by the clarification and context loops.
  - key sections: Verification Table, Sequence Diagram
- **create-clarifications** (group: cclab-genesis)
  - relevance: medium
  - reason: Tool called during the clarification loop per issue.
  - key sections: Overview
- **create-spec-context** (group: cclab-genesis)
  - relevance: medium
  - reason: Tool called during the cumulative context loop per issue.
  - key sections: Overview
- **agent-tool** (group: genesis)
  - relevance: medium
  - reason: Underlying tool for LLM agent invocation used by delegate-agent.
  - key sections: R4 - Execution Flow

## Dependencies

- cclab-genesis/fetch-issues depends on STATE.yaml schema extension
- run_change depends on genesis_fetch_issues for issue resolution
- clarify loop depends on dag.clarify_index in STATE.yaml
- context loop depends on dag.context_index in STATE.yaml

## Gaps

- run_change needs logic to parse description for issue refs and return next: fetch_issues action
- run_change needs to handle topological loop for clarify and context phases using dag counters
- STATE.yaml schema needs to include the new 'dag' section structure
