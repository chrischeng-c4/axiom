//! Cross-layout indexing and recovery oracle.
//! Focused AOF, publication, recovery, field, and compaction contracts have separate Cargo targets.

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

async fn run_history(field_major: bool) -> Value {
    let fixture = fixture();
    create_schema(&fixture.server).await;
    post_index(&fixture.server, corpus_items(field_major, PREFIX_DOCUMENTS)).await;

    let prefix_snapshot = snapshot(&fixture.engine);
    assert_eq!(
        keyword_postings(&prefix_snapshot, "kw", COMMON),
        PREFIX_DOCUMENTS
    );
    let first_sequence = fixture.writer.applied_seq();
    checkpoint(&fixture.server).await;
    let first = fixture
        .store
        .load_current_generation()
        .expect("load first CURRENT")
        .expect("first checkpoint");
    assert_eq!(first.sequence, first_sequence);
    assert_eq!(
        collection(&snapshot(&first.engine)).eid_fields.len(),
        PREFIX_DOCUMENTS
    );
    let first_snapshot = snapshot(&first.engine);
    // This asserted the postings map was EMPTY until #3957, under the reading
    // that a sealed field's postings "move out of the RAM snapshot". They do
    // move out of RAM — that is what the seal is for, and the reopened engine
    // below still answers because it holds the `.lseg`. But `to_snapshot` is
    // not a view of RAM: it is the self-contained document `GET /admin/backup`
    // returns, that `raft_sm` ships to a follower on another host, and that
    // `reshard` moves between shards. None of those readers has this node's
    // segment files, and `from_snapshot` sets `segment: None` — so a snapshot
    // truncated to the post-seal tail silently loses every sealed doc's
    // postings on the far side. This line is where that was pinned as intended.
    assert_eq!(
        keyword_postings(&first_snapshot, "kw", COMMON),
        PREFIX_DOCUMENTS,
        "#3957: the snapshot is self-contained, so a SEALED field's documents \
         must be in it and not only in the `.lseg` its reader may not have"
    );
    assert_keyword_total(&first.engine, COMMON, PREFIX_DOCUMENTS as u64);

    post_index(
        &fixture.server,
        corpus_items_range(field_major, PREFIX_DOCUMENTS, DOCUMENTS),
    )
    .await;
    index_updates(&fixture.server).await;
    fixture
        .aof
        .lock()
        .expect("aof lock")
        .sync_strict()
        .expect("strict-sync AOF");

    let live_digest = digest(&fixture.engine);
    assert_index_invariants(&fixture.engine);
    assert_queries(&fixture.engine);
    let (tail_reopened, _checkpoint_sequence, replayed) = recover_from_checkpoint(&fixture);
    assert!(
        replayed > first_sequence,
        "AOF replay must advance beyond boundary"
    );
    assert_eq!(digest(&tail_reopened), live_digest);
    assert_index_invariants(&tail_reopened);
    assert_queries(&tail_reopened);

    checkpoint(&fixture.server).await;
    let sealed = fixture
        .store
        .load_current_generation()
        .expect("load final CURRENT")
        .expect("final checkpoint");
    assert_eq!(sealed.sequence, fixture.writer.applied_seq());
    assert_eq!(digest(&sealed.engine), live_digest);
    assert_index_invariants(&sealed.engine);
    assert_queries(&sealed.engine);
    let (final_reopened, final_sequence, final_replayed) = recover_from_checkpoint(&fixture);
    assert_eq!(final_sequence, sealed.sequence);
    assert_eq!(final_replayed, 0);
    assert_eq!(digest(&final_reopened), live_digest);
    assert_index_invariants(&final_reopened);
    assert_queries(&final_reopened);
    live_digest
}

#[tokio::test]
async fn indexing_durable_oracle_converges_across_input_layouts() {
    let document_major = run_history(false).await;
    let field_major = run_history(true).await;
    assert_eq!(document_major, field_major);
}
