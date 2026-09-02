use std::{collections::BTreeMap, time::Duration};

use sift::{storage::archive, DurableJournal, EventEnvelope, ServiceState, SignalKind};

const EVENT_COUNT: usize = 600_000;
const BATCH_EVENTS: usize = 1_000;

fn event(index: usize) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "large-archive",
        "test",
        format!("large-{index:06}"),
        SignalKind::Log,
        serde_json::json!({"message": "bounded archive lifecycle"}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "large-archive".into())]);
    event
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn six_hundred_thousand_events_archive_and_restore_with_bounded_resident_state() {
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

    tokio::task::spawn_blocking(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let source = tempfile::tempdir().unwrap();
        let state = ServiceState::open(source.path()).unwrap();
        let projection_worker = runtime.block_on(async { state.start_projection_worker() });
        for first in (0..EVENT_COUNT).step_by(BATCH_EVENTS) {
            let last = (first + BATCH_EVENTS).min(EVENT_COUNT);
            runtime
                .block_on(state.append_events((first..last).map(event).collect()))
                .unwrap();
            state.journal().storage().seal_ready().unwrap();
            assert!(state.journal().resident_event_count() <= 100_000);
            if last % 100_000 == 0 {
                eprintln!("large archive lifecycle accepted {last} events");
            }
        }
        assert_eq!(state.journal().total_event_count(), EVENT_COUNT as u64);
        runtime.block_on(state.finish_drain()).unwrap();
        runtime.block_on(projection_worker.stop());

        let receipt =
            archive::archive_journal_gcs(state.journal(), "gs://sift-large-archive/lifecycle")
                .unwrap();
        assert_eq!(receipt.manifest.event_count, EVENT_COUNT as u64);
        assert!(receipt.manifest.segment_count >= 6);
        assert!(source
            .path()
            .join("tmp")
            .read_dir()
            .unwrap()
            .next()
            .is_none());
        drop(state);
        let reopened = DurableJournal::open_with_resident_limit(source.path(), 100_000).unwrap();
        assert_eq!(reopened.total_event_count(), EVENT_COUNT as u64);
        assert!(reopened.resident_event_count() <= 100_000);
        drop(reopened);

        let restored = tempfile::tempdir().unwrap();
        archive::restore_gcs(&receipt.manifest_uri, restored.path()).unwrap();
        let restored = DurableJournal::open_with_resident_limit(restored.path(), 100_000).unwrap();
        assert_eq!(restored.total_event_count(), EVENT_COUNT as u64);
        assert!(restored.resident_event_count() <= 100_000);
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
