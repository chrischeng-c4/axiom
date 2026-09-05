// HANDWRITE-BEGIN gap="sift-metric-store-tests" tracker="1667" reason="Verify temporality, resets, histograms, exemplars, overflow, late points, rollups, and rebuild equality."
use std::{collections::BTreeMap, sync::Arc};

use sift::{
    projection::{
        MetricProjection, MetricQuery, Projection, ProjectionRuntime, PROJECTION_METRIC_STORE,
    },
    AttributeValue, DurableJournal, EventEnvelope, MetricExemplar, MetricPoint, MetricTemporality,
    SignalKind, StoredEvent,
};

fn metric(
    cursor: u64,
    id: &str,
    name: &str,
    value: f64,
    temporality: MetricTemporality,
    at: &str,
    label: &str,
    payload: serde_json::Value,
) -> StoredEvent {
    let mut event =
        EventEnvelope::for_project("project-a", "prod", id, SignalKind::Metric, payload);
    event.occurred_at = at.into();
    event.observed_at = at.into();
    event.resource = BTreeMap::from([("service.name".into(), "checkout".into())]);
    event
        .attributes
        .insert("route".into(), AttributeValue::String(label.into()));
    event.metric = Some(MetricPoint {
        name: name.into(),
        value,
        stale: false,
        unit: Some("1".into()),
        temporality,
        exemplars: vec![MetricExemplar {
            value,
            trace_id: format!("trace-{id}"),
            span_id: format!("span-{id}"),
        }],
    });
    StoredEvent {
        cursor,
        acknowledged_at: at.into(),
        event,
    }
}

fn explicit_histogram(counts: &[u64], sum: f64) -> serde_json::Value {
    serde_json::json!({
        "histogram": {
            "kind": "explicit",
            "count": counts.iter().sum::<u64>(),
            "sum": sum,
            "explicit_bounds": [10.0, 100.0],
            "bucket_counts": counts,
            "min": 1.0,
            "max": 101.0
        }
    })
}

fn exponential_histogram(counts: &[u64], sum: f64) -> serde_json::Value {
    serde_json::json!({
        "histogram": {
            "kind": "exponential",
            "count": counts.iter().sum::<u64>() + 1,
            "sum": sum,
            "scale": 2,
            "zero_count": 1,
            "positive_offset": 0,
            "positive_bucket_counts": counts,
            "negative_offset": 0,
            "negative_bucket_counts": [],
            "min": 0.0,
            "max": 16.0
        }
    })
}

#[test]
fn late_cumulative_points_detect_resets_and_roll_up_in_event_time() {
    let projection = MetricProjection::new();
    for row in [
        metric(
            1,
            "cumulative-1",
            "requests.total",
            10.0,
            MetricTemporality::Cumulative,
            "2026-07-14T00:00:00Z",
            "/pay",
            serde_json::json!({}),
        ),
        metric(
            2,
            "cumulative-3",
            "requests.total",
            3.0,
            MetricTemporality::Cumulative,
            "2026-07-14T00:00:20Z",
            "/pay",
            serde_json::json!({}),
        ),
        metric(
            3,
            "cumulative-2",
            "requests.total",
            15.0,
            MetricTemporality::Cumulative,
            "2026-07-14T00:00:10Z",
            "/pay",
            serde_json::json!({}),
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let page = projection
        .query(&MetricQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(page.series.len(), 1);
    let series = &page.series[0];
    assert_eq!(series.aggregate, Some(8.0));
    assert_eq!(series.reset_count, 1);
    assert_eq!(
        series
            .points
            .iter()
            .map(|point| point.value)
            .collect::<Vec<_>>(),
        [10.0, 15.0, 3.0]
    );
    assert_eq!(series.rollups.len(), 2);
    assert!(series.rollups.iter().all(|rollup| rollup.point_count == 3));
}

#[test]
fn histogram_exemplars_and_gauge_delta_semantics_survive_snapshot() {
    let projection = MetricProjection::new();
    for row in [
        metric(
            1,
            "hist-1",
            "request.duration",
            20.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/checkout",
            explicit_histogram(&[1, 2, 1], 80.0),
        ),
        metric(
            2,
            "hist-2",
            "request.duration",
            30.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:30Z",
            "/checkout",
            explicit_histogram(&[2, 1, 1], 120.0),
        ),
        metric(
            3,
            "gauge-1",
            "queue.depth",
            9.0,
            MetricTemporality::Gauge,
            "2026-07-14T00:00:00Z",
            "primary",
            serde_json::json!({}),
        ),
        metric(
            4,
            "gauge-2",
            "queue.depth",
            4.0,
            MetricTemporality::Gauge,
            "2026-07-14T00:00:10Z",
            "primary",
            serde_json::json!({}),
        ),
        metric(
            5,
            "exp-1",
            "payload.size",
            12.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/upload",
            exponential_histogram(&[2, 1], 12.0),
        ),
        metric(
            6,
            "exp-2",
            "payload.size",
            20.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:20Z",
            "/upload",
            exponential_histogram(&[1, 2], 20.0),
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let page = projection
        .query(&MetricQuery::for_project("project-a"))
        .unwrap();
    let histogram = page
        .series
        .iter()
        .find(|series| series.name == "request.duration")
        .unwrap();
    assert_eq!(histogram.aggregate, Some(50.0));
    assert_eq!(histogram.histogram.as_ref().unwrap().count, 8);
    assert_eq!(
        histogram.histogram.as_ref().unwrap().bucket_counts,
        [3, 3, 2]
    );
    assert_eq!(histogram.points[0].exemplars[0].trace_id, "trace-hist-1");
    let gauge = page
        .series
        .iter()
        .find(|series| series.name == "queue.depth")
        .unwrap();
    assert_eq!(gauge.aggregate, Some(4.0));
    let exponential = page
        .series
        .iter()
        .find(|series| series.name == "payload.size")
        .unwrap()
        .histogram
        .as_ref()
        .unwrap();
    assert_eq!(exponential.count, 8);
    assert_eq!(exponential.positive_bucket_counts, [3, 3]);
    assert_eq!(exponential.zero_count, 2);

    let snapshot = Projection::snapshot(&projection).unwrap();
    let restored = MetricProjection::new();
    Projection::restore(&restored, &snapshot).unwrap();
    assert_eq!(
        Projection::semantic_digest(&projection).unwrap(),
        Projection::semantic_digest(&restored).unwrap()
    );
}

#[test]
fn a_time_range_filters_raw_points_but_keeps_overlapping_full_bucket_rollups() {
    let projection = MetricProjection::new();
    for row in [
        metric(
            1,
            "range-1",
            "range.total",
            2.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/range",
            serde_json::json!({}),
        ),
        metric(
            2,
            "range-2",
            "range.total",
            3.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:30Z",
            "/range",
            serde_json::json!({}),
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let mut query = MetricQuery::for_project("project-a");
    query.start_time = Some("2026-07-14T00:00:20Z".into());
    let page = projection.query(&query).unwrap();
    assert_eq!(page.series[0].points.len(), 1);
    assert!(page.series[0]
        .rollups
        .iter()
        .all(|rollup| rollup.point_count == 2));
}

#[test]
fn restore_canonicalizes_legacy_cached_rollups_for_rebuild_digest_parity() {
    let projection = MetricProjection::new();
    for row in [
        metric(
            1,
            "legacy-rollup-1",
            "legacy.total",
            2.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/legacy",
            serde_json::json!({}),
        ),
        metric(
            2,
            "legacy-rollup-2",
            "legacy.total",
            3.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:30Z",
            "/legacy",
            serde_json::json!({}),
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let cached_rollups = serde_json::to_value(
        &projection
            .query(&MetricQuery::for_project("project-a"))
            .unwrap()
            .series[0]
            .rollups,
    )
    .unwrap();
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&Projection::snapshot(&projection).unwrap()).unwrap();
    for series in legacy["state"]["series"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        series["rollups"] = cached_rollups.clone();
    }

    let restored = MetricProjection::new();
    Projection::restore(&restored, &serde_json::to_vec(&legacy).unwrap()).unwrap();
    assert_eq!(
        Projection::semantic_digest(&projection).unwrap(),
        Projection::semantic_digest(&restored).unwrap()
    );
}

#[test]
fn cardinality_overflow_is_deterministic_and_diagnostic() {
    let projection = MetricProjection::with_limits(2, 100).unwrap();
    for (cursor, route) in ["/a", "/b", "/c", "/d"].into_iter().enumerate() {
        Projection::apply_idempotent(
            &projection,
            &metric(
                cursor as u64 + 1,
                &format!("overflow-{route}"),
                "requests",
                1.0,
                MetricTemporality::Delta,
                &format!("2026-07-14T00:00:0{cursor}Z"),
                route,
                serde_json::json!({}),
            ),
        )
        .unwrap();
    }
    let page = projection
        .query(&MetricQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(page.series.len(), 3);
    assert_eq!(page.overflowed_series, 2);
    assert_eq!(page.overflowed_points, 2);
    let overflow = page.series.iter().find(|series| series.overflow).unwrap();
    assert_eq!(overflow.points.len(), 2);
    assert_eq!(overflow.resource["sift.metric.overflow"], "true");
    assert!(overflow.attributes.is_empty());
}

#[test]
fn metric_projection_rebuilds_equal_from_raw_events() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    for row in [
        metric(
            1,
            "delta-1",
            "requests",
            2.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/a",
            serde_json::json!({}),
        ),
        metric(
            2,
            "delta-2",
            "requests",
            3.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:10Z",
            "/a",
            serde_json::json!({}),
        ),
    ] {
        journal.append(row.event).unwrap();
    }
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_METRIC_STORE).unwrap();
    let comparison = runtime
        .rebuild_and_compare(PROJECTION_METRIC_STORE)
        .unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
}

#[test]
fn identical_metric_event_ids_are_isolated_by_project() {
    let projection = MetricProjection::new();
    let project_a = metric(
        1,
        "shared-id",
        "requests",
        2.0,
        MetricTemporality::Delta,
        "2026-07-14T00:00:00Z",
        "/a",
        serde_json::json!({}),
    );
    let mut project_b = metric(
        2,
        "shared-id",
        "requests",
        3.0,
        MetricTemporality::Delta,
        "2026-07-14T00:00:01Z",
        "/a",
        serde_json::json!({}),
    );
    project_b.event.project = "project-b".into();

    Projection::apply_idempotent(&projection, &project_a).unwrap();
    Projection::apply_idempotent(&projection, &project_b).unwrap();

    assert_eq!(
        projection
            .query(&MetricQuery::for_project("project-a"))
            .unwrap()
            .series
            .len(),
        1
    );
    assert_eq!(
        projection
            .query(&MetricQuery::for_project("project-b"))
            .unwrap()
            .series
            .len(),
        1
    );
    let snapshot: serde_json::Value =
        serde_json::from_slice(&Projection::snapshot(&projection).unwrap()).unwrap();
    assert_eq!(snapshot["state"]["series"].as_object().unwrap().len(), 2);
}

#[test]
fn event_id_reuse_after_the_receipt_window_keeps_both_accepted_points() {
    let projection = MetricProjection::new();
    for row in [
        metric(
            1,
            "reused-id",
            "requests",
            2.0,
            MetricTemporality::Delta,
            "2026-07-14T00:00:00Z",
            "/reuse",
            serde_json::json!({}),
        ),
        metric(
            2,
            "reused-id",
            "requests",
            3.0,
            MetricTemporality::Delta,
            "2026-07-14T07:00:00Z",
            "/reuse",
            serde_json::json!({}),
        ),
    ] {
        Projection::apply_idempotent(&projection, &row).unwrap();
    }
    let page = projection
        .query(&MetricQuery::for_project("project-a"))
        .unwrap();
    assert_eq!(
        page.series[0]
            .points
            .iter()
            .map(|point| (point.cursor, point.value))
            .collect::<Vec<_>>(),
        [(1, 2.0), (2, 3.0)]
    );

    Projection::apply_idempotent(
        &projection,
        &metric(
            2,
            "reused-id",
            "requests",
            3.0,
            MetricTemporality::Delta,
            "2026-07-14T07:00:00Z",
            "/reuse",
            serde_json::json!({}),
        ),
    )
    .unwrap();
    assert_eq!(
        projection
            .query(&MetricQuery::for_project("project-a"))
            .unwrap()
            .series[0]
            .points
            .len(),
        2
    );
}

#[test]
fn broad_metric_query_materializes_only_limit_plus_one_matching_series() {
    let projection = MetricProjection::with_limits(1_000, 10).unwrap();
    for cursor in 1..=100_u64 {
        Projection::apply_idempotent(
            &projection,
            &metric(
                cursor,
                &format!("event-{cursor}"),
                "requests",
                cursor as f64,
                MetricTemporality::Delta,
                "2026-07-14T00:00:00Z",
                &format!("/{cursor}"),
                serde_json::json!({}),
            ),
        )
        .unwrap();
    }

    let mut query = MetricQuery::for_project("project-a");
    query.limit = 3;
    let first = projection.query(&query).unwrap();
    assert_eq!(first.series.len(), 3);
    assert!(first.has_more);
    query.after_series_id = first.next_series_id;
    let second = projection.query(&query).unwrap();
    assert_eq!(second.series.len(), 3);
    assert!(first.series.last().unwrap().series_id < second.series.first().unwrap().series_id);
}
// HANDWRITE-END
