// SPEC-MANAGED: projects/lumen/tech-design/logic/external-contracts.md#lumen-ops-speed-benchmark-vs-db
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-ops-speed-benchmark-vs-db
// @capability ops-operability
// @claim competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @contract competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @category benchmark
// @required_for_production false
// @command cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn lumen_ops_speed_benchmark_vs_db() {
    panic!("AW EC placeholder for lumen-ops-speed-benchmark-vs-db");
}
// CODEGEN-END
