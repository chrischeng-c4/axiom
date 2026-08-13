//! Retiring a certificate authority without partitioning the fleet (#3112 R5,
//! AC4).
//!
//! An issuer rotation is not an instant. There is a window in which some members
//! present leaves signed by the outgoing CA and some by the incoming one, and
//! the only question that matters is who each member is willing to *accept*
//! during that window. Trusting only what the newest bundle names is how a
//! rotation becomes a partition; trusting the old anchor forever is how a
//! compromised CA stays useful.
//!
//! So the rule under test is: the runtime keeps the previous generation's
//! anchors after the bundle drops them, and lets go only when the certificate
//! controller says every member has activated — the same "activation observed"
//! handshake the controller already implements for leaves (#3110).
//!
//! Every assertion here is a real handshake rather than an inspection of the
//! trust store, because "would this peer be accepted" is a property of rustls'
//! verifier and not of a certificate count.

mod support;

use std::sync::Arc;

use peer_tls::reload::{MaterialSource, MemoryMaterialSource, ReloadableTls, TlsRuntimeProfile};
use peer_tls::material::MaterialPem;
use support::{authority, bundle, handshake, Authority, LeafBuilder};

const MEMBER: &str = "lumen-0.lumen-peer.axiom.svc.cluster.local";
const SPIFFE: &str = "spiffe://axiom/ns/axiom/instance/lumen-0";
const OTHER: &str = "lumen-1.lumen-peer.axiom.svc.cluster.local";
const OTHER_SPIFFE: &str = "spiffe://axiom/ns/axiom/instance/lumen-1";

fn profile() -> TlsRuntimeProfile {
    TlsRuntimeProfile::peer([MEMBER.to_string()], [SPIFFE.to_string()])
}

fn member_material(ca: &Authority, trust: String) -> MaterialPem {
    let leaf = LeafBuilder::new()
        .dns(&[MEMBER])
        .spiffe(&[SPIFFE])
        .issue(ca);
    MaterialPem::new(leaf.cert_pem, leaf.key_pem, trust)
}

/// A different member of the same fleet, dialing in. Whether it is accepted is
/// entirely a question of which CA signed it and what the *server* still
/// trusts, so `trusts` is generous here on purpose: making the dialer picky
/// would move the rejection to the wrong side of the connection and the test
/// would prove nothing about the server's trust store.
fn peer_client(ca: &Authority, trusts: &[&Authority]) -> Arc<rustls::ClientConfig> {
    let leaf = LeafBuilder::new()
        .dns(&[OTHER])
        .spiffe(&[OTHER_SPIFFE])
        .issue(ca);
    let material = MaterialPem::new(leaf.cert_pem, leaf.key_pem, bundle(trusts));
    let source = Arc::new(MemoryMaterialSource::new(material));
    let dialer = ReloadableTls::required(
        TlsRuntimeProfile::peer([OTHER.to_string()], [OTHER_SPIFFE.to_string()]),
        source as Arc<dyn MaterialSource>,
    )
    .expect("the dialing peer's own material is well-formed");
    dialer.client_config().expect("a dialer must have a config")
}

/// The rotation, staged exactly as the certificate controller drives it:
/// publish the new anchor alongside the old, then issue against it, then drop
/// the old anchor from the bundle.
struct Rotation {
    tls: ReloadableTls,
    source: Arc<MemoryMaterialSource>,
    outgoing: Authority,
    incoming: Authority,
}

fn rotation() -> Rotation {
    let outgoing = authority("axiom-peer-ca-gen1");
    let incoming = authority("axiom-peer-ca-gen2");
    let source = Arc::new(MemoryMaterialSource::new(member_material(
        &outgoing,
        outgoing.pem(),
    )));
    let tls = ReloadableTls::required(profile(), source.clone() as Arc<dyn MaterialSource>)
        .expect("the starting material is well-formed");
    assert_eq!(tls.generation(), 1);
    Rotation {
        tls,
        source,
        outgoing,
        incoming,
    }
}

impl Rotation {
    /// Step one: the bundle names both anchors and the leaf moves to the new CA.
    fn publish_overlap(&self) {
        self.source.set(member_material(
            &self.incoming,
            bundle(&[&self.outgoing, &self.incoming]),
        ));
        self.tls.reload().expect("the overlapping bundle activates");
    }

    /// Step two: the controller drops the outgoing anchor from the bundle.
    fn drop_outgoing_from_bundle(&self) {
        self.source
            .set(member_material(&self.incoming, self.incoming.pem()));
        self.tls.reload().expect("the narrowed bundle activates");
    }
}

#[tokio::test]
async fn trust_overlap_accepts_both_generations_during_the_transition() {
    let r = rotation();
    r.publish_overlap();

    assert_eq!(r.tls.generation(), 2);
    let server = r.tls.server_config().expect("serving after the rotation");

    let old_peer = handshake(
        server.clone(),
        peer_client(&r.outgoing, &[&r.outgoing, &r.incoming]),
        MEMBER,
    )
    .await;
    assert!(
        old_peer.accepted(),
        "a member still on the outgoing CA must not be cut off mid-rotation: {:?}",
        old_peer.server
    );

    let new_peer = handshake(server, peer_client(&r.incoming, &[&r.incoming]), MEMBER).await;
    assert!(
        new_peer.accepted(),
        "the incoming CA must be accepted: {:?}",
        new_peer.server
    );
}

#[tokio::test]
async fn trust_overlap_holds_the_outgoing_anchor_after_the_bundle_stops_naming_it() {
    let r = rotation();
    r.publish_overlap();
    r.drop_outgoing_from_bundle();

    let status = r.tls.status();
    assert_eq!(status.generation, 3);
    assert!(
        status.retiring_trust_anchors > 0,
        "the previous generation's anchors must be retained until activation is observed"
    );

    // The bundle on disk no longer names the outgoing CA, and a member that has
    // not been reissued yet still gets in. That is the whole point: the bundle
    // is the controller's intent, not the fleet's state.
    let outcome = handshake(
        r.tls.server_config().unwrap(),
        peer_client(&r.outgoing, &[&r.outgoing, &r.incoming]),
        MEMBER,
    )
    .await;
    assert!(
        outcome.accepted(),
        "dropping the anchor from the bundle must not immediately cut off the fleet: {:?}",
        outcome.server
    );
}

#[tokio::test]
async fn trust_overlap_rejects_the_old_generation_once_activation_is_observed() {
    let r = rotation();
    r.publish_overlap();
    r.drop_outgoing_from_bundle();

    assert!(
        r.tls.retire_previous_trust(3),
        "retiring the generation that is actually active must succeed"
    );
    assert_eq!(r.tls.status().retiring_trust_anchors, 0);
    assert_eq!(
        r.tls.status().trust_anchors,
        1,
        "only the incoming anchor should be left"
    );

    let server = r.tls.server_config().unwrap();
    let retired = handshake(
        server.clone(),
        peer_client(&r.outgoing, &[&r.outgoing, &r.incoming]),
        MEMBER,
    )
    .await;
    assert!(
        retired.server.is_err(),
        "a leaf from the retired CA must no longer be accepted"
    );

    let current = handshake(server, peer_client(&r.incoming, &[&r.incoming]), MEMBER).await;
    assert!(
        current.accepted(),
        "retirement must not disturb the current generation: {:?}",
        current.server
    );
}

#[tokio::test]
async fn trust_overlap_retirement_naming_a_stale_generation_changes_nothing() {
    let r = rotation();
    r.publish_overlap();

    // The controller observed generation 1 and only now got around to saying so.
    // Acting on it would retire trust for a rotation that has already moved on.
    assert!(!r.tls.retire_previous_trust(1));
    assert!(
        r.tls.status().retiring_trust_anchors > 0,
        "a stale retirement must not drop anything"
    );

    let outcome = handshake(
        r.tls.server_config().unwrap(),
        peer_client(&r.outgoing, &[&r.outgoing, &r.incoming]),
        MEMBER,
    )
    .await;
    assert!(outcome.accepted(), "{:?}", outcome.server);
}

#[tokio::test]
async fn trust_overlap_survives_a_second_rotation_before_the_first_was_observed() {
    let r = rotation();
    r.publish_overlap();

    // A third CA arrives while the first retirement is still outstanding. The
    // runtime must not lose the anchor it was holding on behalf of members that
    // have not been reissued.
    let third = authority("axiom-peer-ca-gen3");
    r.source.set(member_material(
        &third,
        bundle(&[&r.incoming, &third]),
    ));
    r.tls.reload().expect("the second rotation activates");

    let server = r.tls.server_config().unwrap();
    for (label, ca) in [
        ("first generation", &r.outgoing),
        ("second generation", &r.incoming),
        ("third generation", &third),
    ] {
        let outcome = handshake(server.clone(), peer_client(ca, &[ca, &third]), MEMBER).await;
        assert!(
            outcome.accepted(),
            "{label} must still be accepted while retirement is outstanding: {:?}",
            outcome.server
        );
    }

    r.tls.retire_previous_trust(r.tls.generation());
    let dropped = handshake(
        r.tls.server_config().unwrap(),
        peer_client(&r.outgoing, &[&r.outgoing, &third]),
        MEMBER,
    )
    .await;
    assert!(
        dropped.server.is_err(),
        "one retirement must clear every carried anchor, not just the newest"
    );
}

#[tokio::test]
async fn trust_overlap_never_lets_an_unrelated_authority_in() {
    let r = rotation();
    r.publish_overlap();

    // Carrying old anchors widens what is accepted, so the test that it does not
    // widen it to *everything* has to exist next to the ones that want it wide.
    let stranger = authority("someone-elses-ca");
    let outcome = handshake(
        r.tls.server_config().unwrap(),
        peer_client(&stranger, &[&stranger, &r.incoming]),
        MEMBER,
    )
    .await;
    assert!(
        outcome.server.is_err(),
        "trust overlap is two generations of one CA, not an open door"
    );
}
