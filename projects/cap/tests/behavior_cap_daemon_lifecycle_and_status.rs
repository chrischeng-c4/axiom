// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-daemon-lifecycle-and-status
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-daemon-lifecycle-and-status
// @capability daemon-lifecycle-and-status
// @claim daemon-process-lifecycle
// @contract daemon-lifecycle-and-status
// @category behavior
// @required_for_production true
// @command cargo test -p cap daemon -- --nocapture && cargo test -p cap cli -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn cap_daemon_lifecycle_and_status() {
    panic!(
        "AW EC placeholder for {}",
        "cap-daemon-lifecycle-and-status"
    );
}
// CODEGEN-END
