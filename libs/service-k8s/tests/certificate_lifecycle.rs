//! Bounded issuance and restart-safe renewal (#3110 R1-R4).
//!
//! These run against the in-process signer, which is the point: none of the
//! properties here are about GCP. They are about *when* a certificate is
//! requested, *how long* it may live, and what happens to a controller that
//! restarts in the middle. Checking them in a cloud gate would mean checking
//! them occasionally.

mod certificate_support;

use std::time::Duration;

use certificate_support::*;
use service_k8s::certificate::profile::{
    CertificateIdentity, CertificateProfile, InstanceScope, ProfileError, Purpose,
    MAX_LIFETIME_SECS, MIN_LIFETIME_SECS,
};
use service_k8s::certificate::projection::parse_leaf;
use service_k8s::certificate::reconcile::{Reconciler, RuntimeReport};
use service_k8s::certificate::state::{renew_at, Action, IssueReason};

#[test]
fn a_bootstrapped_instance_ends_up_with_all_three_projected_keys() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);

    let keys = harness.projected_keys(Purpose::Peer);
    for key in ["tls.crt", "tls.key", "ca.crt"] {
        assert!(
            keys.iter().any(|k| k == key),
            "missing {key}; #2890's peer volume reads all three by name"
        );
    }
}

#[test]
fn a_leaf_lives_exactly_as_long_as_the_profile_asked_for() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);

    let secret = harness
        .store
        .get("lumen", &scope.secret_name(Purpose::Peer))
        .expect("secret");
    let pem = String::from_utf8(secret.data["tls.crt"].clone()).unwrap();
    let facts = parse_leaf(&pem).unwrap();
    assert_eq!(facts.not_before, start());
    assert_eq!(
        facts.not_after,
        start() + chrono::Duration::from_std(LIFETIME).unwrap()
    );
}

#[test]
fn a_lifetime_outside_the_bounds_is_refused_before_a_csr_exists() {
    for seconds in [MIN_LIFETIME_SECS - 1, MAX_LIFETIME_SECS + 1, 0] {
        let error = CertificateProfile::new(
            &scope(),
            Purpose::Serving,
            "lumen.lumen.svc.cluster.local",
            CertificateIdentity {
                dns_names: vec!["lumen.lumen.svc.cluster.local".into()],
                spiffe_uri: None,
            },
            Duration::from_secs(seconds),
            Duration::from_secs(900),
            Duration::ZERO,
        )
        .expect_err("accepted a {seconds}s lifetime");
        assert!(
            matches!(error, ProfileError::LifetimeOutOfBounds { .. }),
            "got {error:?} for {seconds}s"
        );
    }
}

#[test]
fn a_public_name_is_refused_rather_than_sent_to_the_ca() {
    let error = CertificateProfile::new(
        &scope(),
        Purpose::Serving,
        "lumen.example.com",
        CertificateIdentity {
            dns_names: vec!["lumen.example.com".into()],
            spiffe_uri: None,
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .expect_err("accepted a public name");
    assert!(
        matches!(error, ProfileError::PublicDnsName { .. }),
        "got {error:?}; the pool would refuse it (#3109), and failing locally names the value"
    );
}

#[test]
fn nothing_is_reissued_before_the_renewal_window_opens() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let fingerprint = harness.projected_fingerprint(Purpose::Peer).unwrap();
    let issued = harness.issuer.issued_count();

    // 12h leaf, renewed 2h early: due at +10h. One hour short of that.
    let outcome = step(
        &reconciler,
        &profile,
        &activated(Some(fingerprint)),
        plus_hours(start(), 9),
    );
    assert!(matches!(outcome.action, Action::Wait { .. }));
    assert_eq!(harness.issuer.issued_count(), issued);
}

#[test]
fn renewal_fires_once_the_window_opens() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let first = harness.projected_fingerprint(Purpose::Peer).unwrap();

    let due = plus_hours(start(), 10);
    harness.issuer.set_now(due);
    let outcome = step(&reconciler, &profile, &activated(Some(first.clone())), due);
    assert!(
        matches!(
            outcome.action,
            Action::Issue {
                reason: IssueReason::Renewal,
                ..
            }
        ),
        "got {:?}",
        outcome.action
    );
    assert_ne!(
        harness.projected_fingerprint(Purpose::Peer).unwrap(),
        first,
        "renewal must replace the material, not re-project it"
    );
}

#[test]
fn an_expired_leaf_is_replaced_rather_than_left_in_place() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let first = harness.projected_fingerprint(Purpose::Peer).unwrap();

    let late = plus_hours(start(), 30);
    harness.issuer.set_now(late);
    let outcome = step(&reconciler, &profile, &activated(Some(first.clone())), late);
    assert!(
        matches!(
            outcome.action,
            Action::Issue {
                reason: IssueReason::Expired,
                ..
            }
        ),
        "got {:?}; 'expired' has to stay distinguishable from 'due' -- it means something was \
         already wrong",
        outcome.action
    );
}

#[test]
fn a_restarted_controller_reaches_the_same_decision() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();

    {
        let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
        drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    }
    let fingerprint = harness.projected_fingerprint(Purpose::Peer).unwrap();
    let issued = harness.issuer.issued_count();

    // Ten fresh controllers, none of which remembers the others.
    for _ in 0..10 {
        let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
        let outcome = step(
            &reconciler,
            &profile,
            &activated(Some(fingerprint.clone())),
            plus_hours(start(), 1),
        );
        assert!(matches!(outcome.action, Action::Wait { .. }));
    }
    assert_eq!(
        harness.issuer.issued_count(),
        issued,
        "a controller that remembers 'I issued one recently' mints a duplicate on every \
         restart, and the failure is invisible"
    );
}

#[test]
fn the_renewal_deadline_is_the_same_number_on_every_restart() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let fingerprint = harness.projected_fingerprint(Purpose::Peer).unwrap();

    let mut deadlines = Vec::new();
    for hour in 1..6 {
        let outcome = step(
            &reconciler,
            &profile,
            &activated(Some(fingerprint.clone())),
            plus_hours(start(), hour),
        );
        let Action::Wait { until } = outcome.action else {
            panic!("expected Wait");
        };
        deadlines.push(until);
    }
    assert!(
        deadlines.windows(2).all(|pair| pair[0] == pair[1]),
        "the deadline moved between reconciles: {deadlines:?}"
    );
    assert_eq!(deadlines[0], plus_hours(start(), 10));
}

#[test]
fn a_renewal_deadline_sits_inside_the_leafs_validity() {
    // The property that makes the window meaningful: there is always time to
    // fail and retry before the leaf stops working.
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);
    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);

    let secret = harness
        .store
        .get("lumen", &scope.secret_name(Purpose::Peer))
        .unwrap();
    let pem = String::from_utf8(secret.data["tls.crt"].clone()).unwrap();
    let facts = parse_leaf(&pem).unwrap();
    let leaf = service_k8s::certificate::state::ObservedLeaf {
        issuer: service_k8s::certificate::issuer::IssuerId::new("pool-a"),
        not_before: facts.not_before,
        not_after: facts.not_after,
        fingerprint: facts.fingerprint,
        identity_digest: profile.identity_digest(),
    };
    let deadline = renew_at(&profile, &leaf);
    assert!(deadline > facts.not_before && deadline < facts.not_after);
    assert!(
        facts.not_after - deadline >= chrono::Duration::minutes(10),
        "the renewal window has to leave room for several failed attempts"
    );
}

#[test]
fn an_identity_change_reissues_even_though_the_leaf_is_still_valid() {
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
    let first = harness.projected_fingerprint(Purpose::Peer).unwrap();

    // Same instance, one more name.
    let widened = CertificateProfile::new(
        &scope,
        Purpose::Peer,
        "lumen-0.lumen-headless.lumen.svc.cluster.local",
        CertificateIdentity {
            dns_names: vec![
                "lumen-0.lumen-headless.lumen.svc.cluster.local".into(),
                "lumen-0.lumen-headless.lumen.svc".into(),
            ],
            spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/lumen/sa/lumen".into()),
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .unwrap();

    let outcome = step(
        &reconciler,
        &widened,
        &activated(Some(first.clone())),
        plus_hours(start(), 1),
    );
    assert!(
        matches!(
            outcome.action,
            Action::Issue {
                reason: IssueReason::IdentityChanged,
                ..
            }
        ),
        "got {:?}",
        outcome.action
    );
    assert_ne!(harness.projected_fingerprint(Purpose::Peer).unwrap(), first);
}

#[test]
fn a_peer_profile_without_a_spiffe_uri_does_not_exist() {
    let error = CertificateProfile::new(
        &scope(),
        Purpose::Peer,
        "lumen-0.lumen-headless.lumen.svc.cluster.local",
        CertificateIdentity {
            dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
            spiffe_uri: None,
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .expect_err("accepted a peer profile with no SPIFFE identity");
    assert_eq!(error, ProfileError::PeerNeedsSpiffeUri);
}

#[test]
fn a_spiffe_uri_from_another_namespace_is_refused() {
    let error = CertificateProfile::new(
        &InstanceScope::new("lumen", "lumen", "lumen-prod.svc.id.goog"),
        Purpose::Peer,
        "lumen-0.lumen-headless.lumen.svc.cluster.local",
        CertificateIdentity {
            dns_names: vec!["lumen-0.lumen-headless.lumen.svc.cluster.local".into()],
            spiffe_uri: Some("spiffe://lumen-prod.svc.id.goog/ns/other/sa/lumen".into()),
        },
        LIFETIME,
        RENEW_BEFORE,
        Duration::ZERO,
    )
    .expect_err("accepted another namespace's identity");
    assert!(matches!(error, ProfileError::ForeignSpiffeUri { .. }));
}

#[test]
fn a_settled_unchanged_reconcile_does_not_increment_apply_count() {
    let harness = Harness::new("pool-a", start());
    let profile = peer_profile();
    let owner = owner();
    let scope = scope();
    let reconciler = Reconciler::new(&scope, &owner, &harness.store, &harness.issuer);

    drive(&reconciler, &profile, &RuntimeReport::default(), start(), 6);
    let applies_after_bootstrap = harness.store.apply_count();

    let fingerprint = harness.projected_fingerprint(Purpose::Peer).unwrap();

    // Drive another step at a point where lifecycle is settled.
    step(
        &reconciler,
        &profile,
        &activated(Some(fingerprint)),
        plus_hours(start(), 1),
    );

    assert_eq!(
        harness.store.apply_count(),
        applies_after_bootstrap,
        "a settled unchanged reconcile must not increment apply_count"
    );
}
