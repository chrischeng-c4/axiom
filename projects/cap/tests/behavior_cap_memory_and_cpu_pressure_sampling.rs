// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-memory-and-cpu-pressure-sampling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-memory-and-cpu-pressure-sampling
// @capability command-lease-throttling
// @claim memory-and-cpu-pressure-sampling
// @contract memory-and-cpu-pressure-sampling
// @category behavior
// @required_for_production true
// @command cargo test -p cap sampler -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn cap_memory_and_cpu_pressure_sampling() {
    panic!("AW EC placeholder for cap-memory-and-cpu-pressure-sampling");
}
// CODEGEN-END
