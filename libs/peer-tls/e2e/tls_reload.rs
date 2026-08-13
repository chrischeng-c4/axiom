//! What the reloader does when the material is wrong, and what it will admit to
//! (#3112 R6, R7, AC3, AC5).
//!
//! Two properties are load-bearing here and neither is about the happy path.
//!
//! The first is that a bad candidate changes nothing. A projected Secret is
//! briefly inconsistent by construction during an update, so a reloader that
//! tore down the active configuration on every unreadable read would turn every
//! renewal into an outage.
//!
//! The second is that "nothing changed" has an end. Retaining the last known
//! good material past its own expiry is not resilience — the leaf is no longer
//! an identity, and a peer that accepted it would be accepting anyone. So the
//! retention has exactly one bound, and it is the certificate's own `notAfter`.

mod support;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use peer_tls::material::MaterialPem;
use peer_tls::reload::{
    FileMaterialSource, MaterialSource, MemoryMaterialSource, ReloadableTls, TlsRuntimeProfile,
};
use support::{authority, seconds_from_now, Authority, LeafBuilder};

const MEMBER: &str = "lumen-0.lumen-peer.axiom.svc.cluster.local";
const SPIFFE: &str = "spiffe://axiom/ns/axiom/instance/lumen-0";

fn profile() -> TlsRuntimeProfile {
    TlsRuntimeProfile::peer([MEMBER.to_string()], [SPIFFE.to_string()])
}

fn good(ca: &Authority) -> MaterialPem {
    let leaf = LeafBuilder::new()
        .dns(&[MEMBER])
        .spiffe(&[SPIFFE])
        .issue(ca);
    MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem())
}

fn expiring(ca: &Authority, not_after_in: i64) -> MaterialPem {
    let leaf = LeafBuilder::new()
        .dns(&[MEMBER])
        .spiffe(&[SPIFFE])
        .window(seconds_from_now(-60), seconds_from_now(not_after_in))
        .issue(ca);
    MaterialPem::new(leaf.cert_pem, leaf.key_pem, ca.pem())
}

fn started(ca: &Authority) -> (ReloadableTls, Arc<MemoryMaterialSource>) {
    let source = Arc::new(MemoryMaterialSource::new(good(ca)));
    let tls = ReloadableTls::required(profile(), source.clone() as Arc<dyn MaterialSource>)
        .expect("well-formed startup material");
    (tls, source)
}

#[test]
fn tls_reload_keeps_the_last_valid_generation_when_a_candidate_is_rejected() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = started(&ca);
    let first = tls.fingerprint().expect("a leaf is active");

    // The half-written file an in-flight Secret update produces.
    source.set(MaterialPem::new("-----BEGIN CERT", "", ca.pem()));
    tls.reload().expect_err("garbage must not activate");

    assert_eq!(tls.generation(), 1, "a refusal must not advance the generation");
    assert_eq!(tls.fingerprint().as_deref(), Some(first.as_str()));
    assert!(
        tls.server_config().is_some(),
        "the previous generation must keep serving through a refusal"
    );
}

#[test]
fn tls_reload_counts_accepted_and_rejected_reloads_separately() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = started(&ca);

    source.clear();
    let _ = tls.reload();
    source.set(MaterialPem::new("not pem", "not pem", "not pem"));
    let _ = tls.reload();
    source.set(good(&ca));
    tls.reload().expect("a good candidate after two bad ones");

    let status = tls.status();
    assert_eq!(status.accepted_reloads, 2, "startup plus the recovery");
    assert_eq!(status.rejected_reloads, 2);
    assert_eq!(status.generation, 2);
    // A refusal is not sticky: once something valid activates, the last error
    // clears, or every dashboard would show a failure that stopped happening.
    assert_eq!(status.last_error_reason, None);
    assert_eq!(status.last_error, None);
}

#[test]
fn tls_reload_records_the_reason_of_the_most_recent_refusal() {
    let ca = authority("axiom-peer-ca");
    let stranger = authority("someone-elses-ca");
    let (tls, source) = started(&ca);

    // A leaf from somewhere else against the bundle we actually have — the
    // shape a mis-wired issuer produces, as opposed to a whole trust-domain
    // migration, where the bundle moves with the leaf.
    let foreign = LeafBuilder::new()
        .dns(&[MEMBER])
        .spiffe(&[SPIFFE])
        .issue(&stranger);
    source.set(MaterialPem::new(
        foreign.cert_pem,
        foreign.key_pem,
        ca.pem(),
    ));
    tls.reload().unwrap_err();
    assert_eq!(tls.status().last_error_reason, Some("untrusted"));

    source.clear();
    tls.reload().unwrap_err();
    assert_eq!(
        tls.status().last_error_reason,
        Some("unreadable"),
        "the reported reason must be the latest one, not the first"
    );
}

#[test]
fn tls_reload_status_carries_no_pem_body_and_no_filesystem_path() {
    let dir = tempfile::tempdir().unwrap();
    let ca = authority("axiom-peer-ca");
    let material = good(&ca);
    let cert = dir.path().join("tls.crt");
    let key = dir.path().join("tls.key");
    let bundle = dir.path().join("ca.crt");
    std::fs::write(&cert, &material.cert_chain).unwrap();
    std::fs::write(&key, &material.key).unwrap();
    std::fs::write(&bundle, &material.trust_bundle).unwrap();

    let source = Arc::new(FileMaterialSource::new(&cert, &key, &bundle));
    let tls = ReloadableTls::required(profile(), source as Arc<dyn MaterialSource>)
        .expect("a projected Secret layout activates");

    // Break it in a way whose natural error text would name the file.
    std::fs::remove_file(&key).unwrap();
    tls.reload().unwrap_err();

    let status = tls.status();
    let rendered = format!("{status:?}");
    assert!(!rendered.contains("BEGIN"), "status leaked PEM: {rendered}");
    assert!(
        !rendered.contains("tls.key"),
        "status leaked a private-key path: {rendered}"
    );
    assert!(
        !rendered.contains(dir.path().to_str().unwrap()),
        "status leaked a filesystem path: {rendered}"
    );
    assert_eq!(status.last_error_reason, Some("unreadable"));
    assert!(status.serving, "the mount going away must not stop serving");
}

#[test]
fn tls_reload_required_startup_fails_when_no_material_exists() {
    let source = Arc::new(MemoryMaterialSource::empty());
    // Nothing to prove an identity with, and no listener worth publishing: the
    // alternative is a process that reports ready and fails every handshake.
    let rejection = ReloadableTls::required(profile(), source as Arc<dyn MaterialSource>)
        .expect_err("required TLS with no material must not start");
    assert_eq!(rejection.reason.as_str(), "unreadable");
}

#[test]
fn tls_reload_required_startup_fails_when_the_material_is_already_expired() {
    let ca = authority("axiom-peer-ca");
    let leaf = LeafBuilder::new()
        .dns(&[MEMBER])
        .spiffe(&[SPIFFE])
        .window(seconds_from_now(-7200), seconds_from_now(-60))
        .issue(&ca);
    let source = Arc::new(MemoryMaterialSource::new(MaterialPem::new(
        leaf.cert_pem,
        leaf.key_pem,
        ca.pem(),
    )));

    let rejection = ReloadableTls::required(profile(), source as Arc<dyn MaterialSource>)
        .expect_err("an expired leaf is not a startup identity");
    assert_eq!(rejection.reason.as_str(), "expired");
}

#[test]
fn tls_reload_refuses_to_serve_once_the_last_known_good_expires() {
    let ca = authority("axiom-peer-ca");
    let short = expiring(&ca, 300);
    let source = Arc::new(MemoryMaterialSource::new(short));
    let tls = ReloadableTls::required(profile(), source.clone() as Arc<dyn MaterialSource>)
        .expect("a short-lived leaf still starts");

    // The renewal never lands: the controller is wedged, or the CA is
    // unreachable, and every reload attempt fails.
    source.clear();
    tls.reload().unwrap_err();

    let inside = SystemTime::now() + Duration::from_secs(60);
    assert!(
        tls.server_config_at(inside).is_some(),
        "a wedged controller must not cost us the identity we still hold"
    );

    let past_expiry = SystemTime::now() + Duration::from_secs(600);
    assert!(
        tls.server_config_at(past_expiry).is_none(),
        "an expired leaf must stop serving rather than fail open"
    );
    assert!(tls.client_config_at(past_expiry).is_none());
    let status = tls.status_at(past_expiry);
    assert!(!status.serving);
    assert_eq!(status.seconds_to_expiry, Some(0));
    // The generation and fingerprint stay, because "we are holding a leaf that
    // has expired" is a different situation from "we never had one" and the
    // remediation differs.
    assert_eq!(status.generation, 1);
    assert!(status.fingerprint.is_some());
}

#[test]
fn tls_reload_reports_seconds_to_expiry_from_the_active_leaf() {
    let ca = authority("axiom-peer-ca");
    let source = Arc::new(MemoryMaterialSource::new(expiring(&ca, 3600)));
    let tls = ReloadableTls::required(profile(), source as Arc<dyn MaterialSource>).unwrap();

    let left = tls.status().seconds_to_expiry.expect("an active leaf");
    // Bounded either side: an alert that fires on this number is worthless if
    // the number can be a wildly wrong constant.
    assert!(
        (3500..=3600).contains(&left),
        "seconds to expiry should track the certificate, got {left}"
    );
}

#[test]
fn tls_reload_does_not_advance_the_generation_when_nothing_changed() {
    let ca = authority("axiom-peer-ca");
    let (tls, _source) = started(&ca);

    // The poll loop runs every 30 seconds forever. If each tick counted as an
    // activation, the generation would stop meaning "the leaf moved" — which is
    // exactly what the certificate controller compares against.
    for _ in 0..5 {
        tls.reload().expect("re-reading the same material is fine");
    }

    let status = tls.status();
    assert_eq!(status.generation, 1);
    assert_eq!(status.accepted_reloads, 1);
    assert_eq!(status.rejected_reloads, 0);
}

#[test]
fn tls_reload_reads_a_projected_secret_layout_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let ca = authority("axiom-peer-ca");
    let first = good(&ca);
    std::fs::write(dir.path().join("tls.crt"), &first.cert_chain).unwrap();
    std::fs::write(dir.path().join("tls.key"), &first.key).unwrap();
    std::fs::write(dir.path().join("ca.crt"), &first.trust_bundle).unwrap();

    let source = Arc::new(FileMaterialSource::new(
        dir.path().join("tls.crt"),
        dir.path().join("tls.key"),
        dir.path().join("ca.crt"),
    ));
    let tls = ReloadableTls::required(profile(), source as Arc<dyn MaterialSource>).unwrap();
    let before = tls.fingerprint().unwrap();

    let renewed = good(&ca);
    std::fs::write(dir.path().join("tls.crt"), &renewed.cert_chain).unwrap();
    std::fs::write(dir.path().join("tls.key"), &renewed.key).unwrap();
    tls.reload().expect("a renewed projection activates");

    assert_eq!(tls.generation(), 2);
    assert_ne!(tls.fingerprint().unwrap(), before);
}

#[test]
fn tls_reload_accepts_a_leaf_and_bundle_that_move_to_a_new_authority_together() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = started(&ca);

    // Migrating to a different CA entirely is a legitimate operation, and the
    // projection is the trust root — the process has no independent opinion
    // about which authority is correct, only about which identity it must
    // present. Pinning the startup CA here would make CA migration impossible
    // without a rolling restart, which is the outage this whole issue removes.
    let successor = authority("axiom-peer-ca-successor");
    source.set(good(&successor));
    tls.reload().expect("a coherent trust-domain migration activates");
    assert_eq!(tls.generation(), 2);
}

#[test]
fn tls_reload_never_activates_material_for_another_workload() {
    let ca = authority("axiom-peer-ca");
    let (tls, source) = started(&ca);
    let ours = tls.fingerprint().unwrap();

    // The projection that goes to the wrong pod: correct CA, in date, wrong
    // member. Nothing structural is wrong with it, which is why the identity
    // check has to be part of activation rather than a later audit.
    let theirs = LeafBuilder::new()
        .dns(&["lumen-3.lumen-peer.axiom.svc.cluster.local"])
        .spiffe(&["spiffe://axiom/ns/axiom/instance/lumen-3"])
        .issue(&ca);
    source.set(MaterialPem::new(theirs.cert_pem, theirs.key_pem, ca.pem()));

    let rejection = tls.reload().expect_err("another member's leaf must not activate");
    assert_eq!(rejection.reason.as_str(), "wrong_identity");
    assert_eq!(tls.fingerprint().unwrap(), ours);
}
