// HANDWRITE-BEGIN gap="missing-generator:e2e-test:43194fd2" tracker="pending-tracker" reason="Profile-string assertions + a sandbox-exec localhost-only smoke test that skips when unavailable."
//! Network sandbox v3: seatbelt egress policy.
//!
//! Exercises the policy through the public sandbox API (`sandbox::pick` →
//! `Sandbox::resolve`), then runs the generated profile under `sandbox-exec`
//! to prove it (a) is syntactically valid on this macOS, (b) allows localhost,
//! and (c) denies external egress. The sandbox-exec parts skip cleanly when the
//! seatbelt backend isn't available (non-macOS / no sandbox-exec).
//!
//! @command cargo test -p vat --test vat_sandbox_egress -- --nocapture

use std::io::Read;
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

use vat::sandbox;
use vat::spec::{EgressPolicy, EnvSpec, Isolation};

/// Resolve the seatbelt profile string for an egress policy, or `None` when the
/// seatbelt backend isn't active on this host (so callers skip cleanly).
fn seatbelt_profile(egress: EgressPolicy) -> Option<String> {
    let spec = EnvSpec {
        isolation: Isolation::Seatbelt,
        egress,
        ..EnvSpec::default()
    };
    let backend = sandbox::pick(&spec);
    if backend.name() != "seatbelt" {
        return None; // process fallback (not macOS / sandbox-exec absent)
    }
    // resolve() wraps as ["sandbox-exec", "-p", <profile>, <program>, ...].
    let (_prog, argv) = backend.resolve(Path::new("/tmp/vat-egress-test"), "true", &[]);
    assert_eq!(argv.first().map(String::as_str), Some("-p"));
    argv.get(1).cloned()
}

/// Run `sandbox-exec -p <profile> -- <argv>` and return the exit status success.
fn run_sandboxed(profile: &str, argv: &[&str]) -> bool {
    let mut cmd = Command::new("sandbox-exec");
    cmd.arg("-p").arg(profile).arg("--");
    for a in argv {
        cmd.arg(a);
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

#[test]
fn localhost_only_profile_has_deny_and_localhost_allow() {
    let Some(profile) = seatbelt_profile(EgressPolicy::LocalhostOnly) else {
        eprintln!("skip: seatbelt backend unavailable on this host");
        return;
    };
    assert!(profile.contains("(deny network*)"), "profile:\n{profile}");
    assert!(
        profile.contains("(allow network* (remote ip \"localhost:*\"))"),
        "profile:\n{profile}"
    );
}

#[test]
fn localhost_only_profile_is_accepted_by_sandbox_exec() {
    let Some(profile) = seatbelt_profile(EgressPolicy::LocalhostOnly) else {
        eprintln!("skip: seatbelt backend unavailable on this host");
        return;
    };
    // sandbox-exec rejects a syntactically-invalid profile with a non-zero exit,
    // so a successful `true` proves the generated localhost-only profile compiles
    // on this macOS.
    assert!(
        run_sandboxed(&profile, &["/usr/bin/true"]),
        "sandbox-exec rejected the localhost-only profile:\n{profile}"
    );
}

#[test]
fn localhost_only_allows_loopback_denies_external() {
    let Some(profile) = seatbelt_profile(EgressPolicy::LocalhostOnly) else {
        eprintln!("skip: seatbelt backend unavailable on this host");
        return;
    };

    // A live loopback listener: a sandboxed connect to it must succeed.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().unwrap().port();
    let accept = std::thread::spawn(move || {
        if let Ok((mut s, _)) = listener.accept() {
            let mut buf = [0u8; 1];
            let _ = s.read(&mut buf);
        }
    });

    // bash /dev/tcp is the simplest in-sandbox connect probe on macOS. Under the
    // seatbelt deny, an external connect() is refused immediately (EPERM), not a
    // network timeout — so this is deterministic and fast.
    let loopback_ok = run_sandboxed(
        &profile,
        &[
            "/bin/bash",
            "-c",
            &format!("exec 3<>/dev/tcp/127.0.0.1/{port}"),
        ],
    );
    let external_denied = !run_sandboxed(
        &profile,
        &["/bin/bash", "-c", "exec 3<>/dev/tcp/1.1.1.1/80"],
    );

    let _ = accept.join();
    assert!(
        loopback_ok,
        "localhost connect was denied under localhost-only"
    );
    assert!(
        external_denied,
        "external connect was allowed under localhost-only (egress not confined)"
    );
}
// HANDWRITE-END
