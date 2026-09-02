use std::sync::Arc;

use chrono::{Duration, SecondsFormat, TimeZone, Utc};
use sift::{durability, DurableJournal, EventEnvelope, EventQuery, SignalKind};

fn event_for_project(project: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        project,
        "test",
        "same-event-id",
        SignalKind::Log,
        serde_json::json!({"message": "deterministic duplicate decision"}),
    );
    event
        .resource
        .insert("service.name".into(), "raft-dedupe".into());
    event
}

fn event() -> EventEnvelope {
    event_for_project("raft-dedupe")
}

fn command(at: chrono::DateTime<Utc>) -> Vec<u8> {
    durability::encode_raft_batch_at_for_diagnostics(
        vec![event()],
        &at.to_rfc3339_opts(SecondsFormat::Nanos, true),
    )
    .unwrap()
}

#[test]
fn identical_event_ids_are_isolated_by_project() {
    let root = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(root.path()).unwrap());
    let state_machine = durability::SiftStateMachine::new(journal.clone());
    let decision_time = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

    let first = durability::encode_raft_batch_at_for_diagnostics(
        vec![event_for_project("project-a")],
        &decision_time.to_rfc3339_opts(SecondsFormat::Nanos, true),
    )
    .unwrap();
    state_machine.apply_local(1, &first).unwrap();
    let first_result = state_machine.take_append_outcomes(1).unwrap();
    assert_eq!(first_result[0].raw_cursor, 1);
    assert!(!first_result[0].duplicate);

    let second = durability::encode_raft_batch_at_for_diagnostics(
        vec![event_for_project("project-b")],
        &decision_time.to_rfc3339_opts(SecondsFormat::Nanos, true),
    )
    .unwrap();
    state_machine.apply_local(2, &second).unwrap();
    let second_result = state_machine.take_append_outcomes(2).unwrap();
    assert_eq!(second_result[0].raw_cursor, 2);
    assert!(!second_result[0].duplicate);

    drop(state_machine);
    drop(journal);
    let journal = Arc::new(DurableJournal::open(root.path()).unwrap());
    let state_machine = durability::SiftStateMachine::new(journal.clone());

    for (index, project, expected_cursor) in [(3, "project-a", 1), (4, "project-b", 2)] {
        let retry = durability::encode_raft_batch_at_for_diagnostics(
            vec![event_for_project(project)],
            &decision_time.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        state_machine.apply_local(index, &retry).unwrap();
        let result = state_machine.take_append_outcomes(index).unwrap();
        assert_eq!(result[0].raw_cursor, expected_cursor);
        assert!(result[0].duplicate);
    }
    assert_eq!(journal.total_event_count(), 2);
}

#[test]
fn every_voter_uses_the_raft_time_for_the_six_hour_duplicate_boundary() {
    let first_root = tempfile::tempdir().unwrap();
    let second_root = tempfile::tempdir().unwrap();
    let first_journal = Arc::new(DurableJournal::open(first_root.path()).unwrap());
    let second_journal = Arc::new(DurableJournal::open(second_root.path()).unwrap());
    let first = durability::SiftStateMachine::new(first_journal.clone());
    let second = durability::SiftStateMachine::new(second_journal.clone());
    // This time is intentionally far behind the process wall clock. The
    // second voter restarts before it receives the boundary command. Opening
    // the data directory must not prune the T0 decision that T5 still needs.
    let decision_time = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();

    let initial = command(decision_time);
    first.apply_local(1, &initial).unwrap();
    second.apply_local(1, &initial).unwrap();
    let first_initial = first.take_append_outcomes(1).unwrap();
    let second_initial = second.take_append_outcomes(1).unwrap();
    assert_eq!(first_initial, second_initial);
    assert!(!first_initial[0].duplicate);
    assert_eq!(first_initial[0].raw_cursor, 1);
    assert_eq!(first_initial[0].acknowledged_at, decision_time.to_rfc3339());

    drop(second);
    drop(second_journal);

    let boundary = command(decision_time + Duration::hours(6));
    first.apply_local(2, &boundary).unwrap();
    let second_journal = Arc::new(DurableJournal::open(second_root.path()).unwrap());
    let second = durability::SiftStateMachine::new(second_journal.clone());
    second.apply_local(2, &boundary).unwrap();
    let first_boundary = first.take_append_outcomes(2).unwrap();
    let second_boundary = second.take_append_outcomes(2).unwrap();
    assert_eq!(first_boundary, second_boundary);
    assert!(first_boundary[0].duplicate);
    assert_eq!(first_boundary[0].raw_cursor, 1);
    assert_eq!(
        first_boundary[0].acknowledged_at,
        first_initial[0].acknowledged_at
    );
    assert_eq!(first_journal.total_event_count(), 1);
    assert_eq!(second_journal.total_event_count(), 1);

    let outside = command(decision_time + Duration::hours(6) + Duration::nanoseconds(1));
    first.apply_local(3, &outside).unwrap();
    second.apply_local(3, &outside).unwrap();
    let first_outside = first.take_append_outcomes(3).unwrap();
    let second_outside = second.take_append_outcomes(3).unwrap();
    assert_eq!(first_outside, second_outside);
    assert!(!first_outside[0].duplicate);
    assert_eq!(first_outside[0].raw_cursor, 2);
    assert_eq!(
        first_outside[0].acknowledged_at,
        (decision_time + Duration::hours(6) + Duration::nanoseconds(1)).to_rfc3339()
    );
    assert_eq!(
        first_journal.query(EventQuery::default()).unwrap(),
        second_journal.query(EventQuery::default()).unwrap()
    );
}
