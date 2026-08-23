//! Provenance on the stateful-workload capability claim: the capability
//! contract carries no work-item reference.
//!
//! Blurring the two is the failure this guards. `README.md`'s `## Capabilities`
//! section is a durable document, and #1847 fixed the direction of the
//! reference: a durable document never points at a work item, provenance is
//! resolved from the WI side. #2887 then wrote the capability contract with no
//! WI field at all. So a `#<digits>` anywhere in that section is a regression
//! to the shape those two issues removed.
//!
//! The case is structural: it reads the declared contract instead of running
//! anything, because a broken provenance chain changes no runtime behaviour and
//! would otherwise be visible only in review.
//!
//! It used to hold a second half, `primary_td_linkage_is_bound`, which read
//! `tech-design/validate/link-stateful-service-workload-claim-to-primary-td-verification.md`
//! and asserted that #2144 was the active verification-link root and #1553 was
//! retained as history. S4 of the TD/EC retirement deleted that document, and
//! with it the `aw capability check --project lumen` command it named, so the
//! half that read it is gone rather than rewritten: there is no surviving
//! artifact that states the linkage for a test to bind to.
// HANDWRITE-BEGIN gap="missing-generator:unit-test:d81257e0" tracker="#2144" reason="Add a deterministic structural regression test that requires the TD primary capability reference, active #2144 linkage, retained #1553 provenance, and the existing stateful capability gate. generator gap: missing-generator:test:capability-td-linkage (#2144)."

const README: &str = include_str!("../README.md");

#[test]
fn capability_contract_carries_no_work_item_reference() {
    // The contract is `## Capabilities` and nothing else in the file. Scanning
    // all of README would fail on the evidence sections below it, which cite
    // the runs and the issues that produced them and are supposed to.
    let mut inside = false;
    let mut scanned = 0usize;
    for (line_no, line) in README.lines().enumerate() {
        if line.starts_with("## ") {
            inside = line.trim() == "## Capabilities";
        }
        if !inside {
            continue;
        }
        scanned += 1;
        let issue_ref = line
            .match_indices('#')
            .any(|(i, _)| line[i + 1..].starts_with(|c: char| c.is_ascii_digit()));
        assert!(
            !issue_ref,
            "the capability contract must carry no work-item reference, \
             README.md:{} reads {line:?}",
            line_no + 1
        );
    }
    // A slice that resolved to nothing would pass every assertion above having
    // read no contract at all.
    assert!(
        scanned > 100,
        "the `## Capabilities` section resolved to {scanned} lines; the contract is longer"
    );
}

// HANDWRITE-END
