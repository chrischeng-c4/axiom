use std::collections::BTreeSet;

use sift::{
    projection::{LoggingProjection, MetricProjection, Projection, TraceProjection},
    EventEnvelope, MetricPoint, MetricTemporality, SignalKind, StoredEvent,
};

fn json_file_keys(root: &std::path::Path) -> BTreeSet<String> {
    fn collect(root: &std::path::Path, current: &std::path::Path, keys: &mut BTreeSet<String>) {
        for entry in std::fs::read_dir(current).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                collect(root, &entry.path(), keys);
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some("json") {
                let relative = entry.path().strip_prefix(root).unwrap().to_path_buf();
                keys.insert(
                    relative
                        .components()
                        .map(|component| component.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/"),
                );
            }
        }
    }

    let mut keys = BTreeSet::new();
    collect(root, root, &mut keys);
    keys
}

fn snapshot_metric_chunk_keys(snapshot: &[u8]) -> BTreeSet<String> {
    let snapshot: serde_json::Value = serde_json::from_slice(snapshot).unwrap();
    snapshot["state"]["series"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|series| series["sealed_chunks"].as_array().unwrap())
        .map(|chunk| chunk["key"].as_str().unwrap().to_owned())
        .collect()
}

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
        snapshot["location_by_cursor"].as_object().unwrap().len(),
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
fn metric_projection_bounds_retained_points_without_event_id_identity() {
    let projection = MetricProjection::with_limits(10, 10).unwrap();
    for cursor in 1..=100 {
        projection
            .apply_idempotent(&stored(cursor, SignalKind::Metric))
            .unwrap();
    }

    assert_eq!(
        projection
            .query(&sift::projection::MetricQuery::for_project("project-a"))
            .unwrap()
            .series[0]
            .points
            .len(),
        10
    );
}

#[test]
fn monotonic_metric_ingest_does_not_rewrite_the_retained_series_per_point() {
    let projection = MetricProjection::with_limits(10, 1_000).unwrap();
    for cursor in 1..=10_000 {
        let mut row = stored(cursor, SignalKind::Metric);
        row.event.occurred_at = "2026-07-14T00:00:00Z".into();
        row.event.observed_at.clone_from(&row.event.occurred_at);
        projection.apply_idempotent(&row).unwrap();
    }

    assert!(
        projection.maintenance_work_points() <= 20_000,
        "the monotonic write path must touch only the new point and bounded eviction work"
    );
}

#[test]
fn production_metric_projection_seals_chunks_and_bounds_its_memtable() {
    let data = tempfile::tempdir().unwrap();
    let memtable_limit = 16 * 1024;
    let projection =
        MetricProjection::open_with_limits(data.path(), 10, 20_000, memtable_limit).unwrap();
    for cursor in 1..=10_000 {
        let mut row = stored(cursor, SignalKind::Metric);
        row.event.occurred_at = "2026-07-14T00:00:00Z".into();
        row.event.observed_at.clone_from(&row.event.occurred_at);
        projection.apply_idempotent(&row).unwrap();
        assert!(projection.memtable_bytes() <= memtable_limit);
    }
    assert!(projection.sealed_chunk_count() > 1);

    let snapshot = projection.snapshot().unwrap();
    assert!(snapshot.len() < 256 * 1024);
    let restored =
        MetricProjection::open_with_limits(data.path(), 10, 20_000, memtable_limit).unwrap();
    restored.restore(&snapshot).unwrap();
    let page = restored
        .query(&sift::projection::MetricQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(page.series.len(), 1);
    assert_eq!(page.series[0].points.len(), 10_000);
}

#[test]
fn late_metric_chunk_replacement_waits_for_checkpoint_and_reclaims_after_restart() {
    let data = tempfile::tempdir().unwrap();
    let memtable_limit = 16 * 1024;
    let projection =
        MetricProjection::open_with_limits(data.path(), 10, 2_000, memtable_limit).unwrap();
    for cursor in 1..=600 {
        projection
            .apply_idempotent(&stored(cursor, SignalKind::Metric))
            .unwrap();
    }
    let committed_before = projection.snapshot().unwrap();
    projection.checkpoint_committed().unwrap();
    let chunk_root = data.path().join("indexes/metric-store/chunks");
    let initial_chunks = snapshot_metric_chunk_keys(&committed_before);
    assert!(initial_chunks.len() > 2);
    assert_eq!(json_file_keys(&chunk_root), initial_chunks);

    let mut late = stored(601, SignalKind::Metric);
    late.event.occurred_at = "2026-07-14T00:00:30.500000000Z".into();
    late.event.observed_at.clone_from(&late.event.occurred_at);
    projection.apply_idempotent(&late).unwrap();
    let committed_after = projection.snapshot().unwrap();
    let replacement_chunks = snapshot_metric_chunk_keys(&committed_after);
    assert!(!initial_chunks.is_disjoint(&replacement_chunks));
    assert!(!initial_chunks.is_subset(&replacement_chunks));
    assert!(!replacement_chunks.is_subset(&initial_chunks));
    let all_written = initial_chunks
        .union(&replacement_chunks)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(json_file_keys(&chunk_root), all_written);
    drop(projection);

    let reopened =
        MetricProjection::open_with_limits(data.path(), 10, 2_000, memtable_limit).unwrap();
    reopened.restore(&committed_before).unwrap();
    reopened.checkpoint_committed().unwrap();
    assert_eq!(
        reopened
            .query(&sift::projection::MetricQuery::for_project("project-a"))
            .unwrap()
            .series[0]
            .points
            .len(),
        600
    );
    assert_eq!(json_file_keys(&chunk_root), all_written);

    reopened.restore(&committed_after).unwrap();
    reopened.checkpoint_committed().unwrap();
    assert_eq!(
        reopened
            .query(&sift::projection::MetricQuery::for_project("project-a"))
            .unwrap()
            .series[0]
            .points
            .len(),
        601
    );
    assert_eq!(json_file_keys(&chunk_root), replacement_chunks);

    for cursor in 602..=610 {
        let mut repeated_late = stored(cursor, SignalKind::Metric);
        repeated_late.event.occurred_at = format!("2026-07-12T23:59:{:02}Z", cursor - 602);
        repeated_late
            .event
            .observed_at
            .clone_from(&repeated_late.event.occurred_at);
        reopened.apply_idempotent(&repeated_late).unwrap();
        let durable_snapshot = reopened.snapshot().unwrap();
        reopened.checkpoint_committed().unwrap();
        assert_eq!(
            json_file_keys(&chunk_root),
            snapshot_metric_chunk_keys(&durable_snapshot)
        );
    }
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
