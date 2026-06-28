// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_runner_sandbox-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Integration test for #527: the sandbox (seatbelt isolation + the
//! `[network].egress` policy) applies to RUNNER-mode commands, not just direct
//! mode.
//!
//! A real `vat run` with `--isolation seatbelt` + `[network].egress =
//! localhost-only`: a runner that connects to a localhost listener succeeds
//! (so local emulators stay reachable) while a runner that connects to an
//! external host is denied — proving the egress policy reached the runner via
//! the runner-mode sandbox wiring. Skips cleanly off-macOS / when sandbox-exec
//! or bash is unavailable. (The deterministic wrap-logic proof is the
//! `sandbox_wrap` unit test in `commands::run`; the profile's egress behaviour
//! is proven by `vat_sandbox_egress`.)
//!
//! @command cargo test -p vat --test vat_runner_sandbox -- --nocapture

use std::io::Read;
use std::net::TcpListener;
use std::process::Command;
use std::thread;

use serde_json::Value;
use vat::sandbox;
use vat::spec::{EnvSpec, Isolation};

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

/// The seatbelt backend is actually active here (macOS + sandbox-exec present)?
fn seatbelt_active() -> bool {
    let spec = EnvSpec {
        isolation: Isolation::Seatbelt,
        ..EnvSpec::default()
    };
    sandbox::pick(&spec).name() == "seatbelt"
}

fn bash_available() -> bool {
    Command::new("/bin/bash")
        .args(["-c", "exit 0"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn runner_mode_seatbelt_egress_allows_localhost_denies_external() {
    if !seatbelt_active() || !bash_available() {
        return; // not macOS / no sandbox-exec / no bash → skip cleanly
    }

    // A localhost listener the sandboxed runner is allowed to reach.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut s) = stream {
                let mut buf = [0u8; 1];
                let _ = s.read(&mut buf);
            }
        }
    });

    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
name = "runner-egress-smoke"

[workspace]
base = "."
workdir = "."
keep = "never"

[network]
egress = "localhost-only"

[[runners]]
id = "ok"
cmd = ["/bin/bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/{port}"]

[[runners]]
id = "blocked"
cmd = ["/bin/bash", "-c", "exec 3<>/dev/tcp/1.1.1.1/80"]
"#
        ),
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .args(["run", "ok", "blocked", "--isolation", "seatbelt"])
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .output()
        .unwrap();

    let events: Vec<Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let result = events
        .iter()
        .find(|e| e["type"] == "result")
        .unwrap_or_else(|| {
            panic!(
                "missing result event; stdout:\n{}",
                String::from_utf8_lossy(&output.stdout)
            )
        });
    let runners = result["runners"].as_array().expect("runners array");
    let find = |id: &str| {
        runners
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("runner `{id}` missing in {result}"))
    };

    // localhost is reachable under seatbelt localhost-only → emulators/proxy work.
    assert_eq!(
        find("ok")["exit_code"],
        0,
        "localhost runner should succeed under seatbelt: {result}"
    );
    // external egress is denied → the runner fails, proving the egress policy
    // reached the runner via the runner-mode sandbox wiring.
    assert_ne!(
        find("blocked")["exit_code"],
        0,
        "external runner should be denied under seatbelt localhost-only: {result}"
    );
}
// CODEGEN-END
