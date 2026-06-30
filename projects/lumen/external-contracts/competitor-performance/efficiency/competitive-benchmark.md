---
id: lumen-competitor-performance-ec
summary: Competitive performance — meter wraps the Lumen-only regression gate against retained pg/OpenSearch-calibrated floors; explicit calibration runners refresh peer ratios when benchmark cells or peer configs change.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive Performance

Competitive latency gate: Lumen must hold its own per-cell latency floors every
run. Postgres/OpenSearch ratios in `perf-baseline.json` are retained calibration
evidence; they are refreshed only by explicit compare runs
(`ec-efficiency-meter-calibrate` / `ec-efficiency-meter-soak`) when benchmark
cells, peer versions, or peer configs change. The meter-wrapped cargo perf gate
is the pass/fail and resource-evidence surface; rig owns request/query load
scenarios and pins. Arena is legacy-only during retirement and is not a
production EC tool for Lumen efficiency.

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
      - "Lumen-only default gate holds per-cell e2e/engine latency floors from perf-baseline.json without provisioning pg/OpenSearch."
      - "Retained pg/OpenSearch ratios remain the calibrated competitive evidence; explicit compare runners refresh them only when cells or peer configs change."
      - "FILTERING/RANKING/PAGINATION/SORT cells still execute through the same release-mode Lumen search path and qps pin."
      - "Peer floors are ratcheted (perf-baseline.json, 0.8) and btree point-lookup cells stay EXEMPT unless LUMEN_GATE_COMPARE_PEERS=1 is explicitly set."
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
