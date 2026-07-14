// HANDWRITE-BEGIN gap="sift-profile-store-tests" tracker="1669" reason="Verify normalization, analysis, correlations, retention, snapshot, and raw rebuild equality."
use std::sync::Arc;

use chrono::{TimeZone, Utc};
use sift::{
    projection::{ProfileQuery, ProfileView, ProjectionRuntime, PROJECTION_PROFILE_STORE},
    DurableJournal, EventEnvelope, SignalKind,
};

const BASELINE_ID: &str = "11111111111111111111111111111111";
const COMPARISON_ID: &str = "22222222222222222222222222222222";
const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
const SPAN_ID: &str = "b7ad6b7169203331";

fn profile_event(
    event_id: &str,
    profile_id: &str,
    value: i64,
    time_unix_nano: u64,
    occurred_at: &str,
) -> EventEnvelope {
    let dictionary = serde_json::json!({
        "stringTable": ["", "cpu", "nanoseconds", "root", "leaf", "src/app.rs", "thread.name", "worker"],
        "mappingTable": [
            {},
            {"memoryStart":"4096","memoryLimit":"8192","fileOffset":"0","filenameStrindex":5,"attributeIndices":[]}
        ],
        "functionTable": [
            {},
            {"nameStrindex":3,"systemNameStrindex":3,"filenameStrindex":5,"startLine":10},
            {"nameStrindex":4,"systemNameStrindex":4,"filenameStrindex":5,"startLine":20}
        ],
        "locationTable": [
            {},
            {"mappingIndex":1,"address":"4100","lines":[{"functionIndex":1,"line":11,"column":1}]},
            {"mappingIndex":1,"address":"4200","lines":[{"functionIndex":2,"line":21,"column":1}]}
        ],
        "stackTable": [{}, {"locationIndices":[2,1]}],
        "attributeTable": [{}, {"keyStrindex":6,"value":{"stringValue":"worker"},"unitStrindex":0}],
        "linkTable": [{}, {"traceId":TRACE_ID,"spanId":SPAN_ID}]
    });
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        event_id,
        SignalKind::Profile,
        serde_json::json!({
            "profile": {
                "profileId": profile_id,
                "sampleType": {"typeStrindex":1,"unitStrindex":2},
                "samples": [{
                    "stackIndex":1,
                    "attributeIndices":[1],
                    "linkIndex":1,
                    "values":[value],
                    "timestampsUnixNano":[time_unix_nano.to_string()]
                }],
                "timeUnixNano": time_unix_nano.to_string(),
                "durationNano":"1000000000",
                "period":10000000,
                "attributeIndices":[1]
            },
            "dictionary": dictionary
        }),
    );
    event.occurred_at = occurred_at.into();
    event.observed_at.clone_from(&event.occurred_at);
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

#[test]
fn otel_dictionary_materializes_profile_topology_labels_and_correlations() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal
        .append(profile_event(
            "profile-baseline",
            BASELINE_ID,
            10,
            1_783_987_200_000_000_000,
            "2026-07-14T00:00:00Z",
        ))
        .unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_PROFILE_STORE).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 7, 15, 0, 0, 0).unwrap();

    let page = runtime
        .query_profiles(&ProfileQuery::for_project("project-a"), now)
        .unwrap();
    assert_eq!(page.records.len(), 1);
    let record = &page.records[0];
    assert_eq!(record.profile_id, BASELINE_ID);
    assert_eq!(record.sample_type, "cpu");
    assert_eq!(record.unit, "nanoseconds");
    assert_eq!(record.mappings[1].filename, "src/app.rs");
    assert_eq!(record.functions[2].name, "leaf");
    assert_eq!(record.locations[2].lines[0].function_id, 2);
    assert_eq!(record.samples[0].frames, ["root", "leaf"]);
    assert_eq!(record.samples[0].labels["thread.name"], "worker");
    assert_eq!(record.profile_labels["thread.name"], "worker");
    assert_eq!(record.samples[0].trace_id.as_deref(), Some(TRACE_ID));
    assert_eq!(record.samples[0].span_id.as_deref(), Some(SPAN_ID));

    let mut correlated = ProfileQuery::for_project("project-a");
    correlated.trace_id = Some(TRACE_ID.into());
    correlated.span_id = Some(SPAN_ID.into());
    assert_eq!(
        runtime
            .query_profiles(&correlated, now)
            .unwrap()
            .records
            .len(),
        1
    );
}

#[test]
fn flamegraph_top_functions_and_diff_are_deterministic() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal
        .append(profile_event(
            "profile-baseline",
            BASELINE_ID,
            10,
            1_783_987_200_000_000_000,
            "2026-07-14T00:00:00Z",
        ))
        .unwrap();
    journal
        .append(profile_event(
            "profile-comparison",
            COMPARISON_ID,
            25,
            1_783_987_201_000_000_000,
            "2026-07-14T00:00:01Z",
        ))
        .unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_PROFILE_STORE).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 7, 15, 0, 0, 0).unwrap();

    let mut flame = ProfileQuery::for_project("project-a");
    flame.view = ProfileView::Flamegraph;
    flame.profile_id = Some(BASELINE_ID.into());
    let flame = runtime.query_profiles(&flame, now).unwrap();
    assert_eq!(flame.flamegraph[0].frames, ["root", "leaf"]);
    assert_eq!(flame.flamegraph[0].value, 10.0);

    let mut top = ProfileQuery::for_project("project-a");
    top.view = ProfileView::TopFunctions;
    top.profile_id = Some(BASELINE_ID.into());
    let top = runtime.query_profiles(&top, now).unwrap();
    assert_eq!(top.functions[0].function, "leaf");
    assert_eq!(top.functions[0].self_value, 10.0);
    assert_eq!(top.functions[1].function, "root");

    let mut diff = ProfileQuery::for_project("project-a");
    diff.view = ProfileView::Diff;
    diff.baseline_profile_id = Some(BASELINE_ID.into());
    diff.comparison_profile_id = Some(COMPARISON_ID.into());
    let diff = runtime.query_profiles(&diff, now).unwrap();
    assert_eq!(diff.functions[0].baseline, Some(10.0));
    assert_eq!(diff.functions[0].comparison, Some(25.0));
    assert_eq!(diff.functions[0].delta, Some(15.0));
}

#[test]
fn retention_hides_hot_records_while_raw_rebuild_remains_equal() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal
        .append(profile_event(
            "profile-old",
            "33333333333333333333333333333333",
            5,
            1_705_276_800_000_000_000,
            "2024-01-15T00:00:00Z",
        ))
        .unwrap();
    journal
        .append(profile_event(
            "profile-current",
            BASELINE_ID,
            10,
            1_783_987_200_000_000_000,
            "2026-07-14T00:00:00Z",
        ))
        .unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    runtime.catch_up(PROJECTION_PROFILE_STORE).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 7, 15, 0, 0, 0).unwrap();
    let page = runtime
        .query_profiles(&ProfileQuery::for_project("project-a"), now)
        .unwrap();
    assert_eq!(page.records.len(), 1);
    assert_eq!(page.records[0].profile_id, BASELINE_ID);
    assert_eq!(journal.query(Default::default()).unwrap().len(), 2);

    let comparison = runtime
        .rebuild_and_compare(PROJECTION_PROFILE_STORE)
        .unwrap();
    assert!(comparison.equal);
    assert_eq!(comparison.source_cursor, 2);
}
// HANDWRITE-END
