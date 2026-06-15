// SPEC-MANAGED: projects/lumen/ec/stability/resilience-survival.md#lumen-stability-resilience-survival
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-stability-resilience-survival
// @capability resilience
// @claim broker-kill-pod-kill-survival
// @contract broker-kill-pod-kill-survival
// @category stability
// @required_for_production false
// @command cargo test -p lumen --test drop_drain_e2e --test reindex_stream_e2e -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn lumen_stability_resilience_survival() {
    panic!("AW EC placeholder for lumen-stability-resilience-survival");
}
// CODEGEN-END
