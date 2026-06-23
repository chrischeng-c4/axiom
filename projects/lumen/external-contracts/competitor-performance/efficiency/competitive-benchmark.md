---
id: lumen-competitor-performance-ec
summary: Competitive performance — meter wraps the lumen-vs-Postgres/OpenSearch perf gate for every gated search cell at ratcheted floors; rig owns request/load scenarios; arena is retired from production EC dispatch.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Performance

Competitive latency gate: lumen must beat **Postgres and OpenSearch only** on
every contracted search cell at the per-cell ratcheted floor
(`perf-baseline.json`, ratchet 0.8). The meter-wrapped cargo perf gate is the
pass/fail and resource-evidence surface; rig owns request/query load scenarios
and pins. Arena is legacy-only during retirement and is not a production EC
tool for Lumen efficiency.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-competitor-performance-competitive
    capability_id: competitor-performance
    claim_id: competitive-regression-gate-beat-pg-os-per-cell-ratcheting
    contract_id: search-efficiency-filtering-ranking-pagination
    category: efficiency
    test_path: projects/lumen/tests/benchmark_lumen_competitor_performance_competitive.rs
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
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
  - id: lumen-meter-search-efficiency
    tool: meter
    manifest: meter-search-efficiency.toml
    category: efficiency
    command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
    native:
      version: 1
      project: lumen
      source_contract: lumen-competitor-performance-competitive
      delegate_command: "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter"
```
