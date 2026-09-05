use std::collections::BTreeMap;
use std::time::Duration;

use service_http::MetricsProvider;
use sift::{DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind};

fn log() -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "archive-worker",
        "test",
        "worker-log-1",
        SignalKind::Log,
        serde_json::json!({"message":"archive me"}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "worker-test".into())]);
    event
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn leader_lifecycle_worker_commits_gcs_before_compacting_wal() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let address_text = address.to_string();
    let emulator = tokio::spawn(async move {
        vat::emulator::serve(vat::emulator::Kind::CloudStorage, &address_text)
            .await
            .unwrap();
    });
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(address).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    std::env::set_var("STORAGE_EMULATOR_HOST", format!("http://{address}"));

    let data = tempfile::tempdir().unwrap();
    let state = ServiceState::open(data.path()).unwrap();
    state.append_events(vec![log()]).await.unwrap();
    let worker =
        state.start_archive_worker("gs://sift-worker/archive-worker", Duration::from_millis(10));
    let commit = data.path().join("control/archive-commit.json");
    for _ in 0..500 {
        if commit.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    worker.stop().await;

    assert!(
        commit.exists(),
        "worker never committed an archive manifest"
    );
    let wal = data.path().join("wal/logs/events.framed");
    assert!(
        storage_durable::FramedLogReader::read_frames(wal, 0)
            .unwrap()
            .is_empty(),
        "WAL must compact only after the archive commit exists"
    );

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}

#[tokio::test]
async fn local_mode_commits_a_segment_manifest_before_compacting_wal() {
    let data = tempfile::tempdir().unwrap();
    let state = ServiceState::open(data.path()).unwrap();
    state.append_events(vec![log()]).await.unwrap();
    let worker = state.start_local_archive_worker(Duration::from_millis(10));
    let commit = data.path().join("control/local-segment-commit.json");
    for _ in 0..500 {
        if commit.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    worker.stop().await;

    assert!(commit.exists(), "worker never committed a local manifest");
    let wal = data.path().join("wal/logs/events.framed");
    assert!(
        storage_durable::FramedLogReader::read_frames(wal, 0)
            .unwrap()
            .is_empty(),
        "local WAL must compact only after the local commit exists"
    );
    state.finish_drain().await.unwrap();
    drop(state);

    let reopened = DurableJournal::open(data.path()).unwrap();
    let events = reopened.query(EventQuery::default()).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event.event_id, "worker-log-1");
}

#[tokio::test]
async fn local_compaction_reconciles_the_shared_capacity_guard() {
    let data = tempfile::tempdir().unwrap();
    let state = ServiceState::open(data.path()).unwrap();
    state.append_events(vec![log()]).await.unwrap();

    let worker = state.start_local_archive_worker(Duration::from_millis(10));
    let commit = data.path().join("control/local-segment-commit.json");
    for _ in 0..500 {
        if commit.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    worker.stop().await;
    assert!(commit.exists(), "worker never committed a local manifest");

    let reported = metric_value(&state.render_metrics(), "sift_local_storage_used_bytes");
    assert_eq!(
        reported,
        directory_bytes(data.path()),
        "a successful compaction must replace estimates with measured bytes"
    );
}

#[tokio::test]
async fn committed_receipt_retries_a_late_wal_compaction_failure() {
    let data = tempfile::tempdir().unwrap();
    let state = ServiceState::open(data.path()).unwrap();
    state.append_events(vec![log()]).await.unwrap();
    let wal = data.path().join("wal/logs/events.framed");
    let archived_wal = std::fs::read(&wal).unwrap();
    assert!(!archived_wal.is_empty());

    sift::storage::archive::archive_journal_local(state.journal()).unwrap();
    assert!(storage_durable::FramedLogReader::read_frames(&wal, 0)
        .unwrap()
        .is_empty());

    // Simulate a process that committed the receipt but failed before the
    // authorized WAL truncate reached disk.
    std::fs::write(&wal, archived_wal).unwrap();
    assert_eq!(
        storage_durable::FramedLogReader::read_frames(&wal, 0)
            .unwrap()
            .len(),
        1
    );

    let worker = state.start_local_archive_worker(Duration::from_millis(10));
    for _ in 0..500 {
        if storage_durable::FramedLogReader::read_frames(&wal, 0)
            .unwrap()
            .is_empty()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    worker.stop().await;
    assert!(
        storage_durable::FramedLogReader::read_frames(&wal, 0)
            .unwrap()
            .is_empty(),
        "the next lifecycle tick must finish receipt-authorized WAL compaction"
    );
}

fn metric_value(metrics: &str, name: &str) -> u64 {
    metrics
        .lines()
        .find_map(|line| {
            let (metric, value) = line.split_once(' ')?;
            (metric == name).then(|| value.parse().unwrap())
        })
        .unwrap_or_else(|| panic!("missing metric {name}"))
}

fn directory_bytes(root: &std::path::Path) -> u64 {
    let mut total = 0;
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let metadata = std::fs::symlink_metadata(entry.path()).unwrap();
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                pending.push(entry.path());
            } else {
                total += metadata.len();
            }
        }
    }
    total
}
