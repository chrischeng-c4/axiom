use std::{collections::BTreeMap, sync::Arc, time::Duration};

use chrono::{SecondsFormat, Utc};
use sift::{
    projection::{
        LogQuery, MetricQuery, ProjectionRuntime, PROJECTION_LOGGING_STORE,
        PROJECTION_METRIC_STORE, PROJECTION_TRACE_STORE,
    },
    storage::archive,
    DurableJournal, EventEnvelope, MetricPoint, MetricTemporality, SignalKind,
};

fn base_event(id: &str, signal: SignalKind, occurred_at: chrono::DateTime<Utc>) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "archive-projection",
        "prod",
        id,
        signal,
        serde_json::json!({}),
    );
    event.occurred_at = occurred_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "projection-service".into())]);
    event
}

fn log(id: &str, occurred_at: chrono::DateTime<Utc>) -> EventEnvelope {
    let mut event = base_event(id, SignalKind::Log, occurred_at);
    event.payload = serde_json::json!({"message": id});
    event
}

fn metric(id: &str, occurred_at: chrono::DateTime<Utc>) -> EventEnvelope {
    let mut event = base_event(id, SignalKind::Metric, occurred_at);
    event.metric = Some(MetricPoint {
        name: "requests.total".into(),
        value: 42.0,
        stale: false,
        unit: Some("1".into()),
        temporality: MetricTemporality::Gauge,
        exemplars: Vec::new(),
    });
    event
}

fn span(id: &str, occurred_at: chrono::DateTime<Utc>) -> EventEnvelope {
    let start = u64::try_from(occurred_at.timestamp_nanos_opt().unwrap()).unwrap();
    let mut event = base_event(id, SignalKind::Span, occurred_at);
    event.trace_id = Some("cold-trace".into());
    event.span_id = Some("cold-span".into());
    event.payload = serde_json::json!({
        "name": "GET /checkout",
        "kind": "server",
        "start_time_unix_nano": start,
        "end_time_unix_nano": start + 1_000_000,
        "status": {"code": "ok", "message": "complete"},
        "links": [],
        "events": []
    });
    event
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retention_rebuild_reads_cold_archive_rows_and_the_local_suffix() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let address_text = address.to_string();
    let emulator = tokio::spawn(async move {
        vat::emulator::serve(vat::emulator::Kind::CloudStorage, &address_text)
            .await
            .unwrap();
    });
    let stop_emulator = emulator.abort_handle();
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(address).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    std::env::set_var("STORAGE_EMULATOR_HOST", format!("http://{address}"));

    tokio::task::spawn_blocking(move || {
        let now = Utc::now();
        let cold = now - chrono::Duration::days(31);
        let data = tempfile::tempdir().unwrap();
        let journal = Arc::new(DurableJournal::open(data.path()).unwrap());
        for event in [
            log("expired-log", now - chrono::Duration::days(179)),
            log("cold-log", cold),
            metric("cold-metric", cold),
            span("cold-span-event", cold),
        ] {
            journal.append(event).unwrap();
        }
        for index in 0..1_100 {
            journal
                .append(log(&format!("cold-log-{index:04}"), cold))
                .unwrap();
        }
        journal.storage().seal_all().unwrap();

        let runtime = ProjectionRuntime::open(data.path(), journal.clone()).unwrap();
        for projection in [
            PROJECTION_LOGGING_STORE,
            PROJECTION_METRIC_STORE,
            PROJECTION_TRACE_STORE,
        ] {
            runtime.catch_up(projection).unwrap();
        }
        archive::archive_journal_gcs(&journal, "gs://sift-projection-rebuild/archive").unwrap();
        let eviction = archive::evict_committed_cold_segments_at(&journal, now).unwrap();
        assert_eq!(eviction.evicted_events, 1_104);

        journal.append(log("hot-suffix", now)).unwrap();
        runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();
        runtime.persist_all().unwrap();

        let expiry =
            archive::expire_committed_events_at(&journal, now + chrono::Duration::days(2)).unwrap();
        assert_eq!(expiry.expired_events, 1);
        assert_eq!(journal.total_event_count(), 1_104);

        let mut log_query = LogQuery::for_project("archive-projection");
        log_query.limit = 1_000;
        let mut log_records = runtime.query_logs(&log_query).unwrap().records;
        log_query.after_cursor = log_records.last().unwrap().cursor;
        log_records.extend(runtime.query_logs(&log_query).unwrap().records);
        assert_eq!(log_records.len(), 1_102);
        assert!(log_records
            .iter()
            .any(|record| record.event_id == "cold-log"));
        assert!(log_records
            .iter()
            .any(|record| record.event_id == "cold-log-1099"));
        assert!(log_records
            .iter()
            .any(|record| record.event_id == "hot-suffix"));
        let metrics = runtime
            .query_metrics(&MetricQuery::for_project("archive-projection"))
            .unwrap();
        assert_eq!(metrics.series.len(), 1);
        assert_eq!(metrics.series[0].points[0].value, 42.0);
        assert!(runtime
            .get_trace("archive-projection", "cold-trace")
            .unwrap()
            .is_some());

        for projection in [
            PROJECTION_LOGGING_STORE,
            PROJECTION_METRIC_STORE,
            PROJECTION_TRACE_STORE,
        ] {
            let comparison = runtime.rebuild_and_compare(projection).unwrap();
            assert!(comparison.equal, "{projection} rebuild changed its meaning");
            assert_eq!(comparison.source_cursor, 1_105);
        }

        // Keep one projection scan open across a manifest replacement. The
        // second cold suffix is committed and its local files are evicted
        // after the session has exhausted the first manifest.
        let moving_data = tempfile::tempdir().unwrap();
        let moving = Arc::new(DurableJournal::open(moving_data.path()).unwrap());
        for index in 0..5 {
            moving
                .append(log(&format!("moving-{index}"), cold))
                .unwrap();
        }
        moving.storage().seal_all().unwrap();
        archive::archive_journal_gcs(&moving, "gs://sift-projection-moving/archive").unwrap();
        archive::evict_committed_cold_segments_at(&moving, now).unwrap();
        let mut moving_session = moving.projection_read_session(0).unwrap();
        assert_eq!(moving_session.read_next(5).unwrap().len(), 5);

        for index in 5..10 {
            moving
                .append(log(&format!("moving-{index}"), cold))
                .unwrap();
        }
        moving.storage().seal_all().unwrap();
        archive::archive_journal_gcs(&moving, "gs://sift-projection-moving/archive").unwrap();
        archive::evict_committed_cold_segments_at(&moving, now).unwrap();
        let moved_suffix = moving_session.read_next(10).unwrap();
        assert_eq!(moved_suffix.len(), 5);
        assert_eq!(moved_suffix[0].event.event_id, "moving-5");
        assert_eq!(moved_suffix[4].event.event_id, "moving-9");

        let mut reader = archive::CommittedEventReader::open(data.path(), 0)
            .unwrap()
            .unwrap();
        let first = reader.read_next(1_000).unwrap();
        assert_eq!(first.len(), 1_000);
        stop_emulator.abort();
        let mut scanned = first.len();
        loop {
            let page = reader.read_next(1_000).unwrap();
            if page.is_empty() {
                break;
            }
            scanned += page.len();
        }
        assert_eq!(scanned, 1_103);

        journal.append(log("hot-during-gcs-outage", now)).unwrap();
        runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();
        let mut outage_query = LogQuery::for_project("archive-projection");
        outage_query.after_cursor = 1_105;
        let logs = runtime.query_logs(&outage_query).unwrap();
        assert!(logs
            .records
            .iter()
            .any(|record| record.event_id == "hot-during-gcs-outage"));
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
