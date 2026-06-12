---
id: lumen-external-contracts
summary: External contract gates for Lumen production-scoped README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: agentic-integration
    role: primary
    gap: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    claim: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    coverage: full
    rationale: "The EC gate verifies the offline spec and llm CLI surfaces used by agents."
  - id: ops-speed
    role: primary
    gap: ops-speed-competitive-benchmark-vs-db
    claim: ops-speed-competitive-benchmark-vs-db
    coverage: full
    rationale: "The EC benchmark gate verifies lumen's standing commitment to beat Postgres on the contracted search-latency cells (the ratcheting competitive gate)."
---

# External Contracts: lumen

## Agentic Integration Offline CLI EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-offline-cli
    capability_id: agentic-integration
    contract_id: offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec emits valid OpenAPI and JSON-schema output offline."
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "lumen llm guide, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals."
```

## Competitive Search Benchmark vs DB EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-ops-speed-benchmark-vs-db
    capability_id: ops-speed
    contract_id: ops-speed-competitive-benchmark-vs-db
    category: benchmark
    command: "cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
    assertions:
      - "lumen wins the contracted search-latency cells against Postgres (text_bm25 WIN; ratcheted floor holds)."
      - "floor-dominated cells (pg btree point-lookup) stay EXEMPT, not gated."
```
