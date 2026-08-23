//! Rotating from one certificate authority to another without an outage (#3110 R5).
//!
//! The property under test is an *ordering*, and orderings are the kind of thing
//! that pass by accident. So each test asserts the sequence of actions, not just
//! its endpoint: a rotation that reached the right final state by retiring the
//! outgoing anchor first would be indistinguishable from a correct one once it
//! finished, and would have dropped every in-flight connection on the way.

mod certificate_support;

use certificate_support::*;
use service_k8s::certificate::issuer::IssuerId;
use service_k8s::certificate::profile::Purpose;
use service_k8s::certificate::reconcile::{Reconciler, RuntimeReport};
use service_k8s::certificate::state::{Action, IssueReason};

/// Bring an instance to steady state on `issuer`, returning the harness.
fn settled_on(issuer_id: &str) -> (Harness, String) {
    let harness = Harness::new(issuer_id, start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
    drive(
        &reconciler,
        &profile,
        &RuntimeReport::default(),
        start(),
        6,
    );
    let fingerprint = harness
        .projected_fingerprint(Purpose::Peer)
        .expect("a leaf was projected");
    (harness, fingerprint)
}

#[test]
fn certificate_rotation_publishes_trust_before_it_issues_anything() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    let actions = drive(
        &reconciler,
        &profile,
        &RuntimeReport::default(),
        start(),
        6,
    );

    assert!(
        matches!(actions[0], Action::PublishTrustBundle { .. }),
        "first action was {:?}; a leaf from an issuer the fleet does not trust yet fails at \
         handshake time on the far side",
        actions[0]
    );
    assert!(matches!(
        actions[1],
        Action::Issue {
            reason: IssueReason::Bootstrap,
            ..
        }
    ));
    assert!(matches!(actions[2], Action::Wait { .. }));
}

#[test]
fn certificate_rotation_adds_the_new_anchor_before_the_new_leaf() {
    let (harness_a, _) = settled_on("pool-a");
    // Same store, new issuer: this is an operator moving the instance to a
    // different pool.
    let issuer_b = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-b", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness_a.store, &issuer_b);

    let first = step(&reconciler, &profile, &RuntimeReport::default(), start());
    assert!(
        matches!(first.action, Action::PublishTrustBundle { .. }),
        "got {:?}",
        first.action
    );
    let bundle = harness_a.trust_bundle_pem(Purpose::Peer);
    assert_eq!(
        bundle.matches("BEGIN CERTIFICATE").count(),
        2,
        "both anchors must be trusted at once; that overlap is the whole rotation"
    );

    let second = step(&reconciler, &profile, &RuntimeReport::default(), start());
    assert_eq!(
        second.action,
        Action::Issue {
            issuer: IssuerId::new("pool-b"),
            reason: IssueReason::IssuerRotation,
        }
    );
}

#[test]
fn certificate_rotation_waits_for_the_runtime_before_retiring_the_old_anchor() {
    let (harness_a, old_fingerprint) = settled_on("pool-a");
    let issuer_b = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-b", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness_a.store, &issuer_b);

    // Publish, then issue.
    step(&reconciler, &profile, &RuntimeReport::default(), start());
    step(&reconciler, &profile, &RuntimeReport::default(), start());
    let new_fingerprint = harness_a
        .projected_fingerprint(Purpose::Peer)
        .expect("new leaf projected");
    assert_ne!(new_fingerprint, old_fingerprint);

    // The workload has not reloaded yet: it is still presenting the old leaf,
    // which the old anchor is what verifies.
    let stale = step(
        &reconciler,
        &profile,
        &activated(Some(old_fingerprint.clone())),
        start(),
    );
    assert!(
        matches!(stale.action, Action::AwaitActivation { .. }),
        "got {:?}; retiring here would break every connection the old leaf is authenticating",
        stale.action
    );
    assert_eq!(
        harness_a
            .trust_bundle_pem(Purpose::Peer)
            .matches("BEGIN CERTIFICATE")
            .count(),
        2
    );

    // Now it reports the new one.
    let retire = step(
        &reconciler,
        &profile,
        &activated(Some(new_fingerprint.clone())),
        start(),
    );
    assert_eq!(
        retire.action,
        Action::RetireIssuers {
            issuers: vec![IssuerId::new("pool-a")]
        }
    );
    assert_eq!(
        harness_a
            .trust_bundle_pem(Purpose::Peer)
            .matches("BEGIN CERTIFICATE")
            .count(),
        1,
        "the outgoing anchor is dropped only after nothing is using it"
    );

    let settled = step(
        &reconciler,
        &profile,
        &activated(Some(new_fingerprint)),
        start(),
    );
    assert!(matches!(settled.action, Action::Wait { .. }));
}

#[test]
fn certificate_rotation_keeps_the_serving_material_when_a_step_fails() {
    let (harness, fingerprint) = settled_on("pool-a");
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    // Renewal is due, and the CA is unreachable.
    harness.issuer.fail_next("CA unreachable");
    let renewal_due = plus_hours(start(), 11);
    let failed = futures::executor::block_on(reconciler.reconcile(
        &profile,
        &activated(Some(fingerprint.clone())),
        renewal_due,
    ));
    assert!(failed.is_err(), "the CA refused; the reconcile must not succeed");

    assert_eq!(
        harness.projected_fingerprint(Purpose::Peer).as_deref(),
        Some(fingerprint.as_str()),
        "a failed renewal must leave the leaf that is currently serving traffic exactly where \
         it was"
    );
    let keys = harness.projected_keys(Purpose::Peer);
    for key in ["tls.crt", "tls.key", "ca.crt"] {
        assert!(keys.iter().any(|k| k == key), "lost {key} on a failed step");
    }
}

#[test]
fn certificate_rotation_survives_being_interrupted_between_steps() {
    let (harness, _) = settled_on("pool-a");
    let issuer_b = service_k8s::certificate::ephemeral::EphemeralIssuer::new("pool-b", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();

    // Publish the new anchor, then throw the controller away.
    {
        let reconciler = Reconciler::new(&scope, &owner, &harness.store, &issuer_b);
        step(&reconciler, &profile, &RuntimeReport::default(), start());
    }

    // A fresh controller, with no memory of the above, resumes at the next step
    // rather than at the first one.
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &issuer_b);
    let resumed = step(&reconciler, &profile, &RuntimeReport::default(), start());
    assert_eq!(
        resumed.action,
        Action::Issue {
            issuer: IssuerId::new("pool-b"),
            reason: IssueReason::IssuerRotation,
        },
        "the sequence is reconstructed from what is in the cluster, so a restart is not a \
         reason to start over"
    );
}

#[test]
fn certificate_rotation_leaves_the_other_purpose_alone() {
    let harness = Harness::new("pool-a", start());
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(
        &reconciler,
        &peer_profile(),
        &RuntimeReport::default(),
        start(),
        6,
    );
    let peer = harness
        .projected_fingerprint(Purpose::Peer)
        .expect("peer leaf");

    drive(
        &reconciler,
        &serving_profile(),
        &RuntimeReport::default(),
        start(),
        6,
    );

    assert_eq!(
        harness.projected_fingerprint(Purpose::Peer).as_deref(),
        Some(peer.as_str()),
        "serving and peer identity are separate by design (#2890); one lifecycle must not \
         reissue the other's material"
    );
    assert_ne!(
        harness.projected_fingerprint(Purpose::Serving).as_deref(),
        Some(peer.as_str())
    );
}

#[test]
fn certificate_rotation_does_not_reissue_when_nothing_changed() {
    let (harness, fingerprint) = settled_on("pool-a");
    let issued = harness.issuer.issued_count();
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    for _ in 0..5 {
        step(
            &reconciler,
            &profile,
            &activated(Some(fingerprint.clone())),
            plus_hours(start(), 1),
        );
    }
    assert_eq!(
        harness.issuer.issued_count(),
        issued,
        "a steady-state reconcile that mints a certificate is invisible in production -- two \
         valid leaves look exactly like one"
    );
}
