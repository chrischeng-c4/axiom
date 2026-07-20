// HANDWRITE-BEGIN gap="missing-generator:unit-test:d81257e0" tracker="#2144" reason="Add a deterministic structural regression test that requires the TD primary capability reference, active #2144 linkage, retained #1553 provenance, and the existing stateful capability gate. generator gap: missing-generator:test:capability-td-linkage (#2144)."
// @spec apps/lumen/tech-design/validate/link-stateful-service-workload-claim-to-primary-td-verification.md#unit-test

const README: &str = include_str!("../README.md");
const TD: &str = include_str!(
    "../tech-design/validate/link-stateful-service-workload-claim-to-primary-td-verification.md"
);

#[test]
fn primary_td_linkage_is_bound() {
    assert!(TD.contains("id: \"stateful-service-workload\""));
    assert!(TD.contains("role: primary"));
    assert!(TD.contains("gap: \"stateful-service-workload-projection\""));
    assert!(TD.contains("claim: \"stateful-service-workload-projection\""));
    assert!(TD.contains("coverage: full"));
}

#[test]
fn active_and_historical_provenance_are_distinct() {
    let section = README
        .split("### Stateful Service Workload")
        .nth(1)
        .expect("stateful capability section")
        .split("### Developer & Agent Experience")
        .next()
        .expect("stateful capability section boundary");

    assert!(section.contains("Root WI: #2144"));
    assert!(section.contains("Historical WI: #1553"));
    assert!(section.contains("aw capability check --project lumen"));
    assert!(!section.contains("Root WI: #1553"));
}

// HANDWRITE-END
