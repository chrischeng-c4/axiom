use sift::{DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind};

fn event(index: usize) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        format!("paged-{index:05}"),
        SignalKind::Log,
        serde_json::json!({"message": format!("message-{index}")}),
    );
    event
        .resource
        .insert("service.name".into(), "paged-recovery".into());
    event
}

#[tokio::test]
async fn restart_pages_canonical_data_and_rebuilds_a_missing_disk_index() {
    const EVENTS: usize = 20_500;
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();
    for first in (0..EVENTS).step_by(1_000) {
        let last = (first + 1_000).min(EVENTS);
        state
            .append_events((first..last).map(event).collect())
            .await
            .unwrap();
    }
    drop(state);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let dedupe_root = temp.path().join("indexes/dedupe");
    let shard = std::fs::read_dir(&dedupe_root)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("idx"))
        .expect("at least one dedupe shard");
    std::fs::remove_file(shard).unwrap();

    let journal = DurableJournal::open_with_resident_limit(temp.path(), 16).unwrap();
    assert!(journal.resident_event_count() <= 16);
    assert_eq!(journal.total_event_count(), EVENTS as u64);
    assert!(journal.append(event(0)).unwrap().duplicate);

    let mut count = 0;
    let mut after = 0;
    loop {
        let page = journal
            .query(EventQuery {
                after,
                limit: 3_000,
                ..EventQuery::default()
            })
            .unwrap();
        let Some(last) = page.last() else {
            break;
        };
        after = last.cursor;
        count += page.len();
    }
    assert_eq!(count, EVENTS);
    assert_eq!(after, EVENTS as u64);
}
