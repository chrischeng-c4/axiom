---
id: lumen-search-efficiency-ec
summary: Search efficiency — lumen beats Postgres + OpenSearch on every gated search cell (filtering / ranking / pagination) at ratcheted floors; lean under load.
fill_sections: [e2e-test, tool-contract]
---

# EC: Search Efficiency (filtering · ranking · pagination)

Competitive latency gate: lumen must beat **Postgres and OpenSearch only** on
every contracted search cell at the per-cell ratcheted floor
(`perf-baseline.json`, ratchet 0.8). The cargo perf gate is the pass/fail; arena
owns the head-to-head comparison report; meter adds the resource (peak-RSS / CPU)
evidence so "efficient" means fast **and** lean.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-search-efficiency-competitive
    capability_id: search
    claim_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    contract_id: search-efficiency-filtering-ranking-pagination
    category: efficiency
    test_path: projects/lumen/tests/benchmark_lumen_search_efficiency_competitive.rs
    command: "cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
    assertions:
      - "FILTERING: filtered_search (AND[BM25+term+range]) beats pg >= 3.73x and OpenSearch(disk) >= 2.4x; filtered_knn beats pg >= 2.4x (OS exempt, no kNN plugin)."
      - "RANKING: text_bm25 single-term beats pg >= 14.56x; text_and multi-term beats pg >= 1.47x (OS >= 2.4x each)."
      - "PAGINATION/SORT: pure_sort (scan + sort) beats pg >= 18.32x (OS >= 2.4x); cursor pagination stays within the search_qps pin."
      - "Floors are ratcheted (perf-baseline.json, 0.8); btree point-lookup cells stay EXEMPT, not gated. lumen vs pg/OS only."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: lumen-arena-search-efficiency
    tool: arena
    manifest: arena.toml
    category: efficiency
    command: "target/debug/arena run --spec projects/arena/examples/lumen-vs-pg-and-opensearch.toml"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-efficiency-competitive
      delegate_command: "target/debug/arena run --spec projects/arena/examples/lumen-vs-pg-and-opensearch.toml"
  - id: lumen-meter-search-efficiency
    tool: meter
    manifest: meter-search-efficiency.toml
    category: efficiency
    command: "meter test -- cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
    native:
      version: 1
      project: lumen
      source_contract: lumen-search-efficiency-competitive
      delegate_command: "meter test -- cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1"
```
