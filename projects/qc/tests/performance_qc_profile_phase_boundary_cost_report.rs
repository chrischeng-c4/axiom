// SPEC-MANAGED: projects/qc/tech-design/logic/external-contracts.md#qc-profile-phase-boundary-cost-report
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec qc-profile-phase-boundary-cost-report
// @capability rust-performance-check
// @contract profile-phase-boundary-cost-report
// @category performance
// @command cargo run -p qc-cli --bin qc -- profile --phases projects/qc/tests/fixtures/profile_phase_breakdown.json
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn qc_profile_phase_boundary_cost_report() {
    panic!(
        "AW EC placeholder for {}",
        "qc-profile-phase-boundary-cost-report"
    );
}
// CODEGEN-END
