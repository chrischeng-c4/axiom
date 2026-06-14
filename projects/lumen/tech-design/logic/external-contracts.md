---
id: lumen-external-contracts
summary: External contract gates for Lumen production-scoped README capabilities.
fill_sections: [e2e-test]
capability_refs:
  - id: agentic-integration
    role: primary
    gap: lumen-spec-schema-openapi-json-schema-offline
    claim: lumen-spec-schema-openapi-json-schema-offline
    coverage: full
    rationale: "The EC gate verifies the offline OpenAPI and JSON-schema surfaces used by agents."
  - id: agentic-integration
    role: primary
    gap: query-shape-cookbook-field-analyzer-catalog
    claim: query-shape-cookbook-field-analyzer-catalog
    coverage: full
    rationale: "The EC gate verifies the offline query-shape, field, analyzer, and vector-metric catalogs used by agents."
  - id: agentic-integration
    role: primary
    gap: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    claim: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    coverage: full
    rationale: "The EC gate verifies the offline spec and llm CLI surfaces used by agents."
  - id: ops-operability
    role: primary
    gap: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    claim: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    coverage: full
    rationale: "The EC benchmark gate verifies lumen's standing commitment to beat Postgres on the contracted search-latency cells (the ratcheting competitive gate)."
  - id: security-auth
    role: primary
    gap: bearer-token-auth-lumen-auth
    claim: bearer-token-auth-lumen-auth
    coverage: full
    rationale: "The EC security gate verifies bearer-token auth is enforced on every API route."
  - id: security-auth
    role: primary
    gap: role-based-authz-matrix-per-route
    claim: role-based-authz-matrix-per-route
    coverage: full
    rationale: "The EC security gate verifies the per-route RBAC authz matrix is enforced on every API route."
  - id: resilience
    role: primary
    gap: broker-kill-pod-kill-survival
    claim: broker-kill-pod-kill-survival
    coverage: full
    rationale: "The EC stability gate verifies search survives a network partition and 5% packet loss within the latency budget; dispatched to rig (resilience scenarios over toxiproxy) by the ec.stability binding, with a cargo recovery/drain fallback."
---

# External Contracts: lumen

The four cases below are the EC-gatekeeper dimensions for lumen — one per
"對外不出事" guarantee. Each binds a TD contract to a verify command that
`aw health --verify-ec` dispatches (the `ec.*` map in `.aw/config.toml` swaps
the cargo fallback for an external tool where one is bound):

| Guarantee | category | gate |
|---|---|---|
| 功能 functionality | `behavior` | cargo `spec_cli` (offline OpenAPI/llm contract) |
| 效能 performance | `benchmark` | **arena** vs Postgres (ratcheting WIN gate) |
| 安全 security | `security` | cargo `auth_e2e` + `authz_matrix_e2e` (bearer + RBAC) |
| 穩定 stability | `stability` | **rig** resilience scenarios over toxiproxy |

The benchmark/stability gates are external measurement gates: run them against a
host-provisioned lumen + toxiproxy + a parity Postgres `lumenbench`, ideally on a
quiet/isolated host (their p99/ratchet budgets are sensitive to co-located CPU
and memory pressure — see the README "isolated load hosts" note). The same two
workloads are also driven self-provisioning as `vat run` runners in
`projects/lumen/vat.toml` (one spec, two surfaces).

## Agentic Integration Offline CLI EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-offline-cli
    capability_id: agentic-integration
    claim_id: lumen-spec-schema-openapi-json-schema-offline
    contract_id: offline-cli-agent-onboarding
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec emits valid OpenAPI and JSON-schema output offline."
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "lumen llm guide, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals."
```

## Agentic Integration Query Catalog EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-query-catalog
    capability_id: agentic-integration
    claim_id: query-shape-cookbook-field-analyzer-catalog
    contract_id: query-shape-cookbook-field-analyzer-catalog
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs."
      - "agent-facing query catalog output remains deterministic and offline."
```

## Agentic Integration LLM Playbook EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-agentic-integration-llm-playbook
    capability_id: agentic-integration
    claim_id: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    contract_id: lumen-llm-agent-integration-playbook-guide-quickstart-recipes
    category: behavior
    command: "cargo test -p lumen --test spec_cli -- --nocapture"
    assertions:
      - "lumen llm guide, quickstart, and recipes preserve the ingest-search-hydrate agent workflow."
      - "agent-facing playbook output remains deterministic and offline."
```

## Competitive Search Benchmark vs DB EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-ops-speed-benchmark-vs-db
    capability_id: ops-operability
    claim_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    contract_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    category: benchmark
    required_for_production: false
    command: "cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
    assertions:
      - "lumen wins the contracted search-latency cells against Postgres (text_bm25 WIN; ratcheted floor holds)."
      - "floor-dominated cells (pg btree point-lookup) stay EXEMPT, not gated."
```

## Security Auth Bearer RBAC EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-security-auth-bearer-rbac
    capability_id: security-auth
    claim_id: bearer-token-auth-lumen-auth
    contract_id: bearer-token-auth-lumen-auth
    category: security
    required_for_production: false
    command: "cargo test -p lumen --test auth_e2e --test authz_matrix_e2e -- --nocapture"
    assertions:
      - "Bearer-token auth rejects missing and invalid tokens when LUMEN_AUTH=required; accepts valid tokens."
      - "Per-route RBAC authz matrix enforces each token's role permissions on every API route (read vs write vs admin)."
```

## Stability Resilience Survival EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-stability-resilience-survival
    capability_id: resilience
    claim_id: broker-kill-pod-kill-survival
    contract_id: broker-kill-pod-kill-survival
    category: stability
    required_for_production: false
    command: "cargo test -p lumen --test drop_drain_e2e --test reindex_stream_e2e -- --nocapture"
    assertions:
      - "Search p99 stays within 2x baseline under 5% packet loss (toxiproxy timeout toxic; rig resilience scenario)."
      - "Search survives a full network partition and recovers within budget; post-recovery p99 stays within 2x baseline."
```
