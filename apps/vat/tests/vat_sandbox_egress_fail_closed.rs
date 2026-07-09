// HANDWRITE-BEGIN gap="missing-generator:unit-test:a5020e3c" tracker="pending-tracker" reason="Unit-test section edge: regression coverage for UT1-UT4 — process.rs rejects non-Open egress under isolation=none, Open stays unaffected, the seatbelt-unavailable fallback rejects non-Open egress, and the seatbelt-unavailable + Open case still falls back successfully."
//! Regression tests for issue #1300: `sandbox::pick` must fail closed instead
//! of silently downgrading to unrestricted network access when the selected
//! backend cannot enforce a non-`Open` [network].egress policy.

use vat::sandbox;
use vat::spec::{EgressPolicy, EnvSpec, Isolation};

/// UT1: `Isolation::None` with a non-`Open` egress policy must return `Err`
/// (not a printed warning) and must not hand back a usable backend.
#[test]
fn process_backend_rejects_non_open_egress() {
    for egress in [EgressPolicy::LocalhostOnly, EgressPolicy::Deny] {
        let spec = EnvSpec {
            isolation: Isolation::None,
            egress,
            ..EnvSpec::default()
        };
        let result = sandbox::pick(&spec);
        let err = match result {
            Err(err) => err,
            Ok(_) => panic!("isolation=none + egress={egress:?} must fail closed, not run open"),
        };
        assert!(
            err.contains("egress"),
            "error should name the egress policy: {err}"
        );
        assert!(
            err.to_lowercase().contains("none") || err.to_lowercase().contains("enforce"),
            "error should explain isolation=none cannot enforce it: {err}"
        );
    }
}

/// UT2: `Isolation::None` with `EgressPolicy::Open` is the common case and
/// must keep succeeding exactly as before — no regression.
#[test]
fn process_backend_open_unaffected() {
    let spec = EnvSpec {
        isolation: Isolation::None,
        egress: EgressPolicy::Open,
        ..EnvSpec::default()
    };
    let backend = sandbox::pick(&spec).expect("isolation=none + egress=open must succeed");
    assert_eq!(backend.name(), "process");
}

/// UT3: `Isolation::Seatbelt` with a non-`Open` egress policy on a host
/// without `sandbox-exec` must return `Err` instead of silently falling back
/// to `ProcessBackend`.
#[test]
fn seatbelt_unavailable_rejects_non_open_egress() {
    if cfg!(target_os = "macos") && sandbox::seatbelt::available() {
        eprintln!(
            "skipping: sandbox-exec is available on this host, so the \
             seatbelt-unavailable fallback path cannot be exercised here"
        );
        return;
    }
    let spec = EnvSpec {
        isolation: Isolation::Seatbelt,
        egress: EgressPolicy::LocalhostOnly,
        ..EnvSpec::default()
    };
    let err = match sandbox::pick(&spec) {
        Err(err) => err,
        Ok(_) => panic!(
            "seatbelt requested + unavailable + non-open egress must fail closed, not fall back"
        ),
    };
    assert!(
        err.contains("sandbox-exec"),
        "error should name the missing seatbelt backend: {err}"
    );
    assert!(
        err.to_lowercase().contains("egress") || err.to_lowercase().contains("localhostonly"),
        "error should name the requested egress policy: {err}"
    );
}

/// UT4: `Isolation::Seatbelt` with `EgressPolicy::Open` on a host without
/// `sandbox-exec` still falls back to `ProcessBackend` and succeeds — the
/// fallback is only rejected when it would silently drop enforcement.
#[test]
fn seatbelt_unavailable_open_falls_back() {
    if cfg!(target_os = "macos") && sandbox::seatbelt::available() {
        eprintln!(
            "skipping: sandbox-exec is available on this host, so the \
             seatbelt-unavailable fallback path cannot be exercised here"
        );
        return;
    }
    let spec = EnvSpec {
        isolation: Isolation::Seatbelt,
        egress: EgressPolicy::Open,
        ..EnvSpec::default()
    };
    let backend = sandbox::pick(&spec)
        .expect("seatbelt unavailable + egress=open must still fall back to process");
    assert_eq!(backend.name(), "process");
}
// HANDWRITE-END
