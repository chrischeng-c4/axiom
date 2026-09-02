//! Black-box contract for Sift's always-durable local data root.

use std::{fs, process::Command};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

fn sift() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sift"))
}

#[test]
fn every_local_storage_command_advertises_the_shared_durable_default() {
    for command in ["serve", "collect", "snapshot", "restore"] {
        let output = sift()
            .args([command, "--help"])
            .output()
            .expect("run sift help");
        assert!(output.status.success(), "{command} help must succeed");
        let stdout = String::from_utf8(output.stdout).expect("help is utf-8");
        assert!(
            stdout.contains("[default: /var/lib/sift]"),
            "{command} must advertise /var/lib/sift as its default data directory:\n{stdout}"
        );
        assert!(
            !stdout.contains("[default: sift-data]"),
            "{command} must not silently fall back to a working-directory path"
        );
    }
}

#[test]
fn opening_a_fresh_root_creates_the_versioned_private_layout() {
    let temp = tempfile::tempdir().expect("temporary data root parent");
    let data_dir = temp.path().join("data");
    let snapshot = temp.path().join("snapshot.json");
    let output = sift()
        .args(["snapshot", "--data-dir"])
        .arg(&data_dir)
        .arg("--out")
        .arg(&snapshot)
        .output()
        .expect("create snapshot through the real CLI");
    assert!(
        output.status.success(),
        "fresh durable root must open: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let layout_path = data_dir.join("layout.json");
    let layout: serde_json::Value = serde_json::from_slice(
        &fs::read(&layout_path).expect("versioned layout manifest must exist"),
    )
    .expect("layout manifest is json");
    assert_eq!(layout["format_version"], 1);
    assert!(layout["cluster_id"]
        .as_str()
        .is_some_and(|id| !id.is_empty()));
    assert!(layout["node_id"].as_str().is_some_and(|id| !id.is_empty()));
    assert_eq!(layout["role"], "all");

    for relative in [
        "control",
        "wal/logs",
        "wal/metrics",
        "wal/traces",
        "segments/logs",
        "segments/metrics",
        "segments/traces",
        "indexes",
        "snapshots",
        "archive-cache",
        "gateway-spool",
        "query-jobs",
        "agent",
        "tmp",
    ] {
        assert!(
            data_dir.join(relative).is_dir(),
            "durable layout must create {relative}"
        );
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&data_dir).unwrap().permissions().mode() & 0o777,
            0o700,
            "the data root must be private"
        );
        assert_eq!(
            fs::metadata(layout_path).unwrap().permissions().mode() & 0o777,
            0o600,
            "the layout manifest must be private"
        );
    }
}

#[test]
fn snapshot_without_out_emits_the_binary_snapshot_in_a_json_envelope() {
    let temp = tempfile::tempdir().expect("temporary data root");
    let output = sift()
        .args(["snapshot", "--data-dir"])
        .arg(temp.path())
        .output()
        .expect("create snapshot through the real CLI");
    assert!(
        output.status.success(),
        "snapshot must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("snapshot terminal envelope is JSON");
    assert_eq!(envelope["format"], "sift-snapshot-v2");
    assert_eq!(envelope["encoding"], "base64");
    assert_eq!(envelope["next"], "done");
    let snapshot = BASE64
        .decode(
            envelope["snapshot_base64"]
                .as_str()
                .expect("snapshot payload"),
        )
        .expect("snapshot payload is base64");
    assert!(snapshot.starts_with(b"SIFTSNP2"));
    assert_eq!(envelope["bytes"], snapshot.len());
}

#[test]
fn an_unversioned_legacy_root_is_refused_without_changing_its_bytes() {
    let temp = tempfile::tempdir().expect("temporary data root");
    let legacy = temp.path().join("raw-events.framed");
    let original = b"legacy-sift-0.1.1";
    fs::write(&legacy, original).expect("seed legacy marker");

    let output = sift()
        .args(["snapshot", "--data-dir"])
        .arg(temp.path())
        .arg("--out")
        .arg(temp.path().join("snapshot.json"))
        .output()
        .expect("run snapshot against legacy root");

    assert!(!output.status.success(), "legacy data must not be adopted");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("legacy Sift 0.1.1"),
        "refusal must name the incompatible format: {stderr}"
    );
    assert_eq!(fs::read(&legacy).unwrap(), original);
    assert!(!temp.path().join("layout.json").exists());
}

#[test]
fn restart_removes_only_known_orphan_archive_spill_directories() {
    let temp = tempfile::tempdir().expect("temporary data root");
    drop(sift::DurableJournal::open(temp.path()).unwrap());
    let orphan = temp.path().join("tmp/archive-updates-orphan");
    let unrelated = temp.path().join("tmp/user-kept");
    fs::create_dir(&orphan).unwrap();
    fs::write(orphan.join("page"), b"orphan").unwrap();
    fs::create_dir(&unrelated).unwrap();
    fs::write(unrelated.join("note"), b"keep").unwrap();

    drop(sift::DurableJournal::open(temp.path()).unwrap());
    assert!(!orphan.exists());
    assert!(unrelated.join("note").is_file());
}
