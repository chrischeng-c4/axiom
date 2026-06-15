---
id: lumen-competitive-search-benchmark-vs-db-ec
summary: Efficiency contract for competitive search benchmark evidence.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Search Benchmark vs DB

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-ops-speed-benchmark-vs-db
    capability_id: ops-operability
    claim_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    contract_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    category: efficiency
    test_path: projects/lumen/tests/benchmark_lumen_ops_speed_benchmark_vs_db.rs
    required_for_production: false
    command: "cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
    assertions:
      - "lumen wins the contracted search-latency cells against Postgres (text_bm25 WIN; ratcheted floor holds)."
      - "floor-dominated cells (pg btree point-lookup) stay EXEMPT, not gated."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-arena-competitive-search
    tool: arena
    manifest: arena.toml
    category: efficiency
    command: "target/debug/arena run --spec projects/arena/examples/lumen-vs-pg-and-opensearch.toml"
    native:
      version: 1
      project: lumen
      source_contract: lumen-ops-speed-benchmark-vs-db
      delegate_command: "target/debug/arena run --spec projects/arena/examples/lumen-vs-pg-and-opensearch.toml"
```
