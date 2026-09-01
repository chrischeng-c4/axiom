use sift::{
    projection::{LoggingProjection, MetricProjection, Projection, TraceProjection},
    EventEnvelope, MetricPoint, MetricTemporality, SignalKind, StoredEvent,
};

fn stored(cursor: u64, signal: SignalKind) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        format!("event-{cursor}"),
        signal,
        match signal {
            SignalKind::Span => serde_json::json!({
                "name": format!("span-{cursor}"),
                "start_time_unix_nano": cursor * 10,
                "end_time_unix_nano": cursor * 10 + 5
            }),
            SignalKind::Log => serde_json::json!({"message": format!("message-{cursor}")}),
            _ => serde_json::json!({}),
        },
    );
    if signal == SignalKind::Span {
        event.trace_id = Some(format!("trace-{cursor}"));
        event.span_id = Some(format!("span-{cursor}"));
    }
    if signal == SignalKind::Metric {
        event.occurred_at = format!("2026-07-14T00:00:{:02}Z", cursor % 60);
        event.observed_at.clone_from(&event.occurred_at);
        event.metric = Some(MetricPoint {
            name: "requests.total".into(),
            value: cursor as f64,
            stale: false,
            unit: Some("1".into()),
            temporality: MetricTemporality::Cumulative,
            exemplars: Vec::new(),
        });
    }
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
}

#[test]
fn trace_projection_bounds_spans_and_dedupe_metadata_together() {
    let projection = TraceProjection::with_max_spans(10).unwrap();
    for cursor in 1..=100 {
        projection
            .apply_idempotent(&stored(cursor, SignalKind::Span))
            .unwrap();
    }

    let snapshot: serde_json::Value =
        serde_json::from_slice(&projection.snapshot().unwrap()).unwrap();
    let span_count = snapshot["traces"]
        .as_object()
        .unwrap()
        .values()
        .map(|trace| trace.as_object().unwrap().len())
        .sum::<usize>();
    assert_eq!(span_count, 10);
    assert_eq!(
        snapshot["cursor_by_event_id"].as_object().unwrap().len(),
        10
    );
    assert!(projection
        .get_trace("project-a", "trace-1")
        .unwrap()
        .is_none());
    assert!(projection
        .get_trace("project-a", "trace-100")
        .unwrap()
        .is_some());
}

#[test]
fn metric_projection_evicts_dedupe_rows_with_old_points() {
    let projection = MetricProjection::with_limits(10, 10).unwrap();
    for cursor in 1..=100 {
        projection
            .apply_idempotent(&stored(cursor, SignalKind::Metric))
            .unwrap();
    }

    let snapshot: serde_json::Value =
        serde_json::from_slice(&projection.snapshot().unwrap()).unwrap();
    assert_eq!(
        snapshot["state"]["cursor_by_event_id"]
            .as_object()
            .unwrap()
            .len(),
        10
    );
}

#[test]
fn logging_projection_deletes_evicted_documents_from_shared_text_index() {
    let projection = LoggingProjection::with_max_records(10).unwrap();
    for cursor in 1..=100 {
        projection
            .apply_idempotent(&stored(cursor, SignalKind::Log))
            .unwrap();
    }

    let snapshot: serde_json::Value =
        serde_json::from_slice(&projection.snapshot().unwrap()).unwrap();
    assert_eq!(snapshot["records"].as_object().unwrap().len(), 10);
    assert_eq!(
        snapshot["text_index"]["documents"]
            .as_array()
            .unwrap()
            .len(),
        10
    );
}
