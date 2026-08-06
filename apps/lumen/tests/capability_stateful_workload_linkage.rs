// HANDWRITE-BEGIN gap="missing-generator:unit-test:d81257e0" tracker="#2144" reason="Add a deterministic structural regression test that requires the TD primary capability reference, active #2144 linkage, retained #1553 provenance, and the existing stateful capability gate. generator gap: missing-generator:test:capability-td-linkage (#2144)."
// @spec apps/lumen/tech-design/validate/link-stateful-service-workload-claim-to-primary-td-verification.md#unit-test

const CAPABILITIES: &str = include_str!("../CAPABILITIES.md");
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

/// R2 used to read the capability document, back when a capability section
/// carried `Root WI:` and `Historical WI:` lines and the risk was that the two
/// blurred into one. #1847 sent that reference the other way -- a durable
/// document never points at a work item, provenance is resolved from the WI
/// side -- and #2887 wrote the capability contract without any WI field at all.
/// So the requirement it still makes sense to hold is the one the TD can carry:
/// #2144 is the active verification-link root, #1553 is retained as history and
/// never revived as the active one, and the capability contract stays clean of
/// work-item references entirely.
#[test]
fn active_and_historical_provenance_are_distinct() {
    assert!(TD.contains("id: '2144'"), "the TD is the active root");
    assert!(
        TD.contains("retaining closed WI #1553 as historical provenance"),
        "the closed WI stays as history"
    );
    assert!(TD.contains("aw capability check --project lumen"));

    for (line_no, line) in CAPABILITIES.lines().enumerate() {
        let issue_ref = line
            .match_indices('#')
            .any(|(i, _)| line[i + 1..].starts_with(|c: char| c.is_ascii_digit()));
        assert!(
            !issue_ref,
            "the capability contract must carry no work-item reference, \
             CAPABILITIES.md:{} reads {line:?}",
            line_no + 1
        );
    }
}

// HANDWRITE-END
