//! # Facets
//!
//! - Behavior: an error-returning committed `Index` record keeps its applied
//!   Keyword prefix in the AOF and in a cold incremental checkpoint. A rejected
//!   Number replacement keeps its existing live deletion after cold recovery.
//! - Security: `apps/lumen/src/coordinator.rs:377` controls the AOF bytes
//!   recovery reads. `apps/lumen/src/storage.rs:4981` drops the caller value
//!   before fallible validation. These tests reject data loss and resurrection.
//! - Performance: these tests prove recovery correctness. The stage6 durable
//!   workload in `apps/lumen/e2e/perf_gate.rs:2865` measures checkpoint and
//!   merge work.

use std::sync::Arc;

use serde_json::{json, Value};

use lumen::log_entry::RaftLogEntry;
use lumen::storage::Engine;
use lumen::types::{FieldValue, QueryNode, SearchRequest, TermQuery};

#[path = "support/indexing_durable_fixture.rs"]
mod durable_fixture;

use durable_fixture::{
    checkpoint, create_schema, fixture, hit_ids, http_search, post_index, recover_from_checkpoint,
    sorted_hit_ids, COLLECTION,
};

// A committed `Index` record can return an error after changing the engine.
// These cases use the real coordinator/AOF/checkpoint route rather than a
// local Engine shortcut, because recovery consumes the persisted `RaftLogEntry`
// rather than an already-materialized index.
fn partial_error_index_entry(items: Vec<Value>) -> RaftLogEntry {
    RaftLogEntry::Index {
        collection_id: COLLECTION.to_owned(),
        req: serde_json::from_value(json!({ "items": items })).expect("partial-error index entry"),
    }
}

fn partial_error_term_ids(engine: &Arc<Engine>, field: &str, value: FieldValue) -> Vec<String> {
    sorted_hit_ids(
        engine
            .search(
                COLLECTION,
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: field.to_owned(),
                        value,
                    }),
                    limit: 20,
                    offset: 0,
                    cursor: None,
                    routing_key: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .expect("partial-error term query"),
    )
}

#[tokio::test]
async fn partial_error_mixed_index_replays_its_live_keyword_prefix_from_aof() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(
        &fixture.server,
        vec![json!({
            "external_id": "partial-error-baseline",
            "field": "kw",
            "value": "baseline",
        })],
    )
    .await;
    checkpoint(&fixture.server).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    let error = fixture
        .writer
        .submit(partial_error_index_entry(vec![
            json!({
                "external_id": "partial-error-prefix",
                "field": "kw",
                "value": "survives-error",
            }),
            json!({
                "external_id": "partial-error-prefix",
                "field": "unknown-after-prefix",
                "value": "must-error",
            }),
        ]))
        .await
        .expect_err("mixed index must report the unknown field after applying its valid prefix");
    assert!(
        format!("{error:#}").contains("unknown field `unknown-after-prefix`"),
        "the public write must retain its existing unknown-field error, got: {error:#}"
    );
    let committed_sequence = fixture.writer.applied_seq();
    assert_eq!(
        committed_sequence,
        checkpoint_sequence + 1,
        "the error return belongs to one concrete committed record"
    );

    let live = http_search(
        &fixture.server,
        json!({ "term": { "field": "kw", "value": "survives-error" } }),
        "live mixed-index prefix",
    )
    .await;
    assert_eq!(
        hit_ids(&live),
        vec!["partial-error-prefix"],
        "the valid prefix is visible even though the same committed record returned an error"
    );

    fixture
        .aof
        .lock()
        .expect("AOF lock")
        .sync_strict()
        .expect("strict-sync AOF tail");
    let (replayed_engine, recovered_checkpoint_sequence, replayed_sequence) =
        recover_from_checkpoint(&fixture);
    assert_eq!(
        recovered_checkpoint_sequence, checkpoint_sequence,
        "recovery starts from the published base before the partial-error record"
    );
    assert_eq!(
        replayed_sequence, committed_sequence,
        "a committed record with a visible valid prefix must be retained in the AOF tail"
    );
    assert_eq!(
        partial_error_term_ids(
            &replayed_engine,
            "kw",
            FieldValue::String("survives-error".to_owned()),
        ),
        vec!["partial-error-prefix".to_owned()],
        "AOF replay must restore the live valid prefix from an error-returning record"
    );

    checkpoint(&fixture.server).await;
    let (checkpointed_engine, checkpointed_sequence, checkpointed_tail) =
        recover_from_checkpoint(&fixture);
    assert_eq!(
        checkpointed_sequence, committed_sequence,
        "the incremental checkpoint must publish through the partial-error record"
    );
    assert_eq!(
        checkpointed_tail, 0,
        "the checkpointed partial-error record has no remaining AOF tail"
    );
    assert_eq!(
        partial_error_term_ids(
            &checkpointed_engine,
            "kw",
            FieldValue::String("survives-error".to_owned()),
        ),
        vec!["partial-error-prefix".to_owned()],
        "a cold incremental checkpoint must keep the prefix that live reads exposed"
    );
}

#[tokio::test]
async fn partial_error_number_reindex_checkpoint_never_resurrects_the_dropped_value() {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(
        &fixture.server,
        vec![json!({
            "external_id": "partial-error-number",
            "field": "num",
            "value": 41.0,
        })],
    )
    .await;
    checkpoint(&fixture.server).await;
    let checkpoint_sequence = fixture.writer.applied_seq();

    let error = fixture
        .writer
        .submit(partial_error_index_entry(vec![json!({
            "external_id": "partial-error-number",
            "field": "num",
            "value": "not-a-number",
        })]))
        .await
        .expect_err("a Number field must reject a string replacement");
    assert!(
        format!("{error:#}").contains("type mismatch on field `num`"),
        "the public write must retain its existing Number type-mismatch error, got: {error:#}"
    );
    let committed_sequence = fixture.writer.applied_seq();
    assert_eq!(
        committed_sequence,
        checkpoint_sequence + 1,
        "the error return belongs to one concrete committed Number record"
    );

    let live = http_search(
        &fixture.server,
        json!({ "term": { "field": "num", "value": 41.0 } }),
        "live Number after invalid reindex",
    )
    .await;
    assert!(
        hit_ids(&live).is_empty(),
        "the existing partial-error semantics drop the old Number value before returning its error"
    );

    checkpoint(&fixture.server).await;
    let (cold_engine, cold_sequence, replayed_tail) = recover_from_checkpoint(&fixture);
    assert_eq!(
        cold_sequence, committed_sequence,
        "the incremental checkpoint must cover the error-returning Number record"
    );
    assert_eq!(
        replayed_tail, 0,
        "the current generation must contain this cut without an uncheckpointed tail"
    );
    assert!(
        partial_error_term_ids(&cold_engine, "num", FieldValue::Number(41.0)).is_empty(),
        "a cold incremental checkpoint must not resurrect the Number value that live reads lost"
    );
}
