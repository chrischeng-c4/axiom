use std::{collections::BTreeMap, fs, os::unix::fs::PermissionsExt};

use sift::{DurableJournal, EventEnvelope, EventQuery, MetricPoint, MetricTemporality, SignalKind};

fn event(id: &str, signal: SignalKind) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        signal,
        match signal {
            SignalKind::Span => serde_json::json!({
                "name": "request",
                "start_time_unix_nano": 10,
                "end_time_unix_nano": 20
            }),
            _ => serde_json::json!({"message": id}),
        },
    );
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    if signal == SignalKind::Metric {
        event.metric = Some(MetricPoint {
            name: "requests".into(),
            value: 1.0,
            stale: false,
            unit: None,
            temporality: MetricTemporality::Delta,
            exemplars: Vec::new(),
        });
    }
    if signal == SignalKind::Span {
        event.trace_id = Some("trace-1".into());
        event.span_id = Some("span-1".into());
    }
    event
}

#[test]
fn each_phase_one_signal_has_its_own_wal_and_segment_tree() {
    let temp = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open(temp.path()).unwrap();
    journal.append(event("log-1", SignalKind::Log)).unwrap();
    journal
        .append(event("metric-1", SignalKind::Metric))
        .unwrap();
    journal.append(event("span-1", SignalKind::Span)).unwrap();

    for signal in ["logs", "metrics", "traces"] {
        let wal = temp.path().join("wal").join(signal).join("events.framed");
        assert!(
            wal.metadata().unwrap().len() > 0,
            "missing {}",
            wal.display()
        );
        assert_eq!(wal.metadata().unwrap().permissions().mode() & 0o777, 0o600);
        let segment_root = temp.path().join("segments").join(signal);
        assert!(fs::read_dir(&segment_root).unwrap().next().is_some());
    }
    assert!(!temp.path().join("raw-events.framed").exists());
    drop(journal);

    let reopened = DurableJournal::open(temp.path()).unwrap();
    let events = reopened
        .query(EventQuery {
            signal: None,
            after: 0,
            limit: 10,
        })
        .unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(
        events.iter().map(|event| event.cursor).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn the_public_signal_model_contains_only_phase_one_signals() {
    assert_eq!(
        SignalKind::ALL,
        [SignalKind::Log, SignalKind::Metric, SignalKind::Span]
    );
    assert!(serde_json::from_str::<SignalKind>(r#""profile""#).is_err());
    assert!(serde_json::from_str::<SignalKind>(r#""audit_event""#).is_err());
}
