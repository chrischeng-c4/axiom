//! Every way a projected update can be wrong, and the name it gets (#3112 AC3).
//!
//! These are one test per operator-visible failure rather than one test with a
//! table, because the thing under test is the *name* — `wrong_identity` and
//! `untrusted` both mean "the handshake would fail", and only the distinction
//! tells the operator whether they mis-scoped a CSR or pointed at the wrong CA.
//! A collapsed assertion would pass while the taxonomy rotted.

mod support;

use std::time::SystemTime;

use peer_tls::material::{validate, IdentityExpectation, MaterialPem, RejectionReason};
use support::{authority, bundle, seconds_from_now, LeafBuilder};

fn peer_expectation() -> IdentityExpectation {
    IdentityExpectation::peer(
        ["lumen-0.lumen-peer.axiom.svc.cluster.local".to_string()],
        ["spiffe://axiom/ns/axiom/instance/lumen-0".to_string()],
    )
}

fn peer_leaf(ca: &support::Authority) -> support::Leaf {
    LeafBuilder::new()
        .dns(&["lumen-0.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-0"])
        .issue(ca)
}

#[test]
fn accepts_material_that_satisfies_every_configured_expectation() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let validated = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect("well-formed material must activate");

    // A fingerprint an operator can compare against the controller's status:
    // lowercase hex sha256, no separators.
    assert_eq!(validated.fingerprint().len(), 64);
    assert!(validated
        .fingerprint()
        .chars()
        .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));
    assert!(validated.is_valid_at(SystemTime::now()));
    assert_eq!(validated.trust_anchors().len(), 1);
}

#[test]
fn rejects_a_key_that_belongs_to_a_different_leaf() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    let other = peer_leaf(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, other.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("a mismatched key must not activate");
    assert_eq!(rejection.reason, RejectionReason::KeyMismatch);
}

#[test]
fn rejects_pem_that_does_not_decode() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    // The half-written file a two-step Secret update produces.
    let truncated = &leaf.cert_pem[..leaf.cert_pem.len() / 2];
    let pem = MaterialPem::new(truncated, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("truncated PEM must not activate");
    assert_eq!(rejection.reason, RejectionReason::MalformedPem);
}

#[test]
fn rejects_a_trust_bundle_with_no_anchors_in_it() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, "");

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("an empty bundle must not activate");
    // Not `untrusted`: trusting nothing rejects every peer, and calling that a
    // chain problem sends the operator looking at the wrong certificate.
    assert_eq!(rejection.reason, RejectionReason::EmptyTrustBundle);
}

#[test]
fn rejects_an_expired_leaf() {
    let ca = authority("axiom-peer-ca");
    let leaf = LeafBuilder::new()
        .dns(&["lumen-0.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-0"])
        .window(seconds_from_now(-7200), seconds_from_now(-3600))
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("an expired leaf must not activate");
    assert_eq!(rejection.reason, RejectionReason::Expired);
}

#[test]
fn rejects_a_leaf_whose_validity_has_not_started() {
    let ca = authority("axiom-peer-ca");
    let leaf = LeafBuilder::new()
        .dns(&["lumen-0.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-0"])
        .window(seconds_from_now(3600), seconds_from_now(7200))
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("a future leaf must not activate");
    // Distinct from `expired` on purpose: this one usually means clock skew,
    // and the two failures have completely different remediations.
    assert_eq!(rejection.reason, RejectionReason::NotYetValid);
}

#[test]
fn rejects_a_leaf_that_cannot_be_used_for_the_role_it_was_issued_for() {
    let ca = authority("axiom-peer-ca");
    // Server auth only: this member can accept a peer but never dial one, so
    // half the replication mesh would work and the operator would see an
    // asymmetric partition rather than a certificate error.
    let leaf = LeafBuilder::new()
        .dns(&["lumen-0.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-0"])
        .usages(true, false)
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("a leaf missing clientAuth must not activate on a peer port");
    assert_eq!(rejection.reason, RejectionReason::MissingUsage);
}

#[test]
fn rejects_a_leaf_signed_by_a_ca_the_bundle_does_not_contain() {
    let ours = authority("axiom-peer-ca");
    let theirs = authority("someone-elses-ca");
    let leaf = peer_leaf(&theirs);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ours.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("an unrelated CA must not activate");
    assert_eq!(rejection.reason, RejectionReason::Untrusted);
}

#[test]
fn rejects_a_valid_leaf_that_names_a_different_workload() {
    let ca = authority("axiom-peer-ca");
    // Correct CA, in date, both usages — and issued for another member. This is
    // the case a "does it parse and chain?" check waves straight through.
    let leaf = LeafBuilder::new()
        .dns(&["lumen-4.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-4"])
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("another member's identity must not activate here");
    assert_eq!(rejection.reason, RejectionReason::WrongIdentity);
}

#[test]
fn rejects_a_leaf_missing_only_the_spiffe_uri() {
    let ca = authority("axiom-peer-ca");
    let leaf = LeafBuilder::new()
        .dns(&["lumen-0.lumen-peer.axiom.svc.cluster.local"])
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect_err("the SPIFFE identity is not optional on a peer port");
    assert_eq!(rejection.reason, RejectionReason::WrongIdentity);
}

#[test]
fn rejects_a_serving_leaf_that_covers_only_some_of_the_configured_names() {
    let ca = authority("axiom-serving-ca");
    let expect = IdentityExpectation::serving([
        "lumen.axiom.svc.cluster.local".to_string(),
        "lumen.axiom.svc".to_string(),
    ]);
    // A leaf reissued for a shrunken name list still verifies against one of
    // the names. Stopping at the first success is how the fleet finds out at
    // dial time instead of at activation time.
    let leaf = LeafBuilder::new()
        .dns(&["lumen.axiom.svc.cluster.local"])
        .usages(true, false)
        .issue(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    let rejection = validate(&pem, &expect, SystemTime::now())
        .expect_err("every configured serving name must be covered");
    assert_eq!(rejection.reason, RejectionReason::WrongIdentity);
}

#[test]
fn accepts_a_leaf_against_a_bundle_that_carries_more_than_one_anchor() {
    // The steady state during an issuer rotation: the bundle holds both, and
    // only one of them signed this leaf.
    let retiring = authority("axiom-peer-ca-gen1");
    let incoming = authority("axiom-peer-ca-gen2");
    let leaf = peer_leaf(&incoming);
    let pem = MaterialPem::new(
        leaf.cert_pem,
        leaf.key_pem,
        bundle(&[&retiring, &incoming]),
    );

    let validated = validate(&pem, &peer_expectation(), SystemTime::now())
        .expect("an overlapping bundle must not be a rejection");
    assert_eq!(validated.trust_anchors().len(), 2);
}

#[test]
fn a_rejection_detail_never_carries_key_material() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    let other = peer_leaf(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, other.key_pem.clone(), ca.pem());

    let rejection = validate(&pem, &peer_expectation(), SystemTime::now()).unwrap_err();
    let text = rejection.to_string();
    assert!(!text.contains("BEGIN"), "rejection leaked a PEM block: {text}");
    assert!(
        !text.contains("PRIVATE KEY"),
        "rejection leaked key material: {text}"
    );
}

#[test]
fn debug_output_for_material_prints_sizes_rather_than_bytes() {
    let ca = authority("axiom-peer-ca");
    let leaf = peer_leaf(&ca);
    let pem = MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem());

    // Anything that formats an enclosing struct with `{:?}` — a tracing field,
    // a panic message — would otherwise print the private key.
    let text = format!("{pem:?}");
    assert!(!text.contains("BEGIN"), "Debug leaked a PEM block: {text}");
    assert!(text.contains("key_bytes"));
}
