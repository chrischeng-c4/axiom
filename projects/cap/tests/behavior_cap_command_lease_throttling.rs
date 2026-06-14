// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-command-lease-throttling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-command-lease-throttling
// @capability command-lease-throttling
// @claim lease-admission-and-process-supervision
// @contract command-lease-throttling
// @category behavior
// @required_for_production true
// @command cargo test -p cap throttle -- --nocapture && cargo test -p cap sampler -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn cap_command_lease_throttling() {
    panic!("AW EC placeholder for cap-command-lease-throttling");
}
// CODEGEN-END
