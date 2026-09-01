//! One Sift binary exposes every internal deployment role.

use std::{
    fs,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn sift() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sift"))
}

#[test]
fn serve_help_lists_every_product_role() {
    let output = sift()
        .args(["serve", "--help"])
        .output()
        .expect("run serve help");
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).expect("help is utf-8");
    assert!(help.contains("--role <ROLE>"));
    for role in [
        "all", "agent", "gateway", "query", "store", "control", "operator",
    ] {
        assert!(help.contains(role), "serve help must list {role}");
    }
    assert!(help.contains("[default: all]"));
    assert!(help.contains("--ephemeral"));
}

#[test]
fn ephemeral_mode_is_refused_for_production_roles() {
    let output = sift()
        .args([
            "serve",
            "--role",
            "store",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--ephemeral",
        ])
        .output()
        .expect("run forbidden ephemeral store");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--ephemeral is forbidden for production Sift roles"));
}

#[test]
fn all_in_one_ephemeral_mode_requires_an_explicit_flag_and_starts() {
    let mut child = sift()
        .args([
            "serve",
            "--role",
            "all",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--ephemeral",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start explicit ephemeral all-in-one");
    thread::sleep(Duration::from_millis(250));
    assert!(child.try_wait().unwrap().is_none());
    child.kill().unwrap();
    let _ = child.wait();
}

#[test]
fn every_role_records_its_identity_before_serving() {
    for role in [
        "all", "agent", "gateway", "query", "store", "control", "operator",
    ] {
        let temp = tempfile::tempdir().expect("temporary role root");
        let mut child = sift()
            .args([
                "serve",
                "--role",
                role,
                "--host",
                "127.0.0.1",
                "--port",
                "0",
                "--data-dir",
            ])
            .arg(temp.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start role");

        let layout_path = temp.path().join("layout.json");
        let deadline = Instant::now() + Duration::from_secs(5);
        while !layout_path.exists() && Instant::now() < deadline {
            if let Some(status) = child.try_wait().expect("poll role") {
                panic!("role {role} exited before opening storage: {status}");
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert!(layout_path.exists(), "role {role} must initialize storage");
        child.kill().expect("stop role");
        let _ = child.wait();

        let layout: serde_json::Value =
            serde_json::from_slice(&fs::read(layout_path).unwrap()).unwrap();
        assert_eq!(layout["role"], role);
    }
}

#[test]
fn replicated_store_refuses_to_start_without_peer_mtls_material() {
    let temp = tempfile::tempdir().expect("temporary replicated root");
    let mut child = sift()
        .args([
            "serve",
            "--role",
            "store",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--data-dir",
        ])
        .arg(temp.path())
        .env("SHARD_COUNT", "1")
        .env("REPLICAS_PER_SHARD", "3")
        .env("VOTER_COUNT", "3")
        .env("POD_NAME", "sift-store-0")
        .env_remove("SIFT_PEER_TLS_CERT")
        .env_remove("SIFT_PEER_TLS_KEY")
        .env_remove("SIFT_PEER_TLS_CA")
        .env_remove("SIFT_PEER_MTLS")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run replicated store without peer credentials");
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().expect("poll replicated store").is_none() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }
    if child
        .try_wait()
        .expect("poll replicated store at deadline")
        .is_none()
    {
        child
            .kill()
            .expect("stop store that accepted insecure peer mode");
        let _ = child.wait();
        panic!("replicated store accepted missing peer mTLS material");
    }
    let output = child
        .wait_with_output()
        .expect("collect replicated store output");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("replicated Sift requires peer mTLS"),
        "unexpected startup error: {stderr}"
    );
}
