use sift::{DurableJournal, EventEnvelope, EventQuery, SignalKind};

fn log(index: usize) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        format!("bounded-{index:04}"),
        SignalKind::Log,
        serde_json::json!({"message": format!("message-{index}")}),
    );
    event
        .resource
        .insert("service.name".into(), "bounded-journal".into());
    event
}

#[test]
fn resident_state_is_bounded_while_disk_query_and_old_dedupe_remain_complete() {
    let temp = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open_with_resident_limit(temp.path(), 16).unwrap();
    for index in 0..100 {
        journal.append(log(index)).unwrap();
    }

    assert!(journal.resident_event_count() <= 16);
    let page = journal
        .query(EventQuery {
            after: 0,
            limit: 100,
            ..EventQuery::default()
        })
        .unwrap();
    assert_eq!(page.len(), 100);
    assert_eq!(page.first().unwrap().event.event_id, "bounded-0000");
    assert_eq!(page.last().unwrap().event.event_id, "bounded-0099");
    drop(journal);

    let reopened = DurableJournal::open_with_resident_limit(temp.path(), 16).unwrap();
    assert!(reopened.resident_event_count() <= 16);
    let duplicate = reopened.append(log(0)).unwrap();
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.cursor, 1);
    assert_eq!(
        reopened
            .query(EventQuery {
                after: 0,
                limit: 100,
                ..EventQuery::default()
            })
            .unwrap()
            .len(),
        100
    );
}
