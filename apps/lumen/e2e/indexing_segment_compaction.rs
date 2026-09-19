//! Segment compaction, merge scheduling, and retained-generation contracts.

#[path = "support/indexing_durable_catalog_fixture.rs"]
mod catalog_fixture;

use catalog_fixture::*;

mod first_compaction_contract {
    //! # Facets
    //!
    //! - Behavior: v2_four_keyword_deltas_compact_and_preserve_tombstone_across_live_cold_and_retained_base
    //!   writes a real base and four real sparse checkpoints through SegmentRdbStore.
    //!   It requires layer reduction, then checks current live and cold searches
    //!   plus a retained first generation.
    //! - Security: apps/lumen/src/segment_rdb.rs:543 reads the local generation
    //!   bytes on cold open. Existing index_durable_oracle refusal cases beginning
    //!   at apps/lumen/e2e/indexing_durable_oracle.rs:970 feed malformed catalog
    //!   bytes and preserve CURRENT. This compaction case uses the same reader for
    //!   the compacted generation and the retained predecessor.
    //! - Performance: .aw/workitems/deliveries/lumen061-04-bounded-compaction-backpressure.md:3
    //!   and :30 require a compaction request at four deltas and a hard limit of
    //!   sixteen. This bounded structural case asserts the four-delta reduction.
    //!   It does not measure the separate stage-6 latency or RSS budget.
    //!
    //! Append this source to apps/lumen/e2e/indexing_durable_oracle.rs. It relies
    //! only on helpers already in that target and uses no compactor-specific API.

    #[cfg(unix)]
    const FIRST_COMPACTION_COLLECTION: &str = "first-compaction";
    #[cfg(unix)]
    const FIRST_COMPACTION_BASE_ROWS: usize = 256;
    #[cfg(unix)]
    const FIRST_COMPACTION_BASE_SEQUENCE: u64 = 9_100;
    #[cfg(unix)]
    const FIRST_COMPACTION_HOT_ID: &str = "first-compaction-hot";
    #[cfg(unix)]
    const FIRST_COMPACTION_DELETED_ID: &str = "first-compaction-deleted";
    #[cfg(unix)]
    const FIRST_COMPACTION_UNTOUCHED_ID: &str = "first-compaction-base-000";

    #[cfg(unix)]
    struct FirstCompactionFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        store: SegmentRdbStore,
        engine: Arc<Engine>,
        server: TestServer,
        base_name: String,
    }

    #[cfg(unix)]
    fn first_compaction_base_id(index: usize) -> String {
        format!("first-compaction-base-{index:03}")
    }

    #[cfg(unix)]
    fn first_compaction_base_value(index: usize) -> String {
        // Do not use a repeated prefix here. Keyword dictionaries compact common
        // prefixes very well, which could make a visually large base physically
        // smaller than four sparse files. These deterministic high-entropy terms
        // make the physical-size premise below meaningful without randomness.
        first_compaction_entropy_term(index as u64 + 1)
    }

    #[cfg(unix)]
    fn first_compaction_round_value(round: usize) -> String {
        // Every update has the same row count and term length. The largest of the
        // first three physical layers is thus the measured fourth-layer bound.
        first_compaction_entropy_term(10_000 + round as u64)
    }

    #[cfg(unix)]
    fn first_compaction_entropy_term(seed: u64) -> String {
        let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
        let mut value = String::with_capacity(512);
        for _ in 0..32 {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
            value.push_str(&format!("{state:016x}"));
        }
        value
    }

    #[cfg(unix)]
    async fn first_compaction_create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{FIRST_COMPACTION_COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }

    #[cfg(unix)]
    async fn first_compaction_index(server: &TestServer, items: Vec<Value>) {
        for chunk in items.chunks(1_000) {
            server
                .post(&format!("/collections/{FIRST_COMPACTION_COLLECTION}/index"))
                .json(&json!({ "items": chunk }))
                .await
                .assert_status_ok();
        }
    }

    #[cfg(unix)]
    async fn first_compaction_fixture() -> FirstCompactionFixture {
        let dir = tempfile::tempdir().expect("first compaction fixture root");
        let root = dir.path().join("segments");
        let store = SegmentRdbStore::new(&root).expect("create first compaction store");
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine.clone())))
            .expect("first compaction HTTP server");
        first_compaction_create_collection(&server).await;

        let mut items: Vec<_> = (0..FIRST_COMPACTION_BASE_ROWS)
            .map(|index| {
                json!({
                    "external_id": first_compaction_base_id(index),
                    "field": "kw",
                    "value": first_compaction_base_value(index),
                })
            })
            .collect();
        items.extend([
            json!({
                "external_id": FIRST_COMPACTION_HOT_ID,
                "field": "kw",
                "value": "first-hot-base",
            }),
            json!({
                "external_id": FIRST_COMPACTION_DELETED_ID,
                "field": "kw",
                "value": "first-deleted-base",
            }),
        ]);
        first_compaction_index(&server, items).await;
        stage1_restore_legacy_base(&engine, "first compaction base");
        store
            .save(&engine, FIRST_COMPACTION_BASE_SEQUENCE)
            .expect("publish first compaction base");

        FirstCompactionFixture {
            _dir: dir,
            root: root.clone(),
            store,
            engine,
            server,
            base_name: stage1_reuse_current_name(&root),
        }
    }

    #[cfg(unix)]
    async fn first_compaction_apply_round(server: &TestServer, round: usize) {
        first_compaction_index(
            server,
            vec![json!({
                "external_id": FIRST_COMPACTION_HOT_ID,
                "field": "kw",
                "value": first_compaction_round_value(round),
            })],
        )
        .await;
        if round == 1 {
            server
                .delete(&format!(
                "/collections/{FIRST_COMPACTION_COLLECTION}/index/{FIRST_COMPACTION_DELETED_ID}"
            ))
                .await
                .assert_status(axum::http::StatusCode::NO_CONTENT);
        }
    }

    #[cfg(unix)]
    fn first_compaction_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
        let mut refs: Vec<_> =
            stage1_reuse_catalog_collection(manifest, FIRST_COMPACTION_COLLECTION)["segments"]
                .as_array()
                .expect("first compaction catalog segments")
                .iter()
                .filter(|segment| {
                    segment["role"] == json!("field")
                        && segment["field"] == json!("kw")
                        && segment["kind"] == json!(kind)
                })
                .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("keyword ordinal"));
        refs
    }

    #[cfg(unix)]
    fn first_compaction_base_ref<'a>(manifest: &'a Value) -> &'a Value {
        first_compaction_refs(manifest, "base")
            .into_iter()
            .find(|segment| segment["ordinal"] == json!(0))
            .expect("keyword base reference")
    }

    #[cfg(unix)]
    fn first_compaction_delta_sizes(
        generation: &Path,
        manifest: &Value,
        expected_sequences: &[u64],
    ) -> Vec<u64> {
        let deltas = first_compaction_refs(manifest, "delta");
        stage1_assert_delta_sequence_order(
            &deltas,
            expected_sequences,
            "first compaction pre-merge Keyword layers",
        );
        deltas
            .into_iter()
            .map(|delta| {
                std::fs::metadata(
                    generation.join(delta["path"].as_str().expect("delta segment path")),
                )
                .expect("inspect delta segment")
                .len()
                    + std::fs::metadata(stage1_keyword_delta_rows_path(generation, delta))
                        .expect("inspect delta row map")
                        .len()
            })
            .collect()
    }

    #[cfg(unix)]
    fn first_compaction_assert_base_hardlinked(
        base_generation: &Path,
        base_manifest: &Value,
        latest_generation: &Path,
        latest_manifest: &Value,
    ) {
        let original = base_generation.join(
            first_compaction_base_ref(base_manifest)["path"]
                .as_str()
                .expect("base keyword path"),
        );
        let retained = latest_generation.join(
            first_compaction_base_ref(latest_manifest)["path"]
                .as_str()
                .expect("latest keyword base path"),
        );
        let original_metadata =
            std::fs::symlink_metadata(&original).expect("inspect original base");
        let retained_metadata =
            std::fs::symlink_metadata(&retained).expect("inspect retained base");
        assert!(original_metadata.is_file() && !original_metadata.file_type().is_symlink());
        assert!(retained_metadata.is_file() && !retained_metadata.file_type().is_symlink());
        assert_eq!(
            original_metadata.ino(),
            retained_metadata.ino(),
            "small sparse deltas must merge without rewriting the older keyword base"
        );
        assert!(
            retained_metadata.nlink() >= 2,
            "retained keyword base must be a hard link in the compacted generation"
        );
    }

    #[cfg(unix)]
    async fn first_compaction_search_ids(server: &TestServer, value: &str) -> Vec<String> {
        let response = server
            .post(&format!(
                "/collections/{FIRST_COMPACTION_COLLECTION}/search"
            ))
            .json(&json!({
                "query": { "term": { "field": "kw", "value": value } },
                "limit": 32,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let mut ids: Vec<_> = body["hits"]
            .as_array()
            .expect("keyword search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("keyword hit ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "bounded keyword total"
        );
        ids
    }

    #[cfg(unix)]
    async fn first_compaction_assert_state(server: &TestServer, phase: &str) {
        assert_eq!(
            first_compaction_search_ids(server, &first_compaction_round_value(4)).await,
            vec![FIRST_COMPACTION_HOT_ID.to_owned()],
            "{phase}: newest delta value remains visible"
        );
        assert!(
            first_compaction_search_ids(server, &first_compaction_round_value(1))
                .await
                .is_empty(),
            "{phase}: compaction must mask an older hot value"
        );
        assert!(
            first_compaction_search_ids(server, "first-deleted-base")
                .await
                .is_empty(),
            "{phase}: compaction must retain the document tombstone"
        );
        assert_eq!(
            first_compaction_search_ids(server, &first_compaction_base_value(0)).await,
            vec![FIRST_COMPACTION_UNTOUCHED_ID.to_owned()],
            "{phase}: compaction retains untouched base data"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn v2_four_keyword_deltas_compact_and_preserve_tombstone_across_live_cold_and_retained_base(
    ) {
        let fixture = first_compaction_fixture().await;
        let base_generation = fixture.root.join(&fixture.base_name);
        let base_manifest = stage1_read_manifest(&base_generation);
        let base_bytes = std::fs::metadata(
            base_generation.join(
                first_compaction_base_ref(&base_manifest)["path"]
                    .as_str()
                    .expect("original base path"),
            ),
        )
        .expect("inspect original keyword base")
        .len();

        for round in 1..=3 {
            first_compaction_apply_round(&fixture.server, round).await;
            fixture
                .store
                .save(
                    &fixture.engine,
                    FIRST_COMPACTION_BASE_SEQUENCE + round as u64,
                )
                .expect("publish sparse keyword checkpoint");
        }
        let third_generation = stage1_current_generation_dir(&fixture.root);
        let third_manifest = stage1_read_manifest(&third_generation);
        let third_sizes = first_compaction_delta_sizes(
            &third_generation,
            &third_manifest,
            &[
                FIRST_COMPACTION_BASE_SEQUENCE + 1,
                FIRST_COMPACTION_BASE_SEQUENCE + 2,
                FIRST_COMPACTION_BASE_SEQUENCE + 3,
            ],
        );
        assert_eq!(
            third_sizes.len(),
            3,
            "the fourth checkpoint alone must request the first compaction"
        );
        let third_bytes: u64 = third_sizes.iter().sum();
        let fourth_upper_bound = *third_sizes
            .iter()
            .max()
            .expect("three physical sparse delta sizes");
        assert!(
        third_bytes
            .checked_add(fourth_upper_bound)
            .expect("bounded fourth sparse layer byte total")
            < base_bytes,
        "fixture precondition: the first three actual sparse layers ({third_bytes}) plus a bounded fourth layer ({fourth_upper_bound}) must stay below the actual base ({base_bytes}), so the fourth-delta request may not merge the base"
    );

        first_compaction_apply_round(&fixture.server, 4).await;
        fixture
            .store
            .save(&fixture.engine, FIRST_COMPACTION_BASE_SEQUENCE + 4)
            .expect("publish fourth sparse keyword checkpoint");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for first keyword compaction");

        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        let deltas = first_compaction_refs(&latest_manifest, "delta");
        assert!(
        deltas.len() < 4,
        "the fourth keyword delta must request and publish a reduction before the next checkpoint; got {} catalogued layers",
        deltas.len()
    );
        assert!(
            deltas.len() <= 16,
            "a published keyword catalog must stay below the sixteen-delta hard limit"
        );
        first_compaction_assert_base_hardlinked(
            &base_generation,
            &base_manifest,
            &latest_generation,
            &latest_manifest,
        );
        first_compaction_assert_state(&fixture.server, "live compacted checkpoint").await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FIRST_COMPACTION_BASE_SEQUENCE + 4,
            "cold CURRENT carries the fourth checkpoint watermark"
        );
        let cold_server =
            TestServer::new(router(AppState::open(cold_engine))).expect("cold compacted server");
        first_compaction_assert_state(&cold_server, "cold compacted checkpoint").await;

        let (base_engine, base_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
        assert_eq!(
            base_sequence, FIRST_COMPACTION_BASE_SEQUENCE,
            "retained base generation preserves its original sequence"
        );
        let base_server = TestServer::new(router(AppState::open(base_engine)))
            .expect("retained base cold server");
        assert_eq!(
            first_compaction_search_ids(&base_server, "first-hot-base").await,
            vec![FIRST_COMPACTION_HOT_ID.to_owned()],
            "retained first generation keeps its original hot value"
        );
        assert_eq!(
            first_compaction_search_ids(&base_server, "first-deleted-base").await,
            vec![FIRST_COMPACTION_DELETED_ID.to_owned()],
            "retained first generation keeps data predating the later tombstone"
        );
    }

    use super::*;
}

#[cfg(unix)]
mod full_compaction_contract {
    //! # Facets
    //!
    //! - Behavior: `v2_base_eligible_keyword_compaction_keeps_all_field_results_live_cold_and_retained`
    //!   drives only public collection, index, replace, delete, search, and
    //!   `SegmentRdbStore::save` operations. It proves a measured base-eligibility
    //!   premise, requires a Keyword base replacement, and compares all seven
    //!   field types against independent live Engines and retained generations.
    //!   `v2_partial_keyword_compaction_reduces_the_measured_delta_stack_below_base`
    //!   separately pins the pair policy: below the measured base-eligibility
    //!   threshold, one job folds one adjacent delta pair, the base stays
    //!   untouched, and the retained pre-compaction generation still holds the
    //!   replaced layers' bytes.
    //! - Security: `apps/lumen/src/segment_rdb.rs:450-480` writes and validates
    //!   the staged catalog before publication. The existing malformed-CURRENT
    //!   refusals at `apps/lumen/e2e/indexing_durable_oracle.rs:970-1018` cover
    //!   the same persisted catalog trust boundary and assert that CURRENT is not
    //!   changed. This case cold-opens both the compacted and retained bytes; it
    //!   adds no new caller-controlled path or format.
    //! - Performance: the user-approved #4246 plan for this non-AW run requires
    //!   a compaction request at four deltas, a hard limit of sixteen, and a base
    //!   merge only when total delta bytes meet or exceed base bytes. The measured
    //!   threshold, base replacement, hard-link checks, and <=16 assertions carry
    //!   that structural promise. This case does not claim a latency or RSS result.

    use super::*;

    const FULL_COMPACTION_COLLECTION: &str = "full-base-compaction";
    const FULL_COMPACTION_BASE_ROWS: usize = 256;
    const FULL_COMPACTION_UPDATED_ROWS: usize = 64;
    const FULL_COMPACTION_BASE_SEQUENCE: u64 = 9_200;
    const FULL_COMPACTION_HOT_ID: &str = "full-compaction-hot";
    const FULL_COMPACTION_DELETED_ID: &str = "full-compaction-deleted";
    const FULL_COMPACTION_APPENDED_ID: &str = "full-compaction-appended";
    const FULL_COMPACTION_UNTOUCHED_ID: &str = "full-compaction-base-255";

    const FULL_KEYWORD: &str = "kw";
    const FULL_NUMBER: &str = "num";
    const FULL_SET: &str = "tags";
    const FULL_HASH: &str = "sig";
    const FULL_TEXT: &str = "body";
    const FULL_FLAT: &str = "flat";
    const FULL_HNSW: &str = "hnsw";

    const FULL_HOT_BASE_KEYWORD: &str = "full-hot-keyword-base";
    const FULL_HOT_FINAL_KEYWORD: &str = "full-hot-keyword-final";
    const FULL_DELETED_KEYWORD: &str = "full-deleted-keyword";
    const FULL_APPENDED_KEYWORD: &str = "full-appended-keyword";
    const FULL_HOT_BASE_NUMBER: f64 = 90_001.0;
    const FULL_HOT_FINAL_NUMBER: f64 = 90_002.0;
    const FULL_APPENDED_NUMBER: f64 = 90_003.0;
    const FULL_HOT_BASE_TAG: &str = "full-hot-tag-base";
    const FULL_APPENDED_TAG: &str = "full-appended-tag";
    const FULL_HOT_BASE_HASH: &str = "000000000000fa01";
    const FULL_HOT_FINAL_HASH: &str = "000000000000fa02";
    const FULL_APPENDED_HASH: &str = "000000000000fa03";
    const FULL_HOT_BASE_TEXT: &str = "full-text-old full-common";
    const FULL_HOT_FINAL_TEXT: &str = "full-text-final full-common";
    const FULL_APPENDED_TEXT: &str = "full-text-appended full-common";

    struct FullCompactionFixture {
        _dir: tempfile::TempDir,
        root: PathBuf,
        store: SegmentRdbStore,
        engine: Arc<Engine>,
        server: TestServer,
        base_name: String,
    }

    fn full_compaction_base_id(index: usize) -> String {
        format!("full-compaction-base-{index:03}")
    }

    fn full_compaction_entropy_term(seed: u64, words: usize) -> String {
        let mut state = seed ^ 0xd1b5_4a32_d192_ed03;
        let mut value = String::with_capacity(words * 16);
        for _ in 0..words {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
            value.push_str(&format!("{state:016x}"));
        }
        value
    }

    fn full_compaction_base_keyword(index: usize) -> String {
        // Values have no shared textual prefix. This keeps the observed base size
        // tied to real distinct keyword bytes instead of dictionary compression.
        full_compaction_entropy_term(index as u64 + 1, 16)
    }

    fn full_compaction_round_keyword(round: usize, index: usize) -> String {
        // Each changed Keyword value is much larger than its base value. The test
        // still proves base eligibility from catalogued physical bytes, never from
        // this requested payload size alone.
        full_compaction_entropy_term(100_000 + (round * 1_000 + index) as u64, 64)
    }

    fn full_compaction_base_number(index: usize) -> f64 {
        1_000.0 + index as f64
    }

    fn full_compaction_base_tag(index: usize) -> String {
        format!("full-base-tag-{index:03}")
    }

    fn full_compaction_base_hash(index: usize) -> String {
        format!("{:016x}", 0x1_0000_u64 + index as u64)
    }

    fn full_compaction_base_text(index: usize) -> String {
        format!("full-common full-base-text-{index:03}")
    }

    fn full_compaction_flat_x(index: usize) -> f32 {
        10_000.0 + index as f32
    }

    fn full_compaction_hnsw_x(index: usize) -> f32 {
        20_000.0 + index as f32
    }

    fn full_compaction_vector(x: f32) -> Value {
        // The HNSW witness uses L2. The query rebuilds this exact vector, so
        // its intended ID has distance zero. One label coordinate prevents a
        // collision, while seven deterministic mixed coordinates avoid the
        // old all-collinear `[x, 0]` shape that permitted an approximate line
        // neighbour to win. This changes no expected ID or score assertion.
        let mut state = (x as u32)
            .wrapping_mul(0x9e37_79b9)
            .wrapping_add(0x7f4a_7c15);
        let mut values = Vec::with_capacity(8);
        values.push(Value::from(x as f64 / 10_000.0));
        values.extend((0..7).map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            Value::from(((state >> 8) & 0xffff) as f64 / 8_192.0 - 4.0)
        }));
        Value::Array(values)
    }

    fn full_compaction_document_items(
        external_id: &str,
        keyword: String,
        number: f64,
        tag: String,
        hash: String,
        body: String,
        flat_x: f32,
        hnsw_x: f32,
    ) -> Vec<Value> {
        vec![
            json!({ "external_id": external_id, "field": FULL_KEYWORD, "value": keyword }),
            json!({ "external_id": external_id, "field": FULL_NUMBER, "value": number }),
            json!({ "external_id": external_id, "field": FULL_SET, "value": [tag] }),
            json!({ "external_id": external_id, "field": FULL_HASH, "value": hash }),
            json!({ "external_id": external_id, "field": FULL_TEXT, "value": body }),
            json!({ "external_id": external_id, "field": FULL_FLAT, "value": full_compaction_vector(flat_x) }),
            json!({ "external_id": external_id, "field": FULL_HNSW, "value": full_compaction_vector(hnsw_x) }),
        ]
    }

    fn full_compaction_base_items() -> Vec<Value> {
        let mut items = Vec::with_capacity((FULL_COMPACTION_BASE_ROWS + 2) * 7);
        for index in 0..FULL_COMPACTION_BASE_ROWS {
            items.extend(full_compaction_document_items(
                &full_compaction_base_id(index),
                full_compaction_base_keyword(index),
                full_compaction_base_number(index),
                full_compaction_base_tag(index),
                full_compaction_base_hash(index),
                full_compaction_base_text(index),
                full_compaction_flat_x(index),
                full_compaction_hnsw_x(index),
            ));
        }
        items.extend(full_compaction_document_items(
            FULL_COMPACTION_HOT_ID,
            FULL_HOT_BASE_KEYWORD.to_owned(),
            FULL_HOT_BASE_NUMBER,
            FULL_HOT_BASE_TAG.to_owned(),
            FULL_HOT_BASE_HASH.to_owned(),
            FULL_HOT_BASE_TEXT.to_owned(),
            1.0,
            2.0,
        ));
        items.extend(full_compaction_document_items(
            FULL_COMPACTION_DELETED_ID,
            FULL_DELETED_KEYWORD.to_owned(),
            80_001.0,
            "full-deleted-tag".to_owned(),
            "000000000000fb01".to_owned(),
            "full-text-deleted full-common".to_owned(),
            3.0,
            4.0,
        ));
        items
    }

    async fn full_compaction_create_collection(server: &TestServer) {
        server
            .put(&format!("/collections/{FULL_COMPACTION_COLLECTION}"))
            .json(&json!({ "fields": {
                FULL_KEYWORD: { "type": "keyword" },
                FULL_NUMBER: { "type": "number" },
                FULL_SET: { "type": "set" },
                FULL_HASH: { "type": "hash" },
                FULL_TEXT: { "type": "text", "analyzer": "whitespace_lower" },
                FULL_FLAT: {
                    "type": "vector", "dim": 8, "metric": "l2", "backend": "flat-cpu"
                },
                FULL_HNSW: {
                    "type": "vector", "dim": 8, "metric": "l2", "backend": "hnsw-cpu"
                },
            }}))
            .await
            .assert_status_ok();
    }

    async fn full_compaction_post_items(server: &TestServer, items: Vec<Value>) {
        for chunk in items.chunks(1_000) {
            server
                .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/index"))
                .json(&json!({ "items": chunk }))
                .await
                .assert_status_ok();
        }
    }

    async fn full_compaction_fixture() -> FullCompactionFixture {
        let dir = tempfile::tempdir().expect("full compaction fixture root");
        let root = dir.path().join("segments");
        let store = SegmentRdbStore::new(&root).expect("create full compaction store");
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine.clone())))
            .expect("full compaction HTTP server");
        full_compaction_create_collection(&server).await;
        full_compaction_post_items(&server, full_compaction_base_items()).await;
        stage1_restore_legacy_base(&engine, "full compaction base");
        store
            .save(&engine, FULL_COMPACTION_BASE_SEQUENCE)
            .expect("publish full compaction base");
        FullCompactionFixture {
            _dir: dir,
            root: root.clone(),
            store,
            engine,
            server,
            base_name: stage1_reuse_current_name(&root),
        }
    }

    async fn full_compaction_reference() -> TestServer {
        let engine = Arc::new(Engine::new());
        let server = TestServer::new(router(AppState::open(engine)))
            .expect("full compaction reference server");
        full_compaction_create_collection(&server).await;
        full_compaction_post_items(&server, full_compaction_base_items()).await;
        server
    }

    async fn full_compaction_apply_keyword_round(server: &TestServer, round: usize) {
        let mut items: Vec<_> = (0..FULL_COMPACTION_UPDATED_ROWS)
            .map(|index| {
                json!({
                    "external_id": full_compaction_base_id(index),
                    "field": FULL_KEYWORD,
                    "value": full_compaction_round_keyword(round, index),
                })
            })
            .collect();
        items.push(json!({
            "external_id": FULL_COMPACTION_HOT_ID,
            "field": FULL_KEYWORD,
            "value": full_compaction_round_keyword(round, FULL_COMPACTION_UPDATED_ROWS),
        }));
        full_compaction_post_items(server, items).await;
        if round == 1 {
            server
                .delete(&format!(
                    "/collections/{FULL_COMPACTION_COLLECTION}/index/{FULL_COMPACTION_DELETED_ID}"
                ))
                .await
                .assert_status(axum::http::StatusCode::NO_CONTENT);
        }
    }

    async fn full_compaction_apply_final_all_field_change(server: &TestServer) {
        server
            .put(&format!(
                "/collections/{FULL_COMPACTION_COLLECTION}/docs:replace"
            ))
            .json(&json!({ "docs": [{
                "external_id": FULL_COMPACTION_HOT_ID,
                "fields": {
                    FULL_KEYWORD: FULL_HOT_FINAL_KEYWORD,
                    FULL_NUMBER: FULL_HOT_FINAL_NUMBER,
                    FULL_HASH: FULL_HOT_FINAL_HASH,
                    FULL_TEXT: FULL_HOT_FINAL_TEXT,
                    FULL_FLAT: full_compaction_vector(9_000.0),
                    FULL_HNSW: full_compaction_vector(19_000.0),
                },
            }]}))
            .await
            .assert_status_ok();
        full_compaction_post_items(
            server,
            full_compaction_document_items(
                FULL_COMPACTION_APPENDED_ID,
                FULL_APPENDED_KEYWORD.to_owned(),
                FULL_APPENDED_NUMBER,
                FULL_APPENDED_TAG.to_owned(),
                FULL_APPENDED_HASH.to_owned(),
                FULL_APPENDED_TEXT.to_owned(),
                8_000.0,
                18_000.0,
            ),
        )
        .await;
    }

    fn full_compaction_collection<'a>(manifest: &'a Value) -> &'a Value {
        stage1_reuse_catalog_collection(manifest, FULL_COMPACTION_COLLECTION)
    }

    fn full_compaction_field_refs<'a>(
        manifest: &'a Value,
        field: &str,
        kind: &str,
    ) -> Vec<&'a Value> {
        let mut refs: Vec<_> = full_compaction_collection(manifest)["segments"]
            .as_array()
            .expect("full compaction catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!(kind)
            })
            .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("field ordinal"));
        refs
    }

    fn full_compaction_base_ref<'a>(manifest: &'a Value, field: &str, role: &str) -> &'a Value {
        full_compaction_collection(manifest)["segments"]
            .as_array()
            .expect("full compaction catalog segments")
            .iter()
            .find(|segment| {
                segment["role"] == json!(role)
                    && segment["field"] == json!(field)
                    && segment["kind"] == json!("base")
                    && segment["ordinal"] == json!(0)
            })
            .unwrap_or_else(|| panic!("full compaction catalog needs {role} base for {field}"))
    }

    fn full_compaction_delta_bytes(generation: &Path, manifest: &Value, field: &str) -> u64 {
        full_compaction_field_refs(manifest, field, "delta")
            .into_iter()
            .map(|segment| {
                let payload = std::fs::metadata(
                    generation.join(segment["path"].as_str().expect("delta segment path")),
                )
                .expect("inspect delta segment")
                .len();
                let local = segment["local_rows"]
                    .as_object()
                    .expect("delta local rows object");
                let rows = std::fs::metadata(
                    generation.join(local["path"].as_str().expect("delta local row path")),
                )
                .expect("inspect delta row map")
                .len();
                payload.checked_add(rows).expect("delta byte sum")
            })
            .sum()
    }

    fn full_compaction_base_bytes(generation: &Path, manifest: &Value, field: &str) -> u64 {
        std::fs::metadata(
            generation.join(
                full_compaction_base_ref(manifest, field, "field")["path"]
                    .as_str()
                    .expect("base segment path"),
            ),
        )
        .expect("inspect base segment")
        .len()
    }

    fn full_compaction_assert_hardlinked_base(
        old_generation: &Path,
        old_manifest: &Value,
        new_generation: &Path,
        new_manifest: &Value,
        field: &str,
        role: &str,
        context: &str,
    ) {
        let old_path = old_generation.join(
            full_compaction_base_ref(old_manifest, field, role)["path"]
                .as_str()
                .expect("old base path"),
        );
        let new_path = new_generation.join(
            full_compaction_base_ref(new_manifest, field, role)["path"]
                .as_str()
                .expect("new base path"),
        );
        let old_metadata = std::fs::symlink_metadata(&old_path).expect("inspect old base");
        let new_metadata = std::fs::symlink_metadata(&new_path).expect("inspect new base");
        assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
        assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
        assert_eq!(
            old_metadata.ino(),
            new_metadata.ino(),
            "{context}: unchanged {field}/{role} base must be a hard link, not a copy"
        );
        assert!(
            new_metadata.nlink() >= 2,
            "{context}: unchanged {field}/{role} base must retain more than one link"
        );
    }

    fn full_compaction_assert_rewritten_base(
        old_generation: &Path,
        old_manifest: &Value,
        new_generation: &Path,
        new_manifest: &Value,
    ) {
        let old_path = old_generation.join(
            full_compaction_base_ref(old_manifest, FULL_KEYWORD, "field")["path"]
                .as_str()
                .expect("old Keyword base path"),
        );
        let new_path = new_generation.join(
            full_compaction_base_ref(new_manifest, FULL_KEYWORD, "field")["path"]
                .as_str()
                .expect("rewritten Keyword base path"),
        );
        let old_metadata = std::fs::symlink_metadata(&old_path).expect("inspect old Keyword base");
        let new_metadata = std::fs::symlink_metadata(&new_path).expect("inspect new Keyword base");
        assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
        assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
        assert_ne!(
        old_metadata.ino(),
        new_metadata.ino(),
        "base-eligible Keyword compaction must publish a new base, not relink its obsolete base"
    );
    }

    async fn full_compaction_search_ids(server: &TestServer, query: Value) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({ "query": query, "limit": 512, "track_total": true }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let mut ids: Vec<_> = body["hits"]
            .as_array()
            .expect("bounded scalar search hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("search hit ID")
                    .to_owned()
            })
            .collect();
        ids.sort();
        assert_eq!(
            body["total"].as_u64(),
            Some(ids.len() as u64),
            "bounded scalar fixture reports every matching ID: {body}"
        );
        ids
    }

    fn full_compaction_keyword_query(value: &str) -> Value {
        json!({ "term": { "field": FULL_KEYWORD, "value": value } })
    }

    fn full_compaction_number_query(value: f64) -> Value {
        json!({ "range": {
            "field": FULL_NUMBER,
            "gte": value,
            "lte": value,
        }})
    }

    fn full_compaction_set_query(value: &str) -> Value {
        json!({ "term": { "field": FULL_SET, "value": value } })
    }

    fn full_compaction_hash_query(value: &str) -> Value {
        json!({ "hamming": {
            "field": FULL_HASH,
            "hash": value,
            "max_distance": 0,
        }})
    }

    async fn full_compaction_assert_scalar(
        server: &TestServer,
        reference: &TestServer,
        query: Value,
        mut expected: Vec<String>,
        context: &str,
    ) {
        expected.sort();
        let reference_ids = full_compaction_search_ids(reference, query.clone()).await;
        assert_eq!(
            reference_ids, expected,
            "independent reference has expected IDs for {context}"
        );
        assert_eq!(
            full_compaction_search_ids(server, query).await,
            reference_ids,
            "durable compaction state matches independent public API state for {context}"
        );
    }

    async fn full_compaction_text_search(server: &TestServer, text: &str) -> Value {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({
                "query": { "match": { "field": FULL_TEXT, "text": text, "op": "and" } },
                "limit": 512,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        let hits = body["hits"].as_array().expect("Text Match hits");
        assert_eq!(
            body["total"].as_u64(),
            Some(hits.len() as u64),
            "bounded Text fixture returns all Match hits: {body}"
        );
        assert!(
            hits.iter().all(|hit| {
                hit["external_id"].as_str().is_some() && hit["score"].as_f64().is_some()
            }),
            "Text Match exposes an ID and serialized BM25 score for every hit: {body}"
        );
        body
    }

    async fn full_compaction_assert_text(
        server: &TestServer,
        reference: &TestServer,
        text: &str,
        mut expected: Vec<String>,
        context: &str,
    ) {
        let reference_body = full_compaction_text_search(reference, text).await;
        let mut reference_ids: Vec<_> = reference_body["hits"]
            .as_array()
            .expect("reference Text hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("reference Text ID")
                    .to_owned()
            })
            .collect();
        reference_ids.sort();
        expected.sort();
        assert_eq!(
            reference_ids, expected,
            "independent Text reference has expected IDs for {context}"
        );
        let durable_body = full_compaction_text_search(server, text).await;
        assert_eq!(
            durable_body["total"], reference_body["total"],
            "durable Text compaction preserves total for {context}"
        );
        assert_eq!(
            durable_body["hits"], reference_body["hits"],
            "durable Text compaction preserves ordered IDs and BM25 scores for {context}"
        );
    }

    fn full_compaction_ids(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn full_compaction_current_common_ids() -> Vec<String> {
        (0..FULL_COMPACTION_BASE_ROWS)
            .map(full_compaction_base_id)
            .chain(std::iter::once(FULL_COMPACTION_HOT_ID.to_owned()))
            .chain(std::iter::once(FULL_COMPACTION_APPENDED_ID.to_owned()))
            .collect()
    }

    async fn full_compaction_vector_ids(server: &TestServer, field: &str, x: f32) -> Vec<String> {
        let response = server
            .post(&format!("/collections/{FULL_COMPACTION_COLLECTION}/search"))
            .json(&json!({
                "query": { "knn": { "field": field, "vector": full_compaction_vector(x), "k": 1 } },
                "limit": 1,
                "track_total": true,
            }))
            .await;
        response.assert_status_ok();
        let body: Value = response.json();
        body["hits"]
            .as_array()
            .expect("vector kNN hits")
            .iter()
            .map(|hit| {
                hit["external_id"]
                    .as_str()
                    .expect("vector hit ID")
                    .to_owned()
            })
            .collect()
    }

    async fn full_compaction_assert_vector(
        server: &TestServer,
        reference: &TestServer,
        field: &str,
        x: f32,
        expected: &str,
        context: &str,
    ) {
        let reference_ids = full_compaction_vector_ids(reference, field, x).await;
        assert_eq!(
            reference_ids,
            vec![expected.to_owned()],
            "independent {field} reference has expected nearest ID for {context}"
        );
        assert_eq!(
        full_compaction_vector_ids(server, field, x).await,
        reference_ids,
        "durable {field} compaction preserves nearest ID for {context} without claiming a backend score"
    );
    }

    async fn full_compaction_assert_current_state(
        server: &TestServer,
        reference: &TestServer,
        phase: &str,
    ) {
        let untouched_index = FULL_COMPACTION_BASE_ROWS - 1;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
            vec![full_compaction_base_id(0)],
            &format!("{phase} compacted Keyword update"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_round_keyword(1, 0)),
            vec![],
            &format!("{phase} masks an older Keyword delta"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_FINAL_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Keyword replacement"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_BASE_KEYWORD),
            vec![],
            &format!("{phase} masks old Keyword field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_DELETED_KEYWORD),
            vec![],
            &format!("{phase} keeps document tombstone"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_APPENDED_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_APPENDED_ID]),
            &format!("{phase} appended Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(&full_compaction_base_keyword(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_FINAL_NUMBER),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_BASE_NUMBER),
            vec![],
            &format!("{phase} masks old Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(full_compaction_base_number(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_HOT_BASE_TAG),
            vec![],
            &format!("{phase} omits old Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_APPENDED_TAG),
            full_compaction_ids(&[FULL_COMPACTION_APPENDED_ID]),
            &format!("{phase} appended Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(&full_compaction_base_tag(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Set field"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_FINAL_HASH),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Hash"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_BASE_HASH),
            vec![],
            &format!("{phase} masks old Hash"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(&full_compaction_base_hash(untouched_index)),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Hash"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_FINAL_TEXT,
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} final Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_BASE_TEXT,
            vec![],
            &format!("{phase} masks old Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            &format!("full-base-text-{untouched_index:03}"),
            full_compaction_ids(&[FULL_COMPACTION_UNTOUCHED_ID]),
            &format!("{phase} untouched Text"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            "full-common",
            full_compaction_current_common_ids(),
            &format!("{phase} cannot use fixed expected common corpus IDs"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            9_000.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} Flat final vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            19_000.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} HNSW final vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            8_000.0,
            FULL_COMPACTION_APPENDED_ID,
            &format!("{phase} Flat appended vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            18_000.0,
            FULL_COMPACTION_APPENDED_ID,
            &format!("{phase} HNSW appended vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            full_compaction_flat_x(untouched_index),
            FULL_COMPACTION_UNTOUCHED_ID,
            &format!("{phase} untouched Flat vector"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            full_compaction_hnsw_x(untouched_index),
            FULL_COMPACTION_UNTOUCHED_ID,
            &format!("{phase} untouched HNSW vector"),
        )
        .await;
    }

    async fn full_compaction_assert_retained_base(
        server: &TestServer,
        reference: &TestServer,
        phase: &str,
    ) {
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_keyword_query(FULL_HOT_BASE_KEYWORD),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Keyword"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_number_query(FULL_HOT_BASE_NUMBER),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Number"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_set_query(FULL_HOT_BASE_TAG),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Set"),
        )
        .await;
        full_compaction_assert_scalar(
            server,
            reference,
            full_compaction_hash_query(FULL_HOT_BASE_HASH),
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Hash"),
        )
        .await;
        full_compaction_assert_text(
            server,
            reference,
            FULL_HOT_BASE_TEXT,
            full_compaction_ids(&[FULL_COMPACTION_HOT_ID]),
            &format!("{phase} Text"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_FLAT,
            1.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} Flat"),
        )
        .await;
        full_compaction_assert_vector(
            server,
            reference,
            FULL_HNSW,
            2.0,
            FULL_COMPACTION_HOT_ID,
            &format!("{phase} HNSW"),
        )
        .await;
    }

    async fn full_compaction_assert_third_retained(server: &TestServer) {
        assert_eq!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_round_keyword(3, 0)),
            )
            .await,
            vec![full_compaction_base_id(0)],
            "retained pre-merge generation keeps its third Keyword update"
        );
        assert!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_round_keyword(1, 0)),
            )
            .await
            .is_empty(),
            "retained pre-merge generation masks its older Keyword layer"
        );
        assert!(
            full_compaction_search_ids(server, full_compaction_keyword_query(FULL_DELETED_KEYWORD))
                .await
                .is_empty(),
            "retained pre-merge generation keeps its tombstone"
        );
    }

    #[tokio::test]
    async fn v2_base_eligible_keyword_compaction_keeps_all_field_results_live_cold_and_retained() {
        let fixture = full_compaction_fixture().await;
        let baseline_reference = full_compaction_reference().await;
        let reference = full_compaction_reference().await;
        let base_generation = fixture.root.join(&fixture.base_name);
        let base_manifest = stage1_read_manifest(&base_generation);

        for round in 1..=3 {
            full_compaction_apply_keyword_round(&fixture.server, round).await;
            full_compaction_apply_keyword_round(&reference, round).await;
            fixture
                .store
                .save(
                    &fixture.engine,
                    FULL_COMPACTION_BASE_SEQUENCE + round as u64,
                )
                .expect("publish Keyword delta before base eligibility");
        }
        let third_name = stage1_reuse_current_name(&fixture.root);
        let third_generation = fixture.root.join(&third_name);
        let third_manifest = stage1_read_manifest(&third_generation);
        let third_keyword_deltas =
            full_compaction_field_refs(&third_manifest, FULL_KEYWORD, "delta");
        assert_eq!(
            third_keyword_deltas.len(),
            3,
            "the fourth save alone must request compaction for the uniquely deepest Keyword field"
        );
        stage1_assert_delta_sequence_order(
            &third_keyword_deltas,
            &[
                FULL_COMPACTION_BASE_SEQUENCE + 1,
                FULL_COMPACTION_BASE_SEQUENCE + 2,
                FULL_COMPACTION_BASE_SEQUENCE + 3,
            ],
            "full base-eligible Keyword pre-merge layers",
        );
        let base_bytes = full_compaction_base_bytes(&base_generation, &base_manifest, FULL_KEYWORD);
        let third_delta_bytes =
            full_compaction_delta_bytes(&third_generation, &third_manifest, FULL_KEYWORD);
        assert!(
        third_delta_bytes >= base_bytes,
        "fixture precondition: three actual Keyword delta-plus-rowmap bytes ({third_delta_bytes}) must meet or exceed the actual base bytes ({base_bytes}) before the fourth save requests base compaction"
    );

        full_compaction_apply_keyword_round(&fixture.server, 4).await;
        full_compaction_apply_keyword_round(&reference, 4).await;
        fixture
            .store
            .save(&fixture.engine, FULL_COMPACTION_BASE_SEQUENCE + 4)
            .expect("publish base-eligible Keyword checkpoint");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for base-eligible Keyword compaction");
        let compacted_name = stage1_reuse_current_name(&fixture.root);
        let compacted_generation = fixture.root.join(&compacted_name);
        let compacted_manifest = stage1_read_manifest(&compacted_generation);
        assert!(
        full_compaction_field_refs(&compacted_manifest, FULL_KEYWORD, "delta").is_empty(),
        "when actual Keyword delta bytes meet the base before the fourth request, publication must replace the base and consume every captured Keyword delta"
    );
        full_compaction_assert_rewritten_base(
            &base_generation,
            &base_manifest,
            &compacted_generation,
            &compacted_manifest,
        );
        for field in [
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            assert_eq!(
            full_compaction_field_refs(&compacted_manifest, field, "delta").len(),
            1,
            "only the Keyword field has four deltas, so {field} keeps its one sparse tombstone layer"
        );
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
                field,
                "field",
                "base-eligible Keyword compaction",
            );
        }
        for field in [FULL_FLAT, FULL_HNSW] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
                field,
                "vector_eids",
                "base-eligible Keyword compaction",
            );
        }

        full_compaction_apply_final_all_field_change(&fixture.server).await;
        full_compaction_apply_final_all_field_change(&reference).await;
        fixture
            .store
            .save(&fixture.engine, FULL_COMPACTION_BASE_SEQUENCE + 5)
            .expect("publish all-field checkpoint after Keyword base compaction");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for post-compaction all-field merges");
        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        for field in [
            FULL_KEYWORD,
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            assert!(
                full_compaction_field_refs(&latest_manifest, field, "delta").len() <= 16,
                "published {field} catalog must stay at or below the sixteen-delta hard limit"
            );
        }
        full_compaction_assert_hardlinked_base(
            &compacted_generation,
            &compacted_manifest,
            &latest_generation,
            &latest_manifest,
            FULL_KEYWORD,
            "field",
            "post-compaction all-field checkpoint",
        );
        for field in [
            FULL_NUMBER,
            FULL_SET,
            FULL_HASH,
            FULL_TEXT,
            FULL_FLAT,
            FULL_HNSW,
        ] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &latest_generation,
                &latest_manifest,
                field,
                "field",
                "post-compaction all-field checkpoint",
            );
        }
        for field in [FULL_FLAT, FULL_HNSW] {
            full_compaction_assert_hardlinked_base(
                &base_generation,
                &base_manifest,
                &latest_generation,
                &latest_manifest,
                field,
                "vector_eids",
                "post-compaction all-field checkpoint",
            );
        }
        full_compaction_assert_current_state(
            &fixture.server,
            &reference,
            "live current generation",
        )
        .await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FULL_COMPACTION_BASE_SEQUENCE + 5,
            "cold CURRENT uses the all-field checkpoint watermark"
        );
        let cold_server = TestServer::new(router(AppState::open(cold_engine)))
            .expect("cold full compaction server");
        full_compaction_assert_current_state(&cold_server, &reference, "cold current generation")
            .await;

        let (third_engine, third_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &third_name);
        assert_eq!(
            third_sequence,
            FULL_COMPACTION_BASE_SEQUENCE + 3,
            "retained pre-merge generation preserves its original watermark"
        );
        let third_server =
            TestServer::new(router(AppState::open(third_engine))).expect("retained third server");
        full_compaction_assert_third_retained(&third_server).await;

        let (base_engine, base_sequence) =
            stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
        assert_eq!(
            base_sequence, FULL_COMPACTION_BASE_SEQUENCE,
            "retained base generation preserves its original watermark"
        );
        let base_server =
            TestServer::new(router(AppState::open(base_engine))).expect("retained base server");
        full_compaction_assert_retained_base(
            &base_server,
            &baseline_reference,
            "retained base generation",
        )
        .await;
    }

    const FULL_PAIR_WITNESS_COLLECTION: &str = "full-partial-pair-witness";
    const FULL_PAIR_BASE_SEQUENCE: u64 = 9_600;

    #[derive(Clone, Debug)]
    struct FullPairDelta {
        ordinal: u64,
        path: String,
        bytes: u64,
        ids: std::collections::BTreeSet<String>,
        inode: u64,
    }

    fn full_pair_value(round: usize) -> String {
        // The values deliberately vary in physical size. The selection oracle
        // nevertheless derives the legal pair from actual written file sizes.
        let words = match round {
            1 => 2,
            2 => 8,
            3 => 32,
            4 => 4,
            _ => panic!("pair policy needs one of four rounds"),
        };
        full_compaction_entropy_term(700_000 + round as u64, words)
    }

    fn full_pair_collection<'a>(manifest: &'a Value, collection: &str) -> &'a Value {
        manifest["collections"]
            .as_array()
            .expect("pair policy catalog collections")
            .iter()
            .find(|candidate| candidate["collection_id"] == json!(collection))
            .unwrap_or_else(|| panic!("pair policy catalog needs {collection}"))
    }

    fn full_pair_refs<'a>(manifest: &'a Value, collection: &str, kind: &str) -> Vec<&'a Value> {
        let mut refs: Vec<_> = full_pair_collection(manifest, collection)["segments"]
            .as_array()
            .expect("pair policy catalog segments")
            .iter()
            .filter(|segment| {
                segment["role"] == json!("field")
                    && segment["field"] == json!(FULL_KEYWORD)
                    && segment["kind"] == json!(kind)
            })
            .collect();
        refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("pair delta ordinal"));
        refs
    }

    fn full_pair_ref_bytes(generation: &Path, reference: &Value) -> u64 {
        let payload = std::fs::metadata(
            generation.join(
                reference["path"]
                    .as_str()
                    .expect("pair segment reference path"),
            ),
        )
        .expect("inspect pair segment")
        .len();
        let rows = reference["local_rows"]
            .as_object()
            .expect("pair local row reference");
        let rows =
            std::fs::metadata(generation.join(rows["path"].as_str().expect("pair local row path")))
                .expect("inspect pair row map")
                .len();
        payload.checked_add(rows).expect("pair reference byte sum")
    }

    fn full_pair_snapshot(generation: &Path, references: Vec<&Value>) -> Vec<FullPairDelta> {
        references
            .into_iter()
            .map(|reference| {
                let path = reference["path"]
                    .as_str()
                    .expect("pair segment path")
                    .to_owned();
                let ids = stage1_keyword_delta_read_rows(generation, reference)
                    .into_iter()
                    .collect();
                let inode = std::fs::symlink_metadata(generation.join(&path))
                    .expect("inspect pair segment inode")
                    .ino();
                FullPairDelta {
                    ordinal: reference["ordinal"].as_u64().expect("pair segment ordinal"),
                    bytes: full_pair_ref_bytes(generation, reference),
                    path,
                    ids,
                    inode,
                }
            })
            .collect()
    }

    async fn full_pair_add_witness_collection(fixture: &FullCompactionFixture) -> String {
        fixture
            .server
            .put(&format!("/collections/{FULL_PAIR_WITNESS_COLLECTION}"))
            .json(&json!({ "fields": { FULL_KEYWORD: { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        fixture
            .server
            .post(&format!(
                "/collections/{FULL_PAIR_WITNESS_COLLECTION}/index"
            ))
            .json(&json!({ "items": [{
                "external_id": full_compaction_base_id(3),
                "field": FULL_KEYWORD,
                "value": "pair-witness-base",
            }] }))
            .await
            .assert_status_ok();
        fixture
            .store
            .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE)
            .expect("publish pair-policy base generation");
        stage1_reuse_current_name(&fixture.root)
    }

    async fn full_pair_update_target(server: &TestServer, round: usize) {
        full_compaction_post_items(
            server,
            vec![json!({
                "external_id": full_compaction_base_id(round - 1),
                "field": FULL_KEYWORD,
                "value": full_pair_value(round),
            })],
        )
        .await;
    }

    async fn full_pair_update_witness(server: &TestServer) {
        server
            .post(&format!(
                "/collections/{FULL_PAIR_WITNESS_COLLECTION}/index"
            ))
            .json(&json!({ "items": [{
                "external_id": full_compaction_base_id(3),
                "field": FULL_KEYWORD,
                "value": full_pair_value(4),
            }] }))
            .await
            .assert_status_ok();
    }

    async fn full_pair_assert_query_state(server: &TestServer, phase: &str) {
        for round in 1..=4 {
            assert_eq!(
                full_compaction_search_ids(
                    server,
                    full_compaction_keyword_query(&full_pair_value(round))
                )
                .await,
                vec![full_compaction_base_id(round - 1)],
                "{phase}: compacted pair keeps round-{round} Keyword value"
            );
        }
        assert_eq!(
            full_compaction_search_ids(
                server,
                full_compaction_keyword_query(&full_compaction_base_keyword(
                    FULL_COMPACTION_BASE_ROWS - 1
                )),
            )
            .await,
            vec![FULL_COMPACTION_UNTOUCHED_ID.to_owned()],
            "{phase}: compacted pair keeps untouched base Keyword value"
        );
    }

    #[tokio::test]
    async fn v2_partial_keyword_compaction_reduces_the_measured_delta_stack_below_base() {
        let fixture = full_compaction_fixture().await;
        let pair_base_name = full_pair_add_witness_collection(&fixture).await;
        let pair_base_generation = fixture.root.join(&pair_base_name);
        let pair_base_manifest = stage1_read_manifest(&pair_base_generation);
        let base_bytes =
            full_compaction_base_bytes(&pair_base_generation, &pair_base_manifest, FULL_KEYWORD);

        for round in 1..=3 {
            full_pair_update_target(&fixture.server, round).await;
            fixture
                .store
                .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE + round as u64)
                .expect("publish pair-policy target delta");
        }
        let before_generation = stage1_current_generation_dir(&fixture.root);
        let before_manifest = stage1_read_manifest(&before_generation);
        let before_references =
            full_pair_refs(&before_manifest, FULL_COMPACTION_COLLECTION, "delta");
        assert_eq!(
            before_references.len(),
            3,
            "the fourth pair-policy save alone must request compaction"
        );
        stage1_assert_delta_sequence_order(
            &before_references,
            &[
                FULL_PAIR_BASE_SEQUENCE + 1,
                FULL_PAIR_BASE_SEQUENCE + 2,
                FULL_PAIR_BASE_SEQUENCE + 3,
            ],
            "pre-merge Keyword delta layers",
        );
        let before = full_pair_snapshot(&before_generation, before_references);
        assert_eq!(
            before.len(),
            3,
            "the fourth pair-policy save alone must request compaction"
        );
        assert!(
            before.iter().all(|layer| layer.ids.len() == 1),
            "each measured pre-compaction policy layer must name one distinct ID"
        );
        let distinct_before_ids: std::collections::BTreeSet<_> = before
            .iter()
            .flat_map(|layer| layer.ids.iter().cloned())
            .collect();
        assert_eq!(
            distinct_before_ids.len(),
            before.len(),
            "the measured pre-compaction policy layers must name distinct IDs"
        );

        full_pair_update_target(&fixture.server, 4).await;
        full_pair_update_witness(&fixture.server).await;
        fixture
            .store
            .save(&fixture.engine, FULL_PAIR_BASE_SEQUENCE + 4)
            .expect("publish fourth pair-policy target delta");
        fixture
            .store
            .wait_for_merges(Duration::from_secs(30))
            .expect("wait for adjacent-pair delta compaction");
        let latest_generation = stage1_current_generation_dir(&fixture.root);
        let latest_manifest = stage1_read_manifest(&latest_generation);
        let witness_references =
            full_pair_refs(&latest_manifest, FULL_PAIR_WITNESS_COLLECTION, "delta");
        assert_eq!(
            witness_references.len(),
            2,
            "a fresh witness retains its initial sparse layer and the same-checkpoint raw delta"
        );
        stage1_assert_delta_sequence_order(
            &witness_references,
            &[FULL_PAIR_BASE_SEQUENCE, FULL_PAIR_BASE_SEQUENCE + 4],
            "fresh witness layers before the fourth-round whole-stack merge",
        );
        let fourth_witness_reference = witness_references
            .iter()
            .copied()
            .find(|reference| reference["applied_seq"] == json!(FULL_PAIR_BASE_SEQUENCE + 4))
            .expect("same-checkpoint witness delta");
        let witness = full_pair_snapshot(&latest_generation, vec![fourth_witness_reference]);
        assert_eq!(
            witness.len(),
            1,
            "selected same-checkpoint witness delta must have one measured layer"
        );
        assert_eq!(
            witness[0].ids,
            std::collections::BTreeSet::from([full_compaction_base_id(3)]),
            "same-checkpoint witness uses the target fourth ID and local-row shape"
        );
        let before_plus_fourth = before
            .iter()
            .map(|layer| layer.bytes)
            .sum::<u64>()
            .checked_add(witness[0].bytes)
            .expect("pair-policy total bytes");
        assert!(
            before_plus_fourth < base_bytes,
            "fixture precondition: the three measured target layers plus the same-sequence fourth witness ({before_plus_fourth}) stay below the measured base ({base_bytes}), so policy must fold the delta stack instead of merging the base"
        );

        let latest = full_pair_snapshot(
            &latest_generation,
            full_pair_refs(&latest_manifest, FULL_COMPACTION_COLLECTION, "delta"),
        );
        assert_eq!(
            latest.len(),
            3,
            "below the measured base threshold, one job must reduce the four-layer delta stack by one adjacent pair"
        );
        let compacted = latest
            .iter()
            .find(|layer| layer.ordinal == before.last().expect("latest input").ordinal + 1)
            .expect("the selected adjacent pair must publish at the newest ordinal");
        let fourth_ordinal = before
            .last()
            .expect("three pre-compaction target layers")
            .ordinal
            .checked_add(1)
            .expect("fourth delta ordinal");
        assert_eq!(
            compacted.ordinal,
            fourth_ordinal,
            "the compacted output must carry the newest (fourth) input's ordinal, matching write_compacted_field's non-base output path"
        );
        let all_input_ids: std::collections::BTreeSet<String> = before
            .iter()
            .flat_map(|layer| layer.ids.iter().cloned())
            .collect();
        let mut all_input_ids = all_input_ids;
        all_input_ids.insert(full_compaction_base_id(3));
        assert!(
            compacted.ids.is_subset(&all_input_ids),
            "the compacted adjacent pair must contain only measured input IDs"
        );

        for old in &before {
            let retained_metadata = std::fs::symlink_metadata(before_generation.join(&old.path))
                .unwrap_or_else(|error| {
                    panic!(
                        "superseded input at ordinal {} must still be present, unpruned, in its retained pre-compaction generation: {error}",
                        old.ordinal
                    )
                });
            assert_eq!(
                retained_metadata.ino(),
                old.inode,
                "the retained pre-compaction generation must keep the original, unmutated file for ordinal {}",
                old.ordinal
            );
        }
        full_compaction_assert_hardlinked_base(
            &pair_base_generation,
            &pair_base_manifest,
            &latest_generation,
            &latest_manifest,
            FULL_KEYWORD,
            "field",
            "whole-stack delta compaction",
        );
        full_pair_assert_query_state(&fixture.server, "live whole-stack compaction generation")
            .await;

        let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
        assert_eq!(
            cold_sequence,
            FULL_PAIR_BASE_SEQUENCE + 4,
            "cold whole-stack compaction generation preserves the fourth watermark"
        );
        let cold_server =
            TestServer::new(router(AppState::open(cold_engine))).expect("cold pair policy server");
        full_pair_assert_query_state(&cold_server, "cold whole-stack compaction generation").await;
    }

    mod paused_merge_checkpoint_contract {
        //! # Facets
        //!
        //! - Behavior: `indexing_durable_oracle.rs:7713` requires the other checkpoint
        //!   while encoding is paused; `:7728` and `:7771` retain the fifth Keyword layer.
        //! - Security: `apps/lumen/src/segment_rdb.rs:66-73` is a test-only observer seam.
        //!   `indexing_durable_oracle.rs:7740` and `:7760` require CURRENT and cold reopen
        //!   to retain the later collection reference, without a new input boundary.
        //! - Performance: `indexing_durable_oracle.rs:7713` measures the approved bounded
        //!   checkpoint-progress path with two seconds. It does not claim stage6 latency,
        //!   RSS, or throughput acceptance.

        use super::*;

        const OTHER_COLLECTION: &str = "merge-pause-other";
        const OTHER_FIELD: &str = "kw";
        const OTHER_ID: &str = "merge-pause-other-id";
        const OTHER_BASE_VALUE: &str = "merge-pause-other-base";
        const OTHER_NEW_VALUE: &str = "merge-pause-other-new";

        struct PauseBeforeEncode {
            reached: std::sync::mpsc::SyncSender<()>,
            published: std::sync::mpsc::SyncSender<()>,
            release: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
            paused: std::sync::atomic::AtomicBool,
        }

        impl lumen::segment_rdb::MergeObserver for PauseBeforeEncode {
            fn observe(&self, phase: lumen::segment_rdb::MergePhase) -> std::io::Result<()> {
                if phase == lumen::segment_rdb::MergePhase::AfterPublish {
                    self.published.send(()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "merge publication observer receiver dropped",
                        )
                    })?;
                }
                if phase == lumen::segment_rdb::MergePhase::BeforeEncode
                    && !self.paused.swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    self.reached.send(()).map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "merge pause observer receiver dropped",
                        )
                    })?;
                    self.release
                        .lock()
                        .expect("merge pause release mutex")
                        .take()
                        .expect("merge pause releases exactly once")
                        .recv()
                        .map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                "merge pause release sender dropped",
                            )
                        })?;
                }
                Ok(())
            }
        }

        async fn other_term_ids(server: &TestServer, value: &str) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{OTHER_COLLECTION}/search"))
                .json(&json!({
                    "query": { "term": { "field": OTHER_FIELD, "value": value } },
                    "limit": 16,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let mut ids = response.json::<Value>()["hits"]
                .as_array()
                .expect("other collection term hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("other collection external ID")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            ids.sort();
            ids
        }

        async fn create_other_collection(server: &TestServer) {
            server
                .put(&format!("/collections/{OTHER_COLLECTION}"))
                .json(&json!({ "fields": { OTHER_FIELD: { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            server
                .post(&format!("/collections/{OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": OTHER_ID,
                    "field": OTHER_FIELD,
                    "value": OTHER_BASE_VALUE,
                }] }))
                .await
                .assert_status_ok();
        }

        async fn update_other_collection(server: &TestServer) {
            server
                .post(&format!("/collections/{OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": OTHER_ID,
                    "field": OTHER_FIELD,
                    "value": OTHER_NEW_VALUE,
                }] }))
                .await
                .assert_status_ok();
        }

        async fn wait_for_background_idle(store: Arc<SegmentRdbStore>) -> Result<()> {
            tokio::task::spawn_blocking(move || store.wait_for_merges(Duration::from_secs(30)))
                .await
                .map_err(|error| anyhow::anyhow!("background wait task panicked: {error}"))?
        }

        async fn paused_merge_apply_keyword_round(server: &TestServer, round: usize) {
            // Keep this fixture below the base-compaction threshold. It tests
            // successor scheduling after a pair merge, so four small delta
            // layers must remain a delta stack instead of replacing the base.
            full_compaction_post_items(
                server,
                vec![json!({
                    "external_id": full_compaction_base_id(0),
                    "field": FULL_KEYWORD,
                    "value": full_compaction_round_keyword(round, 0),
                })],
            )
            .await;
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn ordinary_checkpoint_does_not_request_successor_until_next_checkpoint() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (published_tx, published_rx) = std::sync::mpsc::sync_channel(4);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let observer = Arc::new(PauseBeforeEncode {
                reached: reached_tx,
                published: published_tx,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open observed full-compaction store"),
            );
            let baseline_sequence = FULL_COMPACTION_BASE_SEQUENCE + 1;
            create_other_collection(&fixture.server).await;
            store
                .save(&fixture.engine, baseline_sequence)
                .expect("publish other collection baseline");

            for round in 1..=3 {
                paused_merge_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, baseline_sequence + round as u64)
                    .expect("publish measured pre-merge delta");
            }
            paused_merge_apply_keyword_round(&fixture.server, 4).await;

            let merge_sequence = baseline_sequence + 4;
            let paused_merge = {
                let store = store.clone();
                let engine = fixture.engine.clone();
                tokio::task::spawn_blocking(move || store.save(&engine, merge_sequence))
            };
            let reached = tokio::task::spawn_blocking(move || {
                reached_rx.recv_timeout(Duration::from_secs(30))
            })
            .await
            .expect("merge-pause receiver task must not panic");

            let mut later_checkpoint = if reached.is_ok() {
                paused_merge_apply_keyword_round(&fixture.server, 5).await;
                update_other_collection(&fixture.server).await;
                let store = store.clone();
                let engine = fixture.engine.clone();
                Some(tokio::task::spawn_blocking(move || {
                    store.save(&engine, merge_sequence + 1)
                }))
            } else {
                None
            };
            let completed_while_merge_paused = match later_checkpoint.as_mut() {
                Some(save) => Some(tokio::time::timeout(Duration::from_secs(2), save).await),
                None => None,
            };
            let checkpoint_finished_while_paused =
                matches!(&completed_while_merge_paused, Some(Ok(Ok(Ok(())))));

            // Send before every join. Thus an assertion below cannot leave the
            // real encoding callback or its checkpoint thread blocked.
            let _ = release_tx.send(());
            let merge_result = paused_merge.await;
            let later_finished_after_release = match &completed_while_merge_paused {
                Some(Ok(result)) => {
                    drop(later_checkpoint.take());
                    Some(matches!(result, Ok(Ok(()))))
                }
                Some(Err(_)) => Some(
                    later_checkpoint
                        .take()
                        .expect("timed-out checkpoint handle")
                        .await
                        .is_ok_and(|result| result.is_ok()),
                ),
                None => None,
            };

            assert!(
                reached.is_ok(),
                "a fourth real delta checkpoint must select a merge candidate before encoding: {reached:?}"
            );
            assert!(
                matches!(&merge_result, Ok(Ok(()))),
                "paused merge must finish cleanly after release: {merge_result:?}"
            );
            assert!(
                checkpoint_finished_while_paused && later_finished_after_release == Some(true),
                "a real checkpoint for another collection must finish while merge encoding is paused: {completed_while_merge_paused:?}; after release joined successfully: {later_finished_after_release:?}"
            );

            let first_idle = wait_for_background_idle(store.clone()).await;
            let first_publications = published_rx.try_iter().count();
            assert!(
                first_idle.is_ok(),
                "the released ordinary merge must become idle: {first_idle:?}"
            );
            assert_eq!(
                first_publications,
                1,
                "an ordinary checkpoint completed while the merge was running must not queue a successor"
            );

            assert_eq!(
                other_term_ids(&fixture.server, OTHER_NEW_VALUE).await,
                vec![OTHER_ID.to_owned()],
                "live state must retain the checkpoint published during the paused merge"
            );
            assert_eq!(
                other_term_ids(&fixture.server, OTHER_BASE_VALUE).await,
                Vec::<String>::new(),
                "live state must not resurrect the other collection base value"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &fixture.server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "the later merge must retain the fifth Keyword layer appended during its pause"
            );

            let current_dir = stage1_current_generation_dir(&fixture.root);
            let current_manifest = stage1_read_manifest(&current_dir);
            assert_eq!(
                current_manifest["checkpoint_sequence"],
                json!(merge_sequence + 1),
                "CURRENT must retain the newer checkpoint watermark after the paused merge finishes"
            );
            assert!(
                current_manifest["collections"]
                    .as_array()
                    .expect("current collection catalog")
                    .iter()
                    .any(|collection| collection["collection_id"] == json!(OTHER_COLLECTION)),
                "CURRENT must retain the newer collection catalog reference"
            );

            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("reopen paused-merge root")
                .load_current_generation()
                .expect("load paused-merge CURRENT")
                .expect("paused-merge CURRENT generation");
            assert_eq!(
                cold.sequence,
                merge_sequence + 1,
                "cold reopen must select the newer checkpoint rather than the paused merge cut"
            );
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("cold paused-merge server");
            assert_eq!(
                other_term_ids(&cold_server, OTHER_NEW_VALUE).await,
                vec![OTHER_ID.to_owned()],
                "cold CURRENT must retain the other collection change published during merge pause"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold CURRENT must retain the fifth Keyword layer appended during merge pause"
            );

            paused_merge_apply_keyword_round(&fixture.server, 6).await;
            store
                .save(&fixture.engine, merge_sequence + 2)
                .expect("the next ordinary checkpoint must request a new merge");
            let second_idle = wait_for_background_idle(store.clone()).await;
            let second_publications = published_rx.try_iter().count();
            assert!(
                second_idle.is_ok(),
                "the next checkpoint's successor merge must become idle: {second_idle:?}"
            );
            assert_eq!(
                second_publications,
                1,
                "the next checkpoint must trigger exactly one new merge after the prior worker became idle"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &fixture.server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "live state must retain the new suffix after the successor merge"
            );
            let final_cold = SegmentRdbStore::new(&fixture.root)
                .expect("reopen successor-merge root")
                .load_current_generation()
                .expect("load successor-merge CURRENT")
                .expect("successor-merge CURRENT generation");
            assert_eq!(
                final_cold.sequence,
                merge_sequence + 2,
                "cold reopen must retain the successor checkpoint watermark"
            );
            let final_cold_server = TestServer::new(router(AppState::open(final_cold.engine)))
                .expect("cold successor-merge server");
            assert_eq!(
                other_term_ids(&final_cold_server, OTHER_NEW_VALUE).await,
                vec![OTHER_ID.to_owned()],
                "cold successor CURRENT must retain the ordinary checkpoint's other collection"
            );
            assert_eq!(
                full_compaction_search_ids(
                    &final_cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold successor CURRENT must retain the new suffix"
            );
        }
    }
    mod background_merge_cap_restore_prune_contract {
        //! # Facets
        //!
        //! - Behavior: `cap_sixteen_deltas_waits_without_publishing_seventeen_and_keeps_other_work_live`
        //!   requires a seventeenth dirty field checkpoint to remain uncommitted while its
        //!   selected four-layer merge is paused, then requires its final live and cold
        //!   results after release. `paused_merge_before_publish_cannot_overwrite_a_truncated_epoch`
        //!   requires an old selected merge to leave a newer truncate epoch and CURRENT
        //!   unchanged. `prune_and_second_opener_keep_paused_merge_source_and_staging_alive`
        //!   requires prune and a second public store opener to preserve the worker's source
        //!   and staging until the selected merge publishes.
        //! - Security: these cases exercise the process-owned generation root through
        //!   `apps/lumen/src/segment_background_merge.rs` and `apps/lumen/src/segment_rdb.rs`.
        //!   They do not add caller-controlled bytes, paths, or identifiers. Existing malformed
        //!   CURRENT refusal cases in the parent target keep the file-input boundary covered.
        //!   The prune case asserts that a second opener cannot mistake an active worker staging
        //!   directory for abandoned input and delete it.
        //! - Performance: the approved #4246 plan requires a merge request at four delta layers
        //!   and a hard maximum of sixteen. The hard-cap case carries those structural limits;
        //!   its two-second observations are bounded progress checks, not a throughput, latency,
        //!   or RSS acceptance claim. Cleanup waits tolerate the process-wide single merge worker.

        use super::*;
        use std::collections::BTreeSet;

        const HARD_CAP_SEQUENCE: u64 = 10_100;
        const HARD_CAP_OTHER_COLLECTION: &str = "background-cap-other";
        const HARD_CAP_OTHER_FIELD: &str = "kw";
        const HARD_CAP_OTHER_ID: &str = "background-cap-other-id";
        const HARD_CAP_OTHER_BASE: &str = "background-cap-other-base";
        const HARD_CAP_OTHER_CHECKPOINTED: &str = "background-cap-other-checkpointed";
        const HARD_CAP_OTHER_LIVE: &str = "background-cap-other-live";

        const STALE_EPOCH_SEQUENCE: u64 = 11_100;
        const PRUNE_SEQUENCE: u64 = 12_100;

        /// Pauses one real background merge outside the root save lock. Each test sends
        /// `release` before it inspects a result, so a failed assertion cannot strand
        /// the process-wide worker or a checkpoint task.
        struct PauseOneMergePhase {
            phase: lumen::segment_rdb::MergePhase,
            reached: std::sync::mpsc::SyncSender<()>,
            published: Option<std::sync::mpsc::Sender<()>>,
            release: Mutex<Option<std::sync::mpsc::Receiver<()>>>,
            paused: std::sync::atomic::AtomicBool,
        }

        /// Sends the observer release on every exit path, including an assertion
        /// panic in the test body. The worker must never remain paused for a later
        /// test in this process.
        struct MergeRelease {
            sender: Option<std::sync::mpsc::Sender<()>>,
        }

        impl MergeRelease {
            fn new(sender: std::sync::mpsc::Sender<()>) -> Self {
                Self {
                    sender: Some(sender),
                }
            }

            fn release(&mut self) {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(());
                }
            }
        }

        impl Drop for MergeRelease {
            fn drop(&mut self) {
                self.release();
            }
        }

        impl lumen::segment_rdb::MergeObserver for PauseOneMergePhase {
            fn observe(&self, phase: lumen::segment_rdb::MergePhase) -> std::io::Result<()> {
                if phase == lumen::segment_rdb::MergePhase::AfterPublish {
                    if let Some(published) = &self.published {
                        published.send(()).map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                "background merge publication receiver dropped",
                            )
                        })?;
                    }
                }
                if phase != self.phase
                    || self.paused.swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    return Ok(());
                }
                self.reached.send(()).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "background merge readiness receiver dropped",
                    )
                })?;
                self.release
                    .lock()
                    .expect("background merge release mutex")
                    .take()
                    .expect("background merge releases exactly once")
                    .recv()
                    .map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "background merge release sender dropped",
                        )
                    })?;
                Ok(())
            }
        }

        fn current_bytes(root: &Path) -> Vec<u8> {
            std::fs::read(root.join("CURRENT")).expect("read committed CURRENT bytes")
        }

        /// A merge scratch directory is a real root directory without the manifest that
        /// makes a generation publishable. This observes lifecycle state without baking
        /// the private staging filename into the contract.
        fn unpublished_generation_directories(root: &Path) -> BTreeSet<String> {
            std::fs::read_dir(root)
                .expect("read checkpoint root")
                .map(|entry| entry.expect("read checkpoint root entry"))
                .filter(|entry| {
                    entry.file_type().expect("inspect root entry type").is_dir()
                        && !entry.path().join("_generation.json").is_file()
                })
                .map(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .expect("checkpoint root entry is UTF-8")
                })
                .collect()
        }

        async fn create_hard_cap_other_collection(server: &TestServer) {
            server
                .put(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}"))
                .json(&json!({ "fields": { HARD_CAP_OTHER_FIELD: { "type": "keyword" } } }))
                .await
                .assert_status_ok();
            hard_cap_index_other(server, HARD_CAP_OTHER_BASE).await;
        }

        async fn hard_cap_index_other(server: &TestServer, value: &str) {
            server
                .post(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}/index"))
                .json(&json!({ "items": [{
                    "external_id": HARD_CAP_OTHER_ID,
                    "field": HARD_CAP_OTHER_FIELD,
                    "value": value,
                }] }))
                .await
                .assert_status_ok();
        }

        async fn hard_cap_other_ids(server: &TestServer, value: &str) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{HARD_CAP_OTHER_COLLECTION}/search"))
                .json(&json!({
                    "query": { "term": { "field": HARD_CAP_OTHER_FIELD, "value": value } },
                    "limit": 16,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let mut ids = response.json::<Value>()["hits"]
                .as_array()
                .expect("hard-cap other hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("hard-cap other external ID")
                        .to_owned()
                })
                .collect::<Vec<_>>();
            ids.sort();
            ids
        }

        async fn wait_for_worker_ready(receiver: std::sync::mpsc::Receiver<()>) -> Result<()> {
            tokio::task::spawn_blocking(move || receiver.recv_timeout(Duration::from_secs(30)))
                .await
                .map_err(|error| {
                    anyhow::anyhow!("merge readiness receiver task panicked: {error}")
                })?
                .map_err(|error| anyhow::anyhow!("merge readiness timed out: {error}"))
        }

        async fn wait_for_background_idle(store: Arc<SegmentRdbStore>) -> Result<()> {
            tokio::task::spawn_blocking(move || store.wait_for_merges(Duration::from_secs(30)))
                .await
                .map_err(|error| anyhow::anyhow!("background wait task panicked: {error}"))?
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn cap_sixteen_deltas_waits_without_publishing_seventeen_and_keeps_other_work_live() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let mut reached_rx = Some(reached_rx);
            let (published_tx, published_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                reached: reached_tx,
                published: Some(published_tx),
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open hard-cap observed store"),
            );
            create_hard_cap_other_collection(&fixture.server).await;
            store
                .save(&fixture.engine, HARD_CAP_SEQUENCE)
                .expect("publish unrelated collection base");

            let mut first_merge_ready = None;
            for round in 1..=16 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, HARD_CAP_SEQUENCE + round as u64)
                    .expect("publish sparse Keyword layer before hard-cap checkpoint");
                if round == 4 {
                    first_merge_ready = Some(
                        wait_for_worker_ready(
                            reached_rx
                                .take()
                                .expect("fourth save consumes merge readiness receiver"),
                        )
                        .await,
                    );
                }
            }
            let first_merge_ready =
                first_merge_ready.expect("fourth save waits for merge selection");
            let sixteenth_generation = stage1_current_generation_dir(&fixture.root);
            let sixteenth_manifest = stage1_read_manifest(&sixteenth_generation);
            let sixteenth_deltas =
                full_compaction_field_refs(&sixteenth_manifest, FULL_KEYWORD, "delta");

            // This save happens before the capped field becomes dirty for round 17. It
            // proves that a checkpoint with no new capped-field layer still progresses.
            hard_cap_index_other(&fixture.server, HARD_CAP_OTHER_CHECKPOINTED).await;
            let mut unrelated_checkpoint = Some(tokio::task::spawn_blocking({
                let store = store.clone();
                let engine = fixture.engine.clone();
                move || store.save(&engine, HARD_CAP_SEQUENCE + 17)
            }));
            let unrelated_observation = tokio::time::timeout(
                Duration::from_secs(2),
                unrelated_checkpoint
                    .as_mut()
                    .expect("unrelated checkpoint handle"),
            )
            .await;
            let unrelated_finished_while_paused = matches!(&unrelated_observation, Ok(Ok(Ok(_))));

            let mut capped_checkpoint = None;
            let mut capped_observation = None;
            let mut capped_delta_count_after_unrelated = None;
            let mut current_before_capped = None;
            let mut current_while_capped = None;
            let mut live_apply_observation = None;
            let mut live_apply_while_capped = None;
            let mut capped_round17_live = None;
            if first_merge_ready.is_ok() && unrelated_finished_while_paused {
                let after_unrelated =
                    stage1_read_manifest(&stage1_current_generation_dir(&fixture.root));
                capped_delta_count_after_unrelated =
                    Some(full_compaction_field_refs(&after_unrelated, FULL_KEYWORD, "delta").len());
                full_compaction_apply_keyword_round(&fixture.server, 17).await;
                current_before_capped = Some(current_bytes(&fixture.root));
                capped_checkpoint = Some(tokio::task::spawn_blocking({
                    let store = store.clone();
                    let engine = fixture.engine.clone();
                    move || store.save(&engine, HARD_CAP_SEQUENCE + 18)
                }));
                capped_observation = Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        capped_checkpoint
                            .as_mut()
                            .expect("capped checkpoint handle"),
                    )
                    .await,
                );
                current_while_capped = Some(current_bytes(&fixture.root));
                live_apply_observation = Some(
                    tokio::time::timeout(
                        Duration::from_secs(2),
                        hard_cap_index_other(&fixture.server, HARD_CAP_OTHER_LIVE),
                    )
                    .await,
                );
                if matches!(&live_apply_observation, Some(Ok(()))) {
                    if let Ok(ids) = tokio::time::timeout(
                        Duration::from_secs(2),
                        hard_cap_other_ids(&fixture.server, HARD_CAP_OTHER_LIVE),
                    )
                    .await
                    {
                        live_apply_while_capped = Some(ids);
                    }
                    if let Ok(ids) = tokio::time::timeout(
                        Duration::from_secs(2),
                        full_compaction_search_ids(
                            &fixture.server,
                            full_compaction_keyword_query(&full_compaction_round_keyword(17, 0)),
                        ),
                    )
                    .await
                    {
                        capped_round17_live = Some(ids);
                    }
                }
            }

            // Always release before joining a task or evaluating an assertion.
            release.release();
            let capped_waited = matches!(&capped_observation, Some(Err(_)));
            let unrelated_after_release = match &unrelated_observation {
                Err(_) => unrelated_checkpoint
                    .take()
                    .expect("timed-out unrelated checkpoint handle")
                    .await
                    .is_ok_and(|result| result.is_ok()),
                Ok(Ok(result)) => {
                    drop(unrelated_checkpoint.take());
                    result.is_ok()
                }
                Ok(Err(_)) => {
                    drop(unrelated_checkpoint.take());
                    false
                }
            };
            let capped_after_release = match &capped_observation {
                Some(Err(_)) => capped_checkpoint
                    .take()
                    .expect("timed-out capped checkpoint handle")
                    .await
                    .is_ok_and(|result| result.is_ok()),
                Some(Ok(Ok(result))) => {
                    drop(capped_checkpoint.take());
                    result.is_ok()
                }
                Some(Ok(Err(_))) => {
                    drop(capped_checkpoint.take());
                    false
                }
                None => false,
            };
            let drained = wait_for_background_idle(store.clone()).await;
            let published_merges = published_rx.try_iter().count();

            assert!(
                first_merge_ready.is_ok(),
                "the fourth real Keyword delta must select the paused merge: {first_merge_ready:?}",
            );
            assert_eq!(
                sixteenth_deltas.len(),
                16,
                "the paused worker must allow exactly sixteen published deltas before hard-cap admission engages",
            );
            assert!(
                unrelated_finished_while_paused && unrelated_after_release,
                "a checkpoint without a newly dirty capped field must finish while encoding is paused: {unrelated_observation:?}",
            );
            assert_eq!(
                capped_delta_count_after_unrelated,
                Some(16),
                "unrelated checkpoint must not add a seventeenth layer for the capped Keyword field",
            );
            assert!(
                capped_waited,
                "the seventeenth dirty Keyword checkpoint must wait for the selected merge instead of publishing layer 17: {capped_observation:?}",
            );
            assert_eq!(
                published_merges, 2,
                "a running capacity request must queue exactly one successor publication"
            );
            assert_eq!(
                current_while_capped,
                current_before_capped,
                "the current pointer must stay on the sixteen-layer checkpoint until the paused merge releases",
            );
            assert!(
                matches!(&live_apply_observation, Some(Ok(()))),
                "an unrelated HTTP apply must finish within the bounded wait while the capped checkpoint is blocked: {live_apply_observation:?}",
            );
            assert_eq!(
                live_apply_while_capped,
                Some(vec![HARD_CAP_OTHER_ID.to_owned()]),
                "an unrelated live apply and query must progress while the capped checkpoint waits",
            );
            assert_eq!(
                capped_round17_live,
                Some(vec![full_compaction_base_id(0)]),
                "the live engine keeps the seventeenth update visible while durable publication waits",
            );
            assert!(
                capped_after_release,
                "the waiting seventeenth checkpoint must finish after merge release",
            );
            drained.expect("all queued background compactions must finish after release");

            let latest_generation = stage1_current_generation_dir(&fixture.root);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            assert_eq!(
                latest_manifest["checkpoint_sequence"],
                json!(HARD_CAP_SEQUENCE + 18),
                "the released capped checkpoint must become the final durable cut",
            );
            assert!(
                full_compaction_field_refs(&latest_manifest, FULL_KEYWORD, "delta").len() <= 16,
                "no durable catalog may publish a seventeenth Keyword delta",
            );
            assert_eq!(
                hard_cap_other_ids(&fixture.server, HARD_CAP_OTHER_LIVE).await,
                vec![HARD_CAP_OTHER_ID.to_owned()],
                "live state retains the unrelated apply through the released capped checkpoint",
            );
            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open hard-cap cold store")
                .load_current_generation()
                .expect("cold-open hard-cap CURRENT")
                .expect("hard-cap CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("hard-cap cold HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(17, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold CURRENT must retain the seventeenth update after capacity admission releases",
            );
            assert_eq!(
                hard_cap_other_ids(&cold_server, HARD_CAP_OTHER_LIVE).await,
                vec![HARD_CAP_OTHER_ID.to_owned()],
                "cold CURRENT must retain unrelated work that progressed while the cap waited",
            );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn paused_merge_before_publish_cannot_overwrite_a_truncated_epoch() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforePublish,
                reached: reached_tx,
                published: None,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open stale-epoch observed store"),
            );

            for round in 1..=4 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, STALE_EPOCH_SEQUENCE + round as u64)
                    .expect("publish delta before stale-epoch merge");
            }
            let reached = wait_for_worker_ready(reached_rx).await;
            let source_name = stage1_reuse_current_name(&fixture.root);
            let source_manifest = stage1_read_manifest(&fixture.root.join(&source_name));
            let source_epoch = stage1_reuse_collection_u64(
                full_compaction_collection(&source_manifest),
                "collection_generation",
            );
            let active_staging = unpublished_generation_directories(&fixture.root);

            let truncate = fixture
                .server
                .post(&format!(
                    "/collections/{FULL_COMPACTION_COLLECTION}/docs:truncate"
                ))
                .await;
            let truncate_status = truncate.status_code();
            let replacement = if reached.is_ok() {
                store.save(&fixture.engine, STALE_EPOCH_SEQUENCE + 5)
            } else {
                Err(anyhow::anyhow!("merge never reached BeforePublish"))
            };
            let replacement_current = current_bytes(&fixture.root);
            let replacement_manifest =
                stage1_read_manifest(&stage1_current_generation_dir(&fixture.root));
            let replacement_epoch = stage1_reuse_collection_u64(
                full_compaction_collection(&replacement_manifest),
                "collection_generation",
            );
            let old_value_live = full_compaction_search_ids(
                &fixture.server,
                full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
            )
            .await;

            // Release before inspection. A stale worker is allowed to refuse; it is not
            // allowed to install its old collection epoch after this replacement checkpoint.
            release.release();
            let drained = wait_for_background_idle(store.clone()).await;
            let final_current = current_bytes(&fixture.root);
            let final_staging = unpublished_generation_directories(&fixture.root);

            assert!(
                reached.is_ok(),
                "the fourth delta merge must reach BeforePublish outside the root lock: {reached:?}",
            );
            assert_eq!(
                truncate_status,
                axum::http::StatusCode::NO_CONTENT,
                "truncate must advance the live collection epoch before the old merge releases",
            );
            assert!(
                replacement.is_ok(),
                "the replacement checkpoint for a truncated epoch must publish while old merge publication is paused: {replacement:?}",
            );
            assert!(
                !active_staging.is_empty(),
                "a paused worker must own unpublished staging before the epoch replacement",
            );
            assert!(
                replacement_epoch > source_epoch,
                "truncate must allocate a new collection generation before the old merge can publish",
            );
            assert!(
                old_value_live.is_empty(),
                "live truncated state must mask the old merge input before release",
            );
            drained
                .expect("old merge must reach a terminal stale refusal or safe publication result");
            assert_eq!(
                final_current, replacement_current,
                "a stale merge selected from the old epoch must not overwrite replacement CURRENT",
            );
            assert!(
                final_staging.is_empty(),
                "the stale worker must clean its owned staging after terminal refusal",
            );

            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open stale-epoch cold store")
                .load_current_generation()
                .expect("cold-open replacement CURRENT")
                .expect("replacement CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("stale-epoch cold HTTP server");
            assert!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await
                .is_empty(),
                "cold CURRENT must retain the truncated epoch rather than the old merge data",
            );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn prune_and_second_opener_keep_paused_merge_source_and_staging_alive() {
            let fixture = full_compaction_fixture().await;
            let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let mut release = MergeRelease::new(release_tx);
            let observer = Arc::new(PauseOneMergePhase {
                phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                reached: reached_tx,
                published: None,
                release: Mutex::new(Some(release_rx)),
                paused: std::sync::atomic::AtomicBool::new(false),
            });
            let store = Arc::new(
                SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                    .expect("open prune observed store"),
            );

            for round in 1..=4 {
                full_compaction_apply_keyword_round(&fixture.server, round).await;
                store
                    .save(&fixture.engine, PRUNE_SEQUENCE + round as u64)
                    .expect("publish delta before prune race");
            }
            let reached = wait_for_worker_ready(reached_rx).await;
            let source_name = stage1_reuse_current_name(&fixture.root);
            let source_path = fixture.root.join(&source_name);
            let staging_before_prune = unpublished_generation_directories(&fixture.root);

            let prune = if reached.is_ok() {
                tokio::task::spawn_blocking({
                    let store = store.clone();
                    move || store.prune(1)
                })
                .await
                .expect("prune task must not panic")
            } else {
                Err(anyhow::anyhow!("merge never reached BeforeEncode"))
            };
            let source_after_prune = source_path.is_dir();
            let staging_after_prune = unpublished_generation_directories(&fixture.root);
            let second_open = SegmentRdbStore::new(&fixture.root)
                .and_then(|second| second.load_current_generation());
            let second_engine = second_open
                .as_ref()
                .ok()
                .and_then(|loaded| loaded.as_ref().map(|loaded| loaded.engine.clone()));
            let source_after_second_open = source_path.is_dir();
            let staging_after_second_open = unpublished_generation_directories(&fixture.root);

            // Always release the real worker before evaluating the prune/open observations.
            release.release();
            let drained = wait_for_background_idle(store.clone()).await;
            let latest_name = stage1_reuse_current_name(&fixture.root);
            let latest_generation = fixture.root.join(&latest_name);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            let staging_after_release = unpublished_generation_directories(&fixture.root);

            assert!(
                reached.is_ok(),
                "the fourth delta merge must own a source and staging directory before prune",
            );
            assert!(
                prune.is_ok(),
                "prune(1) must finish while worker encoding is paused: {prune:?}"
            );
            assert!(
                !staging_before_prune.is_empty(),
                "the paused worker must have unpublished staging for the selected merge",
            );
            assert!(
                source_after_prune && source_after_second_open,
                "prune and a second store opener must retain the selected source generation",
            );
            assert_eq!(
                staging_after_prune, staging_before_prune,
                "prune must not reclaim active background staging",
            );
            assert_eq!(
                staging_after_second_open, staging_before_prune,
                "a second store opener must not sweep active staging as abandoned",
            );
            assert!(
                second_engine.is_some(),
                "a second store must cold-open CURRENT while the worker owns source and staging",
            );
            drained.expect("released background merge must finish after prune/open race");
            assert_ne!(
                latest_name, source_name,
                "released worker must publish a new merged CURRENT after its protected source survives",
            );
            assert!(
                full_compaction_field_refs(&latest_manifest, FULL_KEYWORD, "delta").len() < 4,
                "released selected merge must reduce the four-layer Keyword catalog",
            );
            assert!(
                staging_after_release.is_empty(),
                "completed worker must clean its now-unowned staging directory",
            );
            assert_eq!(
                full_compaction_search_ids(
                    &fixture.server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "live state must retain the merged fourth Keyword update after prune/open race",
            );
            let premerge_server = TestServer::new(router(AppState::open(
                second_engine.expect("second store cold engine"),
            )))
            .expect("premerge second-store HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &premerge_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "the second opener's cold source must keep the fourth layer while merge is paused",
            );
            let cold = SegmentRdbStore::new(&fixture.root)
                .expect("open post-prune cold store")
                .load_current_generation()
                .expect("cold-open merged CURRENT")
                .expect("merged CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("post-prune cold HTTP server");
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(4, 0)),
                )
                .await,
                vec![full_compaction_base_id(0)],
                "cold merged CURRENT must retain the fourth Keyword update after prune/open race",
            );
        }

        mod durable_restore_vs_paused_merge_contract {
            //! # Facets
            //!
            //! - Behavior: `indexing_durable_oracle.rs:8667`, `:8671`, `:8710`, and
            //!   `:8723` require a durable restore to activate its candidate before the
            //!   paused old merge releases, then retain that candidate live and cold.
            //!   Change points: `apps/lumen/src/segment_restore.rs:154-289` and
            //!   `apps/lumen/src/segment_background_merge.rs:449-552`.
            //! - Security: `indexing_durable_oracle.rs:8694` and `:8723` require the
            //!   process-written `CURRENT` input to remain on the restore candidate after
            //!   stale-worker release. Boundary: `apps/lumen/src/segment_rdb.rs:635-676`.
            //! - Performance: `apps/lumen/ROADMAP.md:73-78` promises one background
            //!   merge that rechecks segment input and collection epoch before publication.
            //!   `indexing_durable_oracle.rs:8694` asserts that structural promise; the
            //!   two-second observation is only an interleaving control, not a latency claim.

            use super::*;

            const RESTORED_COLLECTION: &str = "background-restored";
            const RESTORED_FIELD: &str = "kw";
            const RESTORED_ID: &str = "background-restored-id";
            const RESTORED_VALUE: &str = "background-restored-value";

            struct RestoreRaceFixture {
                _dir: tempfile::TempDir,
                root: PathBuf,
                engine: Arc<Engine>,
                writer: Arc<WriteCoordinator>,
                aof: SharedAof,
                server: TestServer,
            }

            async fn restore_race_fixture() -> RestoreRaceFixture {
                let dir = tempfile::tempdir().expect("restore-versus-merge fixture root");
                let root = dir.path().join("segments");
                let store = SegmentRdbStore::new(&root).expect("open restore-versus-merge store");
                let engine = Arc::new(Engine::new());
                let aof: SharedAof = Arc::new(Mutex::new(
                    AofWriter::open(dir.path().join("restore-versus-merge.aof"))
                        .expect("open restore-versus-merge AOF"),
                ));
                let wal: SharedWal = Arc::new(MemWal::new());
                let writer =
                    WriteCoordinator::start_from_with_aof(wal, engine.clone(), 0, aof.clone());
                let sink_writer: Arc<dyn WriteSink> = writer.clone();
                let server = TestServer::new(router(AppState::with_components(
                    engine.clone(),
                    Arc::new(AuthConfig::open()),
                    sink_writer,
                )))
                .expect("restore-versus-merge HTTP server");
                full_compaction_create_collection(&server).await;
                full_compaction_post_items(&server, full_compaction_base_items()).await;
                stage1_restore_legacy_base(&engine, "restore-versus-merge base");
                store
                    .save(&engine, writer.applied_seq())
                    .expect("publish restore-versus-merge base at real writer sequence");
                RestoreRaceFixture {
                    _dir: dir,
                    root,
                    engine,
                    writer,
                    aof,
                    server,
                }
            }

            async fn restored_snapshot() -> SnapshotV1 {
                let candidate = Arc::new(Engine::new());
                let server = TestServer::new(router(AppState::open(candidate.clone())))
                    .expect("restored candidate server");
                server
                    .put(&format!("/collections/{RESTORED_COLLECTION}"))
                    .json(&json!({ "fields": { RESTORED_FIELD: { "type": "keyword" } } }))
                    .await
                    .assert_status_ok();
                server
                    .post(&format!("/collections/{RESTORED_COLLECTION}/index"))
                    .json(&json!({ "items": [{
                        "external_id": RESTORED_ID,
                        "field": RESTORED_FIELD,
                        "value": RESTORED_VALUE,
                    }] }))
                    .await
                    .assert_status_ok();
                candidate.snapshot().expect("snapshot restored candidate")
            }

            async fn restored_ids(server: &TestServer) -> Vec<String> {
                let response = server
                    .post(&format!("/collections/{RESTORED_COLLECTION}/search"))
                    .json(&json!({
                        "query": { "term": { "field": RESTORED_FIELD, "value": RESTORED_VALUE } },
                        "limit": 8,
                        "track_total": true,
                    }))
                    .await;
                response.assert_status_ok();
                let mut ids = response.json::<Value>()["hits"]
                    .as_array()
                    .expect("restored query hits")
                    .iter()
                    .map(|hit| {
                        hit["external_id"]
                            .as_str()
                            .expect("restored query external ID")
                            .to_owned()
                    })
                    .collect::<Vec<_>>();
                ids.sort();
                ids
            }

            #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
            async fn durable_restore_while_before_publish_merge_is_paused_rejects_old_engine_epoch()
            {
                let fixture = restore_race_fixture().await;
                let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
                let (release_tx, release_rx) = std::sync::mpsc::channel();
                let mut release = MergeRelease::new(release_tx);
                let observer = Arc::new(PauseOneMergePhase {
                    phase: lumen::segment_rdb::MergePhase::BeforePublish,
                    reached: reached_tx,
                    published: None,
                    release: Mutex::new(Some(release_rx)),
                    paused: std::sync::atomic::AtomicBool::new(false),
                });
                let store = Arc::new(
                    SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                        .expect("open restore-versus-merge store"),
                );
                for round in 1..=4 {
                    full_compaction_apply_keyword_round(&fixture.server, round).await;
                    store
                        .save(&fixture.engine, fixture.writer.applied_seq())
                        .expect("publish delta before restore-versus-merge race");
                }
                let reached = wait_for_worker_ready(reached_rx).await;
                let source_current = current_bytes(&fixture.root);
                let staging_before_restore = unpublished_generation_directories(&fixture.root);

                let sink = Arc::new(
                    lumen::segment_restore::SegmentRestoreSink::new(
                        fixture.engine.clone(),
                        store.clone(),
                        fixture.writer.clone() as Arc<dyn WriteSink>,
                        fixture.aof.clone(),
                    )
                    .expect("real durable restore sink"),
                );
                let snapshot = restored_snapshot().await;
                let mut restore = tokio::spawn({
                    let sink = sink.clone();
                    async move { lumen::api::RestoreSink::restore(sink.as_ref(), snapshot).await }
                });
                let restore_before_release =
                    tokio::time::timeout(Duration::from_secs(2), &mut restore).await;
                let restore_finished_before_release =
                    matches!(&restore_before_release, Ok(Ok(Ok(()))));
                let candidate_current = current_bytes(&fixture.root);
                let candidate_collections = store
                    .load_current_generation()
                    .ok()
                    .flatten()
                    .and_then(|loaded| loaded.engine.list_collections().ok());
                let live_collections_before_release = fixture.engine.list_collections().ok();

                // Always release before joining. `MergeRelease` repeats this on unwinding.
                release.release();
                let restore_result = match restore_before_release {
                    Ok(result) => result.expect("restore task must not panic"),
                    Err(_) => tokio::time::timeout(Duration::from_secs(30), restore)
                        .await
                        .expect("restore must finish after worker release")
                        .expect("restore task must not panic"),
                };
                let drained = wait_for_background_idle(store.clone()).await;
                let final_current = current_bytes(&fixture.root);
                let final_staging = unpublished_generation_directories(&fixture.root);

                assert!(
                    reached.is_ok(),
                    "the fourth real Keyword delta must select a merge before its publication pause: {reached:?}",
                );
                assert!(
                    restore_finished_before_release,
                    "the real durable restore must finish while the old merge pauses outside the root lock",
                );
                assert!(
                    restore_result.is_ok(),
                    "restore must activate its candidate after durable CURRENT publication: {restore_result:?}",
                );
                assert_ne!(
                    candidate_current, source_current,
                    "the durable restore must replace the old merge source CURRENT before release",
                );
                assert!(
                    !staging_before_restore.is_empty(),
                    "the selected old merge must own staging before the restore changes the Engine epoch",
                );
                assert_eq!(
                    candidate_collections,
                    Some(vec![RESTORED_COLLECTION.to_owned()]),
                    "the durable candidate CURRENT must name only the restored collection before old merge release",
                );
                assert_eq!(
                    live_collections_before_release,
                    Some(vec![RESTORED_COLLECTION.to_owned()]),
                    "the live Engine must activate the restored epoch before old merge release",
                );
                drained.expect("the released stale merge must reach a terminal safe result");
                assert_eq!(
                    final_current, candidate_current,
                    "the old merge must not overwrite restore-owned CURRENT after its old Engine epoch is invalid",
                );
                assert!(
                    final_staging.is_empty(),
                    "the old merge must clean owned staging after its stale epoch is refused",
                );
                assert_eq!(
                    fixture
                        .engine
                        .list_collections()
                        .expect("list restored live collections"),
                    vec![RESTORED_COLLECTION.to_owned()],
                    "live state must not resurrect the old merge collection after release",
                );
                assert_eq!(
                    restored_ids(&fixture.server).await,
                    vec![RESTORED_ID.to_owned()],
                    "live search must retain the restored value after the stale merge releases",
                );

                let cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open final restore root")
                    .load_current_generation()
                    .expect("cold-open final restore CURRENT")
                    .expect("final restore CURRENT generation");
                let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                    .expect("final restore cold server");
                assert_eq!(
                    restored_ids(&cold_server).await,
                    vec![RESTORED_ID.to_owned()],
                    "cold CURRENT must retain the restored value and not the old merge output",
                );
            }
        }

        mod pending_frozen_selected_merge_contract {
            //! # Facets
            //!
            //! - Behavior: `indexing_durable_oracle.rs:8860`, `:8883`, `:8891`, and
            //!   `:8900` require the failed fifth cut to retry before the sixth live
            //!   mutation is captured. Change points: `apps/lumen/src/segment_rdb.rs:459-513`
            //!   and `apps/lumen/src/segment_background_merge.rs:447-501`.
            //! - Security: `indexing_durable_oracle.rs:8864`, `:8875`, and `:8891` keep
            //!   process-written `CURRENT` and its predecessor from making frozen input
            //!   stale. Boundary: `apps/lumen/src/segment_rdb.rs:637-676`.
            //! - Performance: `apps/lumen/ROADMAP.md:52-55` promises that a
            //!   pre-publication failure retains frozen changes; `:73-78` requires merge
            //!   input revalidation. `indexing_durable_oracle.rs:8875` asserts both
            //!   structural rules. Its waits are cleanup and interleaving controls only.

            use super::*;
            use std::sync::atomic::{AtomicBool, Ordering};
            use storage_durable::{CommitStep, FailureInjector, FailurePoint};

            const PENDING_FROZEN_SEQUENCE: u64 = 14_100;

            #[derive(Default)]
            struct FailNextSyncFile {
                armed: AtomicBool,
            }

            impl FailNextSyncFile {
                fn arm(&self) {
                    self.armed.store(true, Ordering::Release);
                }
            }

            impl FailureInjector for FailNextSyncFile {
                fn check(&self, point: &FailurePoint) -> std::io::Result<()> {
                    if point.step == CommitStep::SyncFile
                        && self.armed.swap(false, Ordering::AcqRel)
                    {
                        return Err(std::io::Error::other(
                            "injected pending-frozen SyncFile failure",
                        ));
                    }
                    Ok(())
                }
            }

            #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
            async fn failed_frozen_checkpoint_blocks_selected_merge_and_retries_its_original_cut() {
                let fixture = full_compaction_fixture().await;
                let (reached_tx, reached_rx) = std::sync::mpsc::sync_channel(1);
                let (release_tx, release_rx) = std::sync::mpsc::channel();
                let mut release = MergeRelease::new(release_tx);
                let observer = Arc::new(PauseOneMergePhase {
                    phase: lumen::segment_rdb::MergePhase::BeforeEncode,
                    reached: reached_tx,
                    published: None,
                    release: Mutex::new(Some(release_rx)),
                    paused: AtomicBool::new(false),
                });
                let injector = Arc::new(FailNextSyncFile::default());
                let store = Arc::new(
                    SegmentRdbStore::with_failure_injector_and_merge_observer(
                        &fixture.root,
                        injector.clone(),
                        observer,
                    )
                    .expect("open failure-injected observed store"),
                );

                for round in 1..=4 {
                    full_compaction_apply_keyword_round(&fixture.server, round).await;
                    store
                        .save(&fixture.engine, PENDING_FROZEN_SEQUENCE + round as u64)
                        .expect("publish four sparse Keyword layers before selected merge");
                }
                let reached = wait_for_worker_ready(reached_rx).await;
                let current_before_failure = current_bytes(&fixture.root);
                let staging_before_failure = unpublished_generation_directories(&fixture.root);

                full_compaction_apply_keyword_round(&fixture.server, 5).await;
                injector.arm();
                let failed_checkpoint = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 5);
                let current_after_failure = current_bytes(&fixture.root);

                // This sixth mutation stays live. A retry of the frozen fifth cut must
                // not capture it early. If the old merge advances CURRENT, the original
                // pending predecessor goes stale and this distinction catches the loss.
                full_compaction_apply_keyword_round(&fixture.server, 6).await;

                // Release before every join or assertion. The drop guard repeats this on panic.
                release.release();
                let merge_after_release = wait_for_background_idle(store.clone()).await;
                let current_after_old_merge = current_bytes(&fixture.root);
                let staging_after_old_merge = unpublished_generation_directories(&fixture.root);

                let retry_fifth = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 5);
                let retry_current = current_bytes(&fixture.root);
                let retry_cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open retry root")
                    .load_current_generation()
                    .expect("cold-open fifth retry")
                    .expect("fifth retry CURRENT exists");
                let retry_server = TestServer::new(router(AppState::open(retry_cold.engine)))
                    .expect("fifth retry cold server");
                let fifth_ids = full_compaction_search_ids(
                    &retry_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(5, 0)),
                )
                .await;
                let sixth_ids_before_fresh_capture = full_compaction_search_ids(
                    &retry_server,
                    full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                )
                .await;

                let fresh_sixth = store.save(&fixture.engine, PENDING_FROZEN_SEQUENCE + 6);
                wait_for_background_idle(store.clone())
                    .await
                    .expect("fresh sixth checkpoint and requested merge finish");
                let final_cold = SegmentRdbStore::new(&fixture.root)
                    .expect("open final retry root")
                    .load_current_generation()
                    .expect("cold-open sixth checkpoint")
                    .expect("sixth checkpoint CURRENT exists");
                let final_server = TestServer::new(router(AppState::open(final_cold.engine)))
                    .expect("sixth checkpoint cold server");

                assert!(
                    reached.is_ok(),
                    "the fourth real Keyword delta must select the merge before checkpoint failure: {reached:?}",
                );
                assert!(
                    failed_checkpoint.is_err(),
                    "the armed pre-publication SyncFile failure must fail the fifth checkpoint",
                );
                assert_eq!(
                    current_after_failure, current_before_failure,
                    "a failed checkpoint must not move CURRENT before it retains the frozen cut",
                );
                assert!(
                    !staging_before_failure.is_empty(),
                    "the selected merge must own staging before the fifth checkpoint fails",
                );
                merge_after_release.expect(
                    "the old selected merge must safely refuse once pending frozen payload owns its predecessor",
                );
                assert_eq!(
                    current_after_old_merge, current_before_failure,
                    "the released old merge must not move CURRENT past the pending frozen predecessor",
                );
                assert!(
                    staging_after_old_merge.is_empty(),
                    "the old merge must clean staging after it observes pending frozen work",
                );
                assert!(
                    retry_fifth.is_ok(),
                    "retrying the original fifth sequence must publish retained frozen data: {retry_fifth:?}",
                );
                assert_ne!(
                    retry_current, current_before_failure,
                    "the fifth retry must create its durable generation after old merge refusal",
                );
                assert_eq!(
                    fifth_ids,
                    vec![full_compaction_base_id(0)],
                    "cold retry must retain the fifth frozen Keyword value",
                );
                assert!(
                    sixth_ids_before_fresh_capture.is_empty(),
                    "retrying fifth frozen payload must not silently recapture the later sixth live mutation",
                );
                assert!(
                    fresh_sixth.is_ok(),
                    "a later explicit sixth checkpoint must publish the remaining live mutation: {fresh_sixth:?}",
                );
                assert_eq!(
                    full_compaction_search_ids(
                        &final_server,
                        full_compaction_keyword_query(&full_compaction_round_keyword(6, 0)),
                    )
                    .await,
                    vec![full_compaction_base_id(0)],
                    "the later fresh checkpoint must retain the sixth mutation after fifth retry",
                );
            }
        }
    }

    mod merge_scheduler_priority_contract {
        //! This is a narrow scheduler oracle. It does not accept equivalent
        //! search results as proof: the published catalog must show that the
        //! deepest field won first, and that its smallest adjacent byte pair
        //! was the only pair replaced.

        use super::*;
        use lumen::segment_rdb::{MergeObserver, MergePhase};
        use std::collections::BTreeSet;
        use std::io;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::mpsc;

        const SCHEDULER_SEQUENCE: u64 = 13_900;
        const SCHEDULER_EXTRA_BASE_ROWS: usize = 1_024;
        const SCHEDULER_EXTRA_KEYWORD_BASE_ROWS: usize = 32_768;

        struct FailThenPauseMergeObserver {
            attempts: AtomicUsize,
            failures_before_pause: usize,
            reached: mpsc::SyncSender<usize>,
            release: Mutex<Option<mpsc::Receiver<()>>>,
            ready: Arc<(Mutex<bool>, Condvar)>,
        }

        impl MergeObserver for FailThenPauseMergeObserver {
            fn observe(&self, phase: MergePhase) -> io::Result<()> {
                if phase != MergePhase::BeforeEncode {
                    return Ok(());
                }
                let (ready_lock, ready_cv) = &*self.ready;
                let mut ready = ready_lock
                    .lock()
                    .map_err(|_| io::Error::other("merge scheduler ready mutex poisoned"))?;
                while !*ready {
                    ready = ready_cv
                        .wait(ready)
                        .map_err(|_| io::Error::other("merge scheduler ready wait poisoned"))?;
                }
                drop(ready);
                let attempt = self.attempts.fetch_add(1, Ordering::SeqCst) + 1;
                self.reached.send(attempt).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "merge scheduler observer receiver dropped",
                    )
                })?;
                if attempt <= self.failures_before_pause {
                    return Err(io::Error::other(format!(
                        "merge scheduler setup failure {attempt}"
                    )));
                }
                if attempt == self.failures_before_pause + 1 {
                    self.release
                        .lock()
                        .expect("merge scheduler release mutex")
                        .take()
                        .expect("merge scheduler releases the selected merge once")
                        .recv()
                        .map_err(|_| {
                            io::Error::new(
                                io::ErrorKind::BrokenPipe,
                                "merge scheduler release sender dropped",
                            )
                        })?;
                }
                Ok(())
            }
        }

        struct MergeSchedulerRelease(Option<mpsc::Sender<()>>);

        impl MergeSchedulerRelease {
            fn release(&mut self) {
                if let Some(sender) = self.0.take() {
                    let _ = sender.send(());
                }
            }
        }

        impl Drop for MergeSchedulerRelease {
            fn drop(&mut self) {
                self.release();
            }
        }

        fn scheduler_body_value(round: usize) -> String {
            let words = match round {
                // Keep the seven layers physically different while keeping
                // their total below the body base. This fixture must exercise
                // the partial-pair path, not base compaction.
                // Use separated, non-power-of-two payload sizes.  The v2
                // payload and local-row files have fixed framing overhead, so
                // the old 1,2,4,... sequence could collapse to equal on-disk
                // sizes and make the fixture's unique-pair precondition false.
                1 => 17,
                2 => 61,
                3 => 257,
                4 => 1_021,
                5 => 2_047,
                6 => 3_073,
                7 => 4_093,
                _ => panic!("scheduler contract needs seven body rounds"),
            };
            full_compaction_entropy_term(900_000 + round as u64, words)
        }

        fn scheduler_keyword_value(round: usize) -> String {
            let words = match round {
                1 => 17,
                2 => 61,
                3 => 257,
                4 => 1_021,
                5 => 2_047,
                6 => 3_073,
                7 => 4_093,
                _ => panic!("scheduler contract needs seven keyword rounds"),
            };
            full_compaction_entropy_term(910_000 + round as u64, words)
        }

        async fn scheduler_fixture() -> FullCompactionFixture {
            let dir = tempfile::tempdir().expect("scheduler fixture root");
            let root = dir.path().join("segments");
            let store = SegmentRdbStore::new(&root).expect("create scheduler store");
            let engine = Arc::new(Engine::new());
            let server = TestServer::new(router(AppState::open(engine.clone())))
                .expect("scheduler HTTP server");
            full_compaction_create_collection(&server).await;

            let mut items = full_compaction_base_items();
            for index in 0..SCHEDULER_EXTRA_BASE_ROWS {
                items.push(json!({
                    "external_id": format!("scheduler-base-body-{index:04}"),
                    "field": FULL_TEXT,
                    "value": full_compaction_entropy_term(2_000_000 + index as u64, 16),
                }));
            }
            for index in 0..SCHEDULER_EXTRA_KEYWORD_BASE_ROWS {
                items.push(json!({
                    "external_id": format!("scheduler-base-keyword-{index:04}"),
                    "field": FULL_KEYWORD,
                    "value": format!("scheduler-base-keyword-{index:04}"),
                }));
            }
            full_compaction_post_items(&server, items).await;
            stage1_restore_legacy_base(&engine, "merge scheduler base");
            store
                .save(&engine, FULL_COMPACTION_BASE_SEQUENCE)
                .expect("publish merge scheduler base");
            FullCompactionFixture {
                _dir: dir,
                root: root.clone(),
                store,
                engine,
                server,
                base_name: stage1_reuse_current_name(&root),
            }
        }

        async fn scheduler_update(server: &TestServer, field: &str, round: usize) {
            let (external_id, value) = match field {
                FULL_TEXT => (
                    full_compaction_base_id(round - 1),
                    json!(scheduler_body_value(round)),
                ),
                FULL_KEYWORD => (
                    full_compaction_base_id(32 + round),
                    json!(scheduler_keyword_value(round)),
                ),
                FULL_HASH => (
                    full_compaction_base_id(64 + round),
                    json!(format!("{:016x}", 0x9000_u64 + round as u64)),
                ),
                FULL_NUMBER => (
                    full_compaction_base_id(96 + round),
                    json!(90_000.0 + round as f64),
                ),
                _ => panic!("scheduler contract does not support field {field}"),
            };
            full_compaction_post_items(
                server,
                vec![json!({
                    "external_id": external_id,
                    "field": field,
                    "value": value,
                })],
            )
            .await;
        }

        fn scheduler_reference_bytes(generation: &Path, reference: &Value) -> u64 {
            let payload = std::fs::metadata(
                generation.join(reference["path"].as_str().expect("scheduler segment path")),
            )
            .expect("inspect scheduler delta payload")
            .len();
            let rows = reference
                .get("local_rows")
                .and_then(Value::as_object)
                .map(|rows| {
                    std::fs::metadata(
                        generation.join(rows["path"].as_str().expect("scheduler local row path")),
                    )
                    .expect("inspect scheduler local row map")
                    .len()
                })
                .unwrap_or(0);
            payload
                .checked_add(rows)
                .expect("scheduler delta byte count does not overflow")
        }

        fn scheduler_delta_bytes(generation: &Path, reference: &Value) -> u64 {
            scheduler_reference_bytes(generation, reference)
        }

        fn scheduler_base_bytes(generation: &Path, manifest: &Value, field: &str) -> u64 {
            scheduler_reference_bytes(
                generation,
                full_compaction_base_ref(manifest, field, "field"),
            )
        }

        fn scheduler_smallest_adjacent_pair(
            generation: &Path,
            references: &[&Value],
        ) -> (u64, u64) {
            let pairs: Vec<_> = references
                .windows(2)
                .map(|pair| {
                    (
                        scheduler_delta_bytes(generation, pair[0])
                            .checked_add(scheduler_delta_bytes(generation, pair[1]))
                            .expect("scheduler adjacent pair byte count does not overflow"),
                        pair[0]["ordinal"]
                            .as_u64()
                            .expect("scheduler first pair ordinal"),
                        pair[1]["ordinal"]
                            .as_u64()
                            .expect("scheduler second pair ordinal"),
                    )
                })
                .collect();
            let minimum = pairs
                .iter()
                .map(|(bytes, _, _)| *bytes)
                .min()
                .expect("five scheduler deltas produce adjacent pairs");
            let minima: Vec<_> = pairs
                .iter()
                .filter(|(bytes, _, _)| *bytes == minimum)
                .collect();
            assert_eq!(
                minima.len(),
                1,
                "scheduler fixture must have one smallest adjacent pair, got {minima:?}"
            );
            (minima[0].1, minima[0].2)
        }

        fn scheduler_rows(generation: &Path, reference: &Value) -> BTreeSet<String> {
            stage1_keyword_delta_read_rows(generation, reference)
                .into_iter()
                .collect()
        }

        fn scheduler_reference_by_ordinal<'a>(references: &'a [&Value], ordinal: u64) -> &'a Value {
            references
                .iter()
                .copied()
                .find(|reference| reference["ordinal"].as_u64() == Some(ordinal))
                .unwrap_or_else(|| panic!("scheduler catalog needs delta ordinal {ordinal}"))
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn merge_scheduler_compacts_only_deepest_tied_fields_before_external_checkpoint() {
            let fixture = scheduler_fixture().await;
            let (reached_tx, reached_rx) = mpsc::sync_channel(32);
            let (release_tx, release_rx) = mpsc::channel();
            let scheduler_ready = Arc::new((Mutex::new(false), Condvar::new()));
            let mut release = MergeSchedulerRelease(Some(release_tx));
            let observer = Arc::new(FailThenPauseMergeObserver {
                attempts: AtomicUsize::new(0),
                failures_before_pause: 10,
                reached: reached_tx,
                release: Mutex::new(Some(release_rx)),
                ready: scheduler_ready.clone(),
            });
            let store = SegmentRdbStore::with_merge_observer(&fixture.root, observer)
                .expect("open observed merge scheduler store");

            // Keep a 3-layer shallow field and a 6-layer middle field ready
            // before the two tied 7-layer fields. The first ten attempts are
            // deliberate failures, so the paused eleventh attempt observes
            // the complete 7/7/6/3 catalog in one immutable source.
            let mut sequence = SCHEDULER_SEQUENCE;
            for round in 1..=3 {
                scheduler_update(&fixture.server, FULL_NUMBER, round).await;
                sequence += 1;
                store
                    .save(&fixture.engine, sequence)
                    .expect("publish shallow scheduler delta");
            }
            for round in 1..=6 {
                scheduler_update(&fixture.server, FULL_HASH, round).await;
                sequence += 1;
                store
                    .save(&fixture.engine, sequence)
                    .expect("publish middle scheduler delta");
            }
            for round in 1..=7 {
                scheduler_update(&fixture.server, FULL_TEXT, round).await;
                sequence += 1;
                store
                    .save(&fixture.engine, sequence)
                    .expect("publish first deep scheduler delta");
            }
            for round in 1..=7 {
                scheduler_update(&fixture.server, FULL_KEYWORD, round).await;
                sequence += 1;
                store
                    .save(&fixture.engine, sequence)
                    .expect("publish tied deep scheduler delta");
            }
            assert_eq!(sequence, SCHEDULER_SEQUENCE + 23);
            {
                let (ready_lock, ready_cv) = &*scheduler_ready;
                let mut ready = ready_lock
                    .lock()
                    .expect("merge scheduler ready mutex");
                *ready = true;
                ready_cv.notify_one();
            }
            let attempts: Vec<_> = (0..=10)
                .map(|_| {
                    reached_rx
                        .recv_timeout(Duration::from_secs(30))
                        .expect("merge scheduler attempt must reach the observer")
                })
                .collect();
            assert_eq!(
                attempts,
                (1..=11).collect::<Vec<_>>(),
                "the observed source must be reached only after the ten setup failures"
            );

            let before_generation = stage1_current_generation_dir(&fixture.root);
            let before_manifest = stage1_read_manifest(&before_generation);
            assert_eq!(
                before_manifest["checkpoint_sequence"],
                json!(sequence),
                "the observed merge must read the complete scheduler fixture"
            );
            let before_tied_a = full_compaction_field_refs(&before_manifest, FULL_TEXT, "delta");
            let before_tied_b = full_compaction_field_refs(&before_manifest, FULL_KEYWORD, "delta");
            let before_middle = full_compaction_field_refs(&before_manifest, FULL_HASH, "delta");
            let before_shallow = full_compaction_field_refs(&before_manifest, FULL_NUMBER, "delta");
            assert_eq!(
                before_tied_a.len(),
                7,
                "first deepest field must have seven deltas"
            );
            assert_eq!(
                before_tied_b.len(),
                7,
                "tied deepest field must have seven deltas"
            );
            assert_eq!(before_middle.len(), 6, "middle field must have six deltas");
            assert_eq!(
                before_shallow.len(),
                3,
                "shallow field must have three deltas"
            );
            let body_base_bytes =
                scheduler_base_bytes(&before_generation, &before_manifest, FULL_TEXT);
            let body_delta_bytes = before_tied_a.iter().fold(0u64, |total, reference| {
                total
                    .checked_add(scheduler_delta_bytes(&before_generation, reference))
                    .expect("scheduler body delta byte count does not overflow")
            });
            assert!(
                body_delta_bytes < body_base_bytes,
                "scheduler fixture must exercise pair compaction: body delta bytes {body_delta_bytes} must stay below body base bytes {body_base_bytes}",
            );
            let keyword_base_bytes =
                scheduler_base_bytes(&before_generation, &before_manifest, FULL_KEYWORD);
            let keyword_delta_bytes = before_tied_b.iter().fold(0u64, |total, reference| {
                total
                    .checked_add(scheduler_delta_bytes(&before_generation, reference))
                    .expect("scheduler keyword delta byte count does not overflow")
            });
            assert!(
                keyword_delta_bytes < keyword_base_bytes,
                "scheduler fixture must exercise keyword pair compaction: delta bytes {keyword_delta_bytes} must stay below base bytes {keyword_base_bytes}",
            );
            let mut expected_pairs = Vec::new();
            for (field, references) in [(FULL_TEXT, &before_tied_a), (FULL_KEYWORD, &before_tied_b)]
            {
                let (first_ordinal, second_ordinal) =
                    scheduler_smallest_adjacent_pair(&before_generation, references);
                let first_before = scheduler_reference_by_ordinal(references, first_ordinal);
                let second_before = scheduler_reference_by_ordinal(references, second_ordinal);
                let expected_rows = scheduler_rows(&before_generation, first_before)
                    .into_iter()
                    .chain(scheduler_rows(&before_generation, second_before))
                    .collect::<BTreeSet<_>>();
                expected_pairs.push((field, first_ordinal, second_ordinal, expected_rows));
            }

            // Always release before an assertion can unwind. The guard keeps
            // the process-wide merge worker from remaining blocked.
            release.release();
            store
                .wait_for_merges(Duration::from_secs(30))
                .expect("selected merge and its follow-up worker must finish");

            let after_generation = stage1_current_generation_dir(&fixture.root);
            let after_manifest = stage1_read_manifest(&after_generation);
            let after_tied_a = full_compaction_field_refs(&after_manifest, FULL_TEXT, "delta");
            let after_tied_b = full_compaction_field_refs(&after_manifest, FULL_KEYWORD, "delta");
            let after_middle = full_compaction_field_refs(&after_manifest, FULL_HASH, "delta");
            let after_shallow = full_compaction_field_refs(&after_manifest, FULL_NUMBER, "delta");
            assert_eq!(
                after_tied_a.len(),
                6,
                "the first deepest field must lose one pair layer"
            );
            assert_eq!(
                after_tied_b.len(),
                6,
                "the tied deepest field must lose one pair layer"
            );
            assert_eq!(
                after_middle.len(),
                6,
                "the six-layer field must remain untouched"
            );
            assert_eq!(
                after_shallow.len(),
                3,
                "the three-layer field must remain untouched"
            );
            for (field, first_ordinal, second_ordinal, expected_rows) in expected_pairs {
                let before = if field == FULL_TEXT {
                    &before_tied_a
                } else {
                    &before_tied_b
                };
                let after = if field == FULL_TEXT {
                    &after_tied_a
                } else {
                    &after_tied_b
                };
                assert!(
                    !after
                        .iter()
                        .any(|reference| reference["ordinal"].as_u64() == Some(first_ordinal)),
                    "the first member of the {field} smallest pair must be removed"
                );
                let compacted = scheduler_reference_by_ordinal(after, second_ordinal);
                assert_eq!(
                    scheduler_rows(&after_generation, compacted),
                    expected_rows,
                    "the {field} replacement must contain exactly the selected pair rows"
                );
                assert_eq!(
                    compacted["applied_seq"],
                    json!(sequence),
                    "the {field} replacement must be newly written at the observed cut"
                );
                for reference in before {
                    let ordinal = reference["ordinal"].as_u64().expect("deep delta ordinal");
                    if ordinal == first_ordinal || ordinal == second_ordinal {
                        continue;
                    }
                    let retained = scheduler_reference_by_ordinal(after, ordinal);
                    assert_eq!(
                        retained["payload_sha256"], reference["payload_sha256"],
                        "unselected {field} ordinal {ordinal} must retain its original payload"
                    );
                }
            }
            for (field, before, after) in [
                (FULL_HASH, &before_middle, &after_middle),
                (FULL_NUMBER, &before_shallow, &after_shallow),
            ] {
                for reference in before {
                    let ordinal = reference["ordinal"]
                        .as_u64()
                        .expect("retained delta ordinal");
                    let retained = scheduler_reference_by_ordinal(after, ordinal);
                    assert_eq!(
                        retained["payload_sha256"],
                        reference["payload_sha256"],
                        "the {field} ordinal {ordinal} must retain its identity until a later checkpoint"
                    );
                }
            }

            let live_keyword = full_compaction_search_ids(
                &fixture.server,
                full_compaction_keyword_query(&scheduler_keyword_value(7)),
            )
            .await;
            let live_text =
                full_compaction_text_search(&fixture.server, &scheduler_body_value(7)).await;
            let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
            assert_eq!(
                cold_sequence, sequence,
                "cold open must select the first publication"
            );
            let cold_server = TestServer::new(router(AppState::open(cold_engine)))
                .expect("scheduler cold server");
            assert_eq!(
                full_compaction_search_ids(
                    &cold_server,
                    full_compaction_keyword_query(&scheduler_keyword_value(7)),
                )
                .await,
                live_keyword,
                "live and cold Keyword searches must agree after the tied deepest publication"
            );
            let cold_text =
                full_compaction_text_search(&cold_server, &scheduler_body_value(7)).await;
            assert_eq!(
                cold_text["total"],
                live_text["total"],
                "live and cold Text totals must agree after the tied deepest publication"
            );
            assert_eq!(
                cold_text["hits"],
                live_text["hits"],
                "live and cold Text hits and BM25 scores must agree after the tied deepest publication"
            );

            // A later external checkpoint is the first point at which the
            // retained six-layer fields may be selected. Add one shallow
            // suffix so the checkpoint is a real durable change, then verify
            // the old three-layer identities remain alongside it.
            scheduler_update(&fixture.server, FULL_NUMBER, 4).await;
            sequence += 1;
            store
                .save(&fixture.engine, sequence)
                .expect("publish later external scheduler checkpoint");
            store
                .wait_for_merges(Duration::from_secs(30))
                .expect("later external scheduler checkpoint must drain its merge");
            let later_generation = stage1_current_generation_dir(&fixture.root);
            let later_manifest = stage1_read_manifest(&later_generation);
            assert_eq!(
                later_manifest["checkpoint_sequence"],
                json!(sequence),
                "the later external checkpoint must publish its own watermark"
            );
            assert_eq!(
                full_compaction_field_refs(&later_manifest, FULL_TEXT, "delta").len(),
                5,
                "the first tied deepest field may advance only after the external checkpoint"
            );
            assert_eq!(
                full_compaction_field_refs(&later_manifest, FULL_KEYWORD, "delta").len(),
                5,
                "the second tied deepest field may advance only after the external checkpoint"
            );
            assert_eq!(
                full_compaction_field_refs(&later_manifest, FULL_HASH, "delta").len(),
                5,
                "the retained six-layer field may advance only after the external checkpoint"
            );
            let later_shallow = full_compaction_field_refs(&later_manifest, FULL_NUMBER, "delta");
            assert_eq!(
                later_shallow.len(),
                4,
                "the external checkpoint must retain the new shallow suffix"
            );
            for reference in &before_shallow {
                let ordinal = reference["ordinal"]
                    .as_u64()
                    .expect("original shallow ordinal");
                let retained = scheduler_reference_by_ordinal(&later_shallow, ordinal);
                assert_eq!(
                    retained["payload_sha256"], reference["payload_sha256"],
                    "the original three-layer shallow ordinal {ordinal} must remain retained"
                );
            }
        }
    }

    mod collection_batch_contract {
        //! A merge job is collection-scoped. When several fields in the
        //! selected collection are eligible at the same durable cut, one
        //! publication must compact each of those fields. A collection with
        //! fewer than four delta layers must remain untouched, and the
        //! resulting catalog must still serve the same public values after a
        //! cold open.

        use super::*;
        use lumen::segment_rdb::{MergeObserver, MergePhase};
        use std::io;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc;

        const HOT_COLLECTION: &str = "merge-batch-hot";
        const IDLE_COLLECTION: &str = "merge-batch-idle";
        const HOT_FIELDS: [&str; 14] = [
            "h00", "h01", "h02", "h03", "h04", "h05", "h06", "h07", "h08", "h09", "h10", "h11",
            "h12", "h13",
        ];
        const FIRST_COHORT_FIELDS: [&str; 6] = ["h00", "h01", "h02", "h03", "h04", "h05"];
        const SECOND_COHORT_FIELDS: [&str; 8] =
            ["h06", "h07", "h08", "h09", "h10", "h11", "h12", "h13"];
        const IDLE_FIELD: &str = "idle";
        const BASE_ROWS: usize = 512;
        const BASE_SEQUENCE: u64 = 15_200;

        struct PauseFirstMergeObserver {
            before_publish: mpsc::SyncSender<()>,
            before_release: Mutex<Option<mpsc::Receiver<()>>>,
            after_publish: mpsc::SyncSender<()>,
            after_release: Mutex<Option<mpsc::Receiver<()>>>,
            before_paused: AtomicBool,
            after_paused: AtomicBool,
        }

        impl MergeObserver for PauseFirstMergeObserver {
            fn observe(&self, phase: MergePhase) -> io::Result<()> {
                let (reached, release, paused) = match phase {
                    MergePhase::BeforePublish => (
                        &self.before_publish,
                        &self.before_release,
                        &self.before_paused,
                    ),
                    MergePhase::AfterPublish => {
                        (&self.after_publish, &self.after_release, &self.after_paused)
                    }
                    MergePhase::BeforeEncode => return Ok(()),
                };
                if paused
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    return Ok(());
                }
                reached.send(()).map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "collection batch observer receiver dropped",
                    )
                })?;
                release
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                    .expect("collection batch merge release receiver")
                    .recv()
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "collection batch merge release sender dropped",
                        )
                    })?;
                Ok(())
            }
        }

        struct MergeRelease {
            before: Option<mpsc::Sender<()>>,
            after: Option<mpsc::Sender<()>>,
        }

        impl MergeRelease {
            fn release_before(&mut self) {
                if let Some(sender) = self.before.take() {
                    let _ = sender.send(());
                }
            }

            fn release_after(&mut self) {
                if let Some(sender) = self.after.take() {
                    let _ = sender.send(());
                }
            }
        }

        impl Drop for MergeRelease {
            fn drop(&mut self) {
                self.release_before();
                self.release_after();
            }
        }

        async fn create_keyword_collection(server: &TestServer, collection: &str, fields: &[&str]) {
            let mut schema = Map::new();
            for field in fields {
                schema.insert((*field).to_owned(), json!({ "type": "keyword" }));
            }
            server
                .put(&format!("/collections/{collection}"))
                .json(&json!({ "fields": schema }))
                .await
                .assert_status_ok();
        }

        async fn index_keyword(
            server: &TestServer,
            collection: &str,
            external_id: &str,
            field: &str,
            value: &str,
        ) {
            index_keywords(
                server,
                collection,
                vec![json!({
                    "external_id": external_id,
                    "field": field,
                    "value": value,
                })],
            )
            .await;
        }

        async fn index_keywords(server: &TestServer, collection: &str, items: Vec<Value>) {
            for chunk in items.chunks(1_000) {
                server
                    .post(&format!("/collections/{collection}/index"))
                    .json(&json!({ "items": chunk }))
                    .await
                    .assert_status_ok();
            }
        }

        fn keyword_item(external_id: String, field: &str, value: String) -> Value {
            json!({
                "external_id": external_id,
                "field": field,
                "value": value,
            })
        }

        async fn index_base_field(
            server: &TestServer,
            collection: &str,
            field: &str,
            field_index: usize,
        ) {
            let mut items = Vec::with_capacity(BASE_ROWS + 1);
            items.push(keyword_item(
                format!("hot-{field}-mutable"),
                field,
                format!("{field}-base"),
            ));
            for row in 0..BASE_ROWS {
                items.push(keyword_item(
                    format!("hot-{field}-base-{row:03}"),
                    field,
                    full_compaction_entropy_term(
                        2_500_000 + (field_index * BASE_ROWS + row) as u64,
                        8,
                    ),
                ));
            }
            index_keywords(server, collection, items).await;
        }

        async fn index_idle_base(server: &TestServer) {
            let mut items = Vec::with_capacity(BASE_ROWS + 1);
            items.push(keyword_item(
                "idle-mutable".to_owned(),
                IDLE_FIELD,
                "idle-base".to_owned(),
            ));
            for row in 0..BASE_ROWS {
                items.push(keyword_item(
                    format!("idle-base-{row:03}"),
                    IDLE_FIELD,
                    full_compaction_entropy_term(2_600_000 + row as u64, 8),
                ));
            }
            index_keywords(server, IDLE_COLLECTION, items).await;
        }

        async fn batch_fixture() -> (tempfile::TempDir, PathBuf, Arc<Engine>, TestServer) {
            let dir = tempfile::tempdir().expect("collection batch fixture root");
            let root = dir.path().join("segments");
            let initial_store = SegmentRdbStore::new(&root).expect("create collection batch store");
            let engine = Arc::new(Engine::new());
            let server = TestServer::new(router(AppState::open(engine.clone())))
                .expect("collection batch HTTP server");
            create_keyword_collection(&server, HOT_COLLECTION, &HOT_FIELDS).await;
            create_keyword_collection(&server, IDLE_COLLECTION, &[IDLE_FIELD]).await;

            for (field_index, field) in HOT_FIELDS.iter().enumerate() {
                index_base_field(&server, HOT_COLLECTION, field, field_index).await;
            }
            index_idle_base(&server).await;

            stage1_restore_legacy_base(&engine, "collection batch base");
            initial_store
                .save(&engine, BASE_SEQUENCE)
                .expect("publish collection batch base");
            drop(initial_store);
            (dir, root, engine, server)
        }

        fn batch_collection<'a>(manifest: &'a Value, collection: &str) -> &'a Value {
            manifest["collections"]
                .as_array()
                .expect("collection batch catalog collections")
                .iter()
                .find(|entry| entry["collection_id"] == json!(collection))
                .unwrap_or_else(|| panic!("collection batch catalog needs {collection}"))
        }

        fn batch_delta_refs<'a>(
            manifest: &'a Value,
            collection: &str,
            field: &str,
        ) -> Vec<&'a Value> {
            let mut refs: Vec<_> = batch_collection(manifest, collection)["segments"]
                .as_array()
                .expect("collection batch catalog segments")
                .iter()
                .filter(|segment| {
                    segment["role"] == json!("field")
                        && segment["kind"] == json!("delta")
                        && segment["field"] == json!(field)
                })
                .collect();
            refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("delta ordinal"));
            refs
        }

        fn batch_segment_bytes(generation: &Path, segment: &Value) -> u64 {
            let payload = std::fs::metadata(
                generation.join(segment["path"].as_str().expect("batch segment path")),
            )
            .expect("inspect batch segment")
            .len();
            let rows = segment
                .get("local_rows")
                .and_then(Value::as_object)
                .map(|local| {
                    std::fs::metadata(
                        generation.join(local["path"].as_str().expect("batch local rows path")),
                    )
                    .expect("inspect batch local rows")
                    .len()
                })
                .unwrap_or(0);
            payload
                .checked_add(rows)
                .expect("collection batch segment byte count does not overflow")
        }

        fn batch_base_bytes(
            generation: &Path,
            manifest: &Value,
            collection: &str,
            field: &str,
        ) -> u64 {
            let base = batch_collection(manifest, collection)["segments"]
                .as_array()
                .expect("collection batch catalog segments")
                .iter()
                .find(|segment| {
                    segment["role"] == json!("field")
                        && segment["kind"] == json!("base")
                        && segment["field"] == json!(field)
                        && segment["ordinal"] == json!(0)
                })
                .unwrap_or_else(|| {
                    panic!("collection batch base missing for {collection}/{field}")
                });
            batch_segment_bytes(generation, base)
        }

        fn batch_payload_bytes(
            generation: &Path,
            manifest: &Value,
            collection: &str,
            field: &str,
        ) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
            batch_delta_refs(manifest, collection, field)
                .into_iter()
                .map(|segment| {
                    let payload = std::fs::read(
                        generation.join(segment["path"].as_str().expect("batch payload path")),
                    )
                    .expect("read batch payload");
                    let local_rows =
                        segment
                            .get("local_rows")
                            .and_then(Value::as_object)
                            .map(|local| {
                                std::fs::read(
                                    generation.join(
                                        local["path"].as_str().expect("batch local rows path"),
                                    ),
                                )
                                .expect("read batch local rows")
                            });
                    (payload, local_rows)
                })
                .collect()
        }

        async fn term_ids(
            server: &TestServer,
            collection: &str,
            field: &str,
            value: &str,
        ) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{collection}/search"))
                .json(&json!({
                    "query": { "term": { "field": field, "value": value } },
                    "limit": 128,
                }))
                .await;
            response.assert_status_ok();
            let body: Value = response.json();
            let mut actual: Vec<_> = body["hits"]
                .as_array()
                .expect("collection batch search hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("collection batch hit ID")
                        .to_owned()
                })
                .collect();
            actual.sort_unstable();
            actual
        }

        async fn assert_live_and_cold_term_ids(
            live: &TestServer,
            cold: &TestServer,
            collection: &str,
            field: &str,
            value: &str,
            expected: &[&str],
        ) {
            let live_ids = term_ids(live, collection, field, value).await;
            let cold_ids = term_ids(cold, collection, field, value).await;
            assert_eq!(
                &live_ids, &cold_ids,
                "live and cold public query results for {collection}/{field}/{value}"
            );
            let mut expected = expected
                .iter()
                .map(|id| (*id).to_owned())
                .collect::<Vec<_>>();
            expected.sort_unstable();
            assert_eq!(
                live_ids, expected,
                "public query for {collection}/{field}/{value}"
            );
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn one_collection_merge_publication_reduces_all_eligible_fields_by_one_pair() {
            let (_dir, root, engine, server) = batch_fixture().await;
            let (before_publish_tx, before_publish_rx) = mpsc::sync_channel(1);
            let (before_release_tx, before_release_rx) = mpsc::channel();
            let (after_publish_tx, after_publish_rx) = mpsc::sync_channel(1);
            let (after_release_tx, after_release_rx) = mpsc::channel();
            let mut release = MergeRelease {
                before: Some(before_release_tx),
                after: Some(after_release_tx),
            };
            let observer = Arc::new(PauseFirstMergeObserver {
                before_publish: before_publish_tx,
                before_release: Mutex::new(Some(before_release_rx)),
                after_publish: after_publish_tx,
                after_release: Mutex::new(Some(after_release_rx)),
                before_paused: AtomicBool::new(false),
                after_paused: AtomicBool::new(false),
            });
            let store = SegmentRdbStore::with_merge_observer(&root, observer)
                .expect("open observed collection batch store");

            // The fourth cut makes only the first six hot fields eligible.
            // The other eight retain their segment identity for a later
            // explicit checkpoint.
            for round in 1..=4u64 {
                for field in HOT_FIELDS {
                    if round == 4 && SECOND_COHORT_FIELDS.contains(&field) {
                        continue;
                    }
                    index_keyword(
                        &server,
                        HOT_COLLECTION,
                        &format!("hot-{field}-mutable"),
                        field,
                        &format!("{field}-v{round}"),
                    )
                    .await;
                }
                if round <= 3 {
                    index_keyword(
                        &server,
                        IDLE_COLLECTION,
                        "idle-mutable",
                        IDLE_FIELD,
                        &format!("idle-v{round}"),
                    )
                    .await;
                }
                store
                    .save(&engine, BASE_SEQUENCE + round)
                    .expect("publish collection batch delta cut");
            }

            assert_eq!(
                before_publish_rx.recv_timeout(Duration::from_secs(30)),
                Ok(()),
                "merge worker must finish encoding before the first publication"
            );
            let before_generation = stage1_current_generation_dir(&root);
            let before_manifest = stage1_read_manifest(&before_generation);
            assert_eq!(
                before_manifest["checkpoint_sequence"],
                json!(BASE_SEQUENCE + 4),
                "merge must observe the complete multi-field cut"
            );
            for field in FIRST_COHORT_FIELDS {
                let refs = batch_delta_refs(&before_manifest, HOT_COLLECTION, field);
                assert_eq!(refs.len(), 4, "hot field {field} must have four deltas");
                let delta_bytes: u64 = refs
                    .iter()
                    .map(|segment| batch_segment_bytes(&before_generation, segment))
                    .sum();
                let base_bytes =
                    batch_base_bytes(&before_generation, &before_manifest, HOT_COLLECTION, field);
                assert!(
                    delta_bytes < base_bytes,
                    "hot field {field} must exercise pair compaction below base size: delta={delta_bytes}, base={base_bytes}"
                );
            }
            let second_before = SECOND_COHORT_FIELDS
                .iter()
                .map(|field| {
                    let refs = batch_delta_refs(&before_manifest, HOT_COLLECTION, field);
                    assert_eq!(
                        refs.len(),
                        3,
                        "deferred hot field {field} must have three deltas"
                    );
                    (
                        *field,
                        batch_payload_bytes(
                            &before_generation,
                            &before_manifest,
                            HOT_COLLECTION,
                            field,
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let idle_before = batch_delta_refs(&before_manifest, IDLE_COLLECTION, IDLE_FIELD);
            assert_eq!(
                idle_before.len(),
                3,
                "idle collection must remain below threshold"
            );
            let idle_payloads = batch_payload_bytes(
                &before_generation,
                &before_manifest,
                IDLE_COLLECTION,
                IDLE_FIELD,
            );
            let before_revision = before_manifest["revision"]
                .as_u64()
                .expect("collection batch revision");

            release.release_before();
            assert_eq!(
                after_publish_rx.recv_timeout(Duration::from_secs(30)),
                Ok(()),
                "the first merge publication must be observable before a follow-up job"
            );
            store
                .wait_for_merges(Duration::from_secs(1))
                .expect_err("the observer must hold the first published merge");
            let after_generation = stage1_current_generation_dir(&root);
            let after_manifest = stage1_read_manifest(&after_generation);
            assert_eq!(
                after_manifest["revision"],
                json!(before_revision + 1),
                "all eligible fields must be published in one generation"
            );
            for field in FIRST_COHORT_FIELDS {
                assert_eq!(
                    batch_delta_refs(&after_manifest, HOT_COLLECTION, field).len(),
                    3,
                    "one publication must reduce every eligible hot field by one pair: {field}"
                );
            }
            for field in SECOND_COHORT_FIELDS {
                assert_eq!(
                    batch_payload_bytes(&after_generation, &after_manifest, HOT_COLLECTION, field),
                    second_before[field],
                    "deferred hot field {field} must retain its segment identity"
                );
            }
            assert_eq!(
                batch_payload_bytes(
                    &after_generation,
                    &after_manifest,
                    IDLE_COLLECTION,
                    IDLE_FIELD
                ),
                idle_payloads,
                "the shallower idle collection must remain unchanged"
            );

            let cold = store
                .load_current_generation()
                .expect("load collection batch CURRENT")
                .expect("collection batch CURRENT generation");
            let cold_server = TestServer::new(router(AppState::open(cold.engine)))
                .expect("collection batch cold HTTP server");
            for field in FIRST_COHORT_FIELDS {
                assert_live_and_cold_term_ids(
                    &server,
                    &cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v4"),
                    &[&format!("hot-{field}-mutable")],
                )
                .await;
                assert_live_and_cold_term_ids(
                    &server,
                    &cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v3"),
                    &[],
                )
                .await;
            }
            for field in SECOND_COHORT_FIELDS {
                assert_live_and_cold_term_ids(
                    &server,
                    &cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v3"),
                    &[&format!("hot-{field}-mutable")],
                )
                .await;
                assert_live_and_cold_term_ids(
                    &server,
                    &cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v2"),
                    &[],
                )
                .await;
            }
            assert_live_and_cold_term_ids(
                &server,
                &cold_server,
                IDLE_COLLECTION,
                IDLE_FIELD,
                "idle-v3",
                &["idle-mutable"],
            )
            .await;
            assert_live_and_cold_term_ids(
                &server,
                &cold_server,
                IDLE_COLLECTION,
                IDLE_FIELD,
                "idle-v2",
                &[],
            )
            .await;

            release.release_after();
            store
                .wait_for_merges(Duration::from_secs(30))
                .expect("first collection batch merge must finish after inspection");

            // The idle worker does not schedule a successor by itself. A new
            // explicit durable cut makes the deferred eight-field cohort
            // eligible and publishes it in a later generation.
            for field in SECOND_COHORT_FIELDS {
                index_keyword(
                    &server,
                    HOT_COLLECTION,
                    &format!("hot-{field}-mutable"),
                    field,
                    &format!("{field}-v4"),
                )
                .await;
            }
            store
                .save(&engine, BASE_SEQUENCE + 5)
                .expect("publish explicit deferred cohort checkpoint");
            store
                .wait_for_merges(Duration::from_secs(30))
                .expect("explicit deferred cohort merge must finish");
            let final_generation = stage1_current_generation_dir(&root);
            let final_manifest = stage1_read_manifest(&final_generation);
            for field in SECOND_COHORT_FIELDS {
                assert_eq!(
                    batch_delta_refs(&final_manifest, HOT_COLLECTION, field).len(),
                    3,
                    "the explicit later checkpoint must reduce deferred field {field} by one pair"
                );
            }
            let final_cold = store
                .load_current_generation()
                .expect("load final collection batch CURRENT")
                .expect("final collection batch CURRENT generation");
            let final_cold_server = TestServer::new(router(AppState::open(final_cold.engine)))
                .expect("final collection batch cold HTTP server");
            for field in SECOND_COHORT_FIELDS {
                assert_live_and_cold_term_ids(
                    &server,
                    &final_cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v4"),
                    &[&format!("hot-{field}-mutable")],
                )
                .await;
                assert_live_and_cold_term_ids(
                    &server,
                    &final_cold_server,
                    HOT_COLLECTION,
                    field,
                    &format!("{field}-v3"),
                    &[],
                )
                .await;
            }
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
        async fn metrics_and_search_accept_requests_after_checkpoint_merge_drain() {
            let (_dir, root, engine, server) = batch_fixture().await;
            let store = SegmentRdbStore::new(&root).expect("open short drain store");

            for round in 1..=4u64 {
                index_keyword(
                    &server,
                    HOT_COLLECTION,
                    "hot-h00-mutable",
                    "h00",
                    &format!("h00-drain-v{round}"),
                )
                .await;
                store
                    .save(&engine, BASE_SEQUENCE + round)
                    .expect("publish short drain checkpoint");
            }
            store
                .wait_for_merges(Duration::from_secs(30))
                .expect("short deterministic checkpoint and merge drain");

            let metrics = tokio::time::timeout(Duration::from_secs(1), server.get("/metrics"))
                .await
                .expect("GET /metrics must finish within one second after merge drain");
            metrics.assert_status_ok();

            tokio::time::timeout(
                Duration::from_secs(1),
                term_ids(&server, HOT_COLLECTION, "h00", "h00-drain-v4"),
            )
            .await
            .expect("a new search request must finish within one second after merge drain");
        }
    }

    #[cfg(unix)]
    mod vector_base_compaction_contract {
        //! # Facets
        //!
        //! - Behavior: `v2_flat_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained`
        //!   and `v2_hnsw_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained`
        //!   use public collection, index, replace, delete, search, and
        //!   `SegmentRdbStore::save` operations. They require a new mapped vector
        //!   base after four measured delta layers, then verify Flat and HNSW live,
        //!   cold, retained, absent, deleted, appended, newer-update, and
        //!   full-replace behavior.
        //! - Security: `v2_current_refuses_malformed_mapped_flat_vector_base_without_fallback`
        //!   corrupts the persisted mapped-base local-row descriptor and checksum.
        //!   It requires `load_current_generation` to refuse each input and leave
        //!   CURRENT unchanged. This covers the changed file-read boundary in
        //!   `apps/lumen/src/segment_rdb.rs:1625-1795`.
        //! - Performance: the user-approved #4246 plan requires a compaction
        //!   request at four deltas and base replacement only when measured delta
        //!   bytes meet or exceed the complete base. These cases assert both
        //!   conditions and hard-link reuse. They do not claim a graph-rebuild,
        //!   latency, or RSS result because no public probe measures those paths.

        use super::*;
        use sha2::{Digest, Sha256};

        const VECTOR_BASE_COLLECTION: &str = "vector-base-compaction";
        const VECTOR_BASE_KIND: &str = "kind";
        const VECTOR_BASE_FIELD: &str = "embedding";
        const VECTOR_BASE_ROWS: usize = 96;
        const VECTOR_BASE_SEQUENCE: u64 = 13_100;
        const VECTOR_BASE_HOT_ID: &str = "vector-base-hot";
        const VECTOR_BASE_DELETED_ID: &str = "vector-base-deleted";
        const VECTOR_BASE_ABSENT_ID: &str = "vector-base-absent";
        const VECTOR_BASE_APPENDED_ID: &str = "vector-base-appended";

        struct VectorBaseFixture {
            _dir: tempfile::TempDir,
            root: PathBuf,
            store: SegmentRdbStore,
            engine: Arc<Engine>,
            server: TestServer,
            base_name: String,
        }

        fn vector_base_id(index: usize) -> String {
            format!("vector-base-{index:03}")
        }

        fn vector_base_vector(x: f32) -> Value {
            // The earlier `[x, -x]` fixture put every L2 point on one line.
            // That makes an approximate HNSW walk depend on insertion order.
            // This odd-multiplier permutation gives every integral fixture key
            // a unique, deterministic two-coordinate point. The query uses
            // the same point, so its expected ID has distance zero while every
            // other fixture ID has a strictly positive L2 distance.
            let key = x as u32;
            assert_eq!(key as f32, x, "vector fixture keys must be integral");
            let mixed = key.wrapping_mul(0x9e37_79b1).wrapping_add(0x7f4a_7c15);
            json!([(mixed & 0xffff) as f32, (mixed >> 16) as f32])
        }

        fn vector_base_initial_x(index: usize) -> f32 {
            1_000.0 + index as f32
        }

        fn vector_base_round_x(round: usize, index: usize) -> f32 {
            10_000.0 * round as f32 + index as f32
        }

        fn vector_base_hot_x(round: usize) -> f32 {
            100_000.0 + round as f32
        }

        fn vector_base_appended_x(round: usize) -> f32 {
            200_000.0 + round as f32
        }

        async fn vector_base_create_collection(server: &TestServer, backend: &str) {
            server
                .put(&format!("/collections/{VECTOR_BASE_COLLECTION}"))
                .json(&json!({ "fields": {
                    VECTOR_BASE_KIND: { "type": "keyword" },
                    VECTOR_BASE_FIELD: {
                        "type": "vector", "dim": 2, "metric": "l2", "backend": backend,
                    },
                }}))
                .await
                .assert_status_ok();
        }

        async fn vector_base_post_items(server: &TestServer, items: Vec<Value>) {
            for chunk in items.chunks(1_000) {
                server
                    .post(&format!("/collections/{VECTOR_BASE_COLLECTION}/index"))
                    .json(&json!({ "items": chunk }))
                    .await
                    .assert_status_ok();
            }
        }

        fn vector_base_initial_items() -> Vec<Value> {
            let mut items = Vec::with_capacity((VECTOR_BASE_ROWS + 3) * 2);
            for index in 0..VECTOR_BASE_ROWS {
                let id = vector_base_id(index);
                items.push(json!({
                    "external_id": id,
                    "field": VECTOR_BASE_KIND,
                    "value": format!("base-kind-{index:03}"),
                }));
                items.push(json!({
                    "external_id": vector_base_id(index),
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_initial_x(index)),
                }));
            }
            items.extend([
                json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "hot-base",
                }),
                json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_hot_x(0)),
                }),
                json!({
                    "external_id": VECTOR_BASE_DELETED_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "deleted-base",
                }),
                json!({
                    "external_id": VECTOR_BASE_DELETED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(500.0),
                }),
                // This document is in the collection EID metadata but never has a
                // vector. A mapped vector base must not make it searchable.
                json!({
                    "external_id": VECTOR_BASE_ABSENT_ID,
                    "field": VECTOR_BASE_KIND,
                    "value": "absent-base-vector",
                }),
            ]);
            items
        }

        async fn vector_base_fixture(backend: &str) -> VectorBaseFixture {
            let dir = tempfile::tempdir().expect("vector base compaction fixture root");
            let root = dir.path().join("segments");
            let store = SegmentRdbStore::new(&root).expect("create vector base store");
            let engine = Arc::new(Engine::new());
            let server = TestServer::new(router(AppState::open(engine.clone())))
                .expect("vector base compaction HTTP server");
            vector_base_create_collection(&server, backend).await;
            vector_base_post_items(&server, vector_base_initial_items()).await;
            stage1_restore_legacy_base(&engine, "vector base compaction fixture");
            store
                .save(&engine, VECTOR_BASE_SEQUENCE)
                .expect("publish vector base generation");
            VectorBaseFixture {
                _dir: dir,
                root: root.clone(),
                store,
                engine,
                server,
                base_name: stage1_reuse_current_name(&root),
            }
        }

        async fn vector_base_apply_round(server: &TestServer, round: usize) {
            assert!(
                (1..=4).contains(&round),
                "vector base contract has four rounds"
            );
            let mut items = Vec::with_capacity(VECTOR_BASE_ROWS + 2);
            for index in 0..VECTOR_BASE_ROWS {
                items.push(json!({
                    "external_id": vector_base_id(index),
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_round_x(round, index)),
                }));
            }
            if round < 4 {
                items.push(json!({
                    "external_id": VECTOR_BASE_HOT_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_hot_x(round)),
                }));
            }
            if round > 1 {
                items.push(json!({
                    "external_id": VECTOR_BASE_APPENDED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_appended_x(round)),
                }));
            }
            vector_base_post_items(server, items).await;

            if round == 1 {
                server
                    .delete(&format!(
                        "/collections/{VECTOR_BASE_COLLECTION}/index/{VECTOR_BASE_DELETED_ID}"
                    ))
                    .await
                    .assert_status(axum::http::StatusCode::NO_CONTENT);
                vector_base_post_items(
                    server,
                    vec![
                        json!({
                            "external_id": VECTOR_BASE_APPENDED_ID,
                            "field": VECTOR_BASE_KIND,
                            "value": "appended-after-base",
                        }),
                        json!({
                            "external_id": VECTOR_BASE_APPENDED_ID,
                            "field": VECTOR_BASE_FIELD,
                            "value": vector_base_vector(vector_base_appended_x(round)),
                        }),
                    ],
                )
                .await;
            }
            if round == 4 {
                server
                    .put(&format!(
                        "/collections/{VECTOR_BASE_COLLECTION}/docs:replace"
                    ))
                    .json(&json!({ "docs": [{
                        "external_id": VECTOR_BASE_HOT_ID,
                        "fields": {
                            VECTOR_BASE_KIND: "hot-full-replace",
                            VECTOR_BASE_FIELD: vector_base_vector(vector_base_hot_x(round)),
                        },
                    }]}))
                    .await
                    .assert_status_ok();
            }
        }

        async fn vector_base_apply_newer_update(server: &TestServer) {
            vector_base_post_items(
                server,
                vec![json!({
                    "external_id": VECTOR_BASE_APPENDED_ID,
                    "field": VECTOR_BASE_FIELD,
                    "value": vector_base_vector(vector_base_appended_x(5)),
                })],
            )
            .await;
        }

        fn vector_base_collection<'a>(manifest: &'a Value) -> &'a Value {
            stage1_reuse_catalog_collection(manifest, VECTOR_BASE_COLLECTION)
        }

        fn vector_base_field_refs<'a>(manifest: &'a Value, kind: &str) -> Vec<&'a Value> {
            let mut refs: Vec<_> = vector_base_collection(manifest)["segments"]
                .as_array()
                .expect("vector base catalog segments")
                .iter()
                .filter(|segment| {
                    segment["role"] == json!("field")
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!(kind)
                })
                .collect();
            refs.sort_by_key(|segment| segment["ordinal"].as_u64().expect("vector base ordinal"));
            refs
        }

        fn vector_base_ref<'a>(manifest: &'a Value, role: &str) -> &'a Value {
            vector_base_collection(manifest)["segments"]
                .as_array()
                .expect("vector base catalog segments")
                .iter()
                .find(|segment| {
                    segment["role"] == json!(role)
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!("base")
                        && segment["ordinal"] == json!(0)
                })
                .unwrap_or_else(|| panic!("vector base catalog needs {role} base reference"))
        }

        fn vector_base_ref_mut<'a>(manifest: &'a mut Value, role: &str) -> &'a mut Value {
            manifest["collections"]
                .as_array_mut()
                .expect("vector base catalog collections")
                .iter_mut()
                .find(|collection| collection["collection_id"] == json!(VECTOR_BASE_COLLECTION))
                .expect("vector base catalog collection")["segments"]
                .as_array_mut()
                .expect("vector base catalog segments")
                .iter_mut()
                .find(|segment| {
                    segment["role"] == json!(role)
                        && segment["field"] == json!(VECTOR_BASE_FIELD)
                        && segment["kind"] == json!("base")
                        && segment["ordinal"] == json!(0)
                })
                .unwrap_or_else(|| {
                    panic!("vector base catalog needs mutable {role} base reference")
                })
        }

        fn vector_base_ref_bytes(generation: &Path, reference: &Value) -> u64 {
            let payload = std::fs::metadata(
                generation.join(
                    reference["path"]
                        .as_str()
                        .expect("vector base segment path"),
                ),
            )
            .expect("inspect vector base segment")
            .len();
            let rows = reference["local_rows"].as_object().map(|rows| {
                std::fs::metadata(
                    generation.join(rows["path"].as_str().expect("vector base local-row path")),
                )
                .expect("inspect vector base local-row map")
                .len()
            });
            rows.map_or(payload, |rows| {
                payload.checked_add(rows).expect("vector base byte sum")
            })
        }

        /// Reproduce the versioned base-payload framing so this test can replace a
        /// vector EID segment with independently valid bytes and a matching
        /// catalog checksum.  The forged sidecar therefore reaches the mapped-row
        /// consistency check instead of failing an earlier integrity check.
        fn vector_base_payload_sha256(path: &Path) -> String {
            let bytes = std::fs::read(path).expect("read forged vector EID sidecar for checksum");
            let mut hasher = Sha256::new();
            hasher.update(b"lumen.base.payload-sha256.v1\0");
            hasher.update((b"segment".len() as u64).to_be_bytes());
            hasher.update(b"segment");
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
            hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        }

        /// Build a second valid vector-EID base through the public Engine and
        /// SegmentRdbStore APIs. It has the compacted checkpoint sequence and
        /// count, while distinct stable IDs make its row mapping disagree with the
        /// original mapped-base local rows.
        async fn vector_base_forged_eid_sidecar(sequence: u64, ids: &[String]) -> Vec<u8> {
            let forge_dir = tempfile::tempdir().expect("forge vector EID sidecar directory");
            let forge_engine = Arc::new(Engine::new());
            let forge_server = TestServer::new(router(AppState::open(forge_engine.clone())))
                .expect("forge vector EID sidecar server");
            vector_base_create_collection(&forge_server, "flat-cpu").await;
            let items = ids
                .iter()
                .enumerate()
                .map(|(row, external_id)| {
                    json!({
                        "external_id": external_id,
                        "field": VECTOR_BASE_FIELD,
                        "value": vector_base_vector(1_000_000.0 + row as f32),
                    })
                })
                .collect();
            vector_base_post_items(&forge_server, items).await;
            let forge_store =
                SegmentRdbStore::new(forge_dir.path()).expect("forge vector EID store");
            forge_store
                .save_required(&forge_engine, sequence)
                .expect("save independently valid same-sequence vector EID base");
            let forge_generation = stage1_current_generation_dir(forge_dir.path());
            let forge_manifest = stage1_read_manifest(&forge_generation);
            let forge_eids = vector_base_ref(&forge_manifest, "vector_eids");
            std::fs::read(
                forge_generation.join(
                    forge_eids["path"]
                        .as_str()
                        .expect("forged vector EID sidecar path"),
                ),
            )
            .expect("read independently valid same-sequence vector EID sidecar")
        }

        fn vector_base_delta_bytes(generation: &Path, manifest: &Value) -> u64 {
            vector_base_field_refs(manifest, "delta")
                .into_iter()
                .map(|reference| vector_base_ref_bytes(generation, reference))
                .sum()
        }

        fn vector_base_complete_base_bytes(generation: &Path, manifest: &Value) -> u64 {
            ["field", "vector_eids"]
                .into_iter()
                .map(|role| vector_base_ref_bytes(generation, vector_base_ref(manifest, role)))
                .sum()
        }

        fn vector_base_mapped_rows_path(generation: &Path, reference: &Value) -> PathBuf {
            let local = reference["local_rows"]
                .as_object()
                .expect("compacted vector base has a local-row descriptor");
            generation.join(
                local["path"]
                    .as_str()
                    .expect("compacted vector base local-row path"),
            )
        }

        fn vector_base_assert_new_mapped_base(
            base_generation: &Path,
            base_manifest: &Value,
            compacted_generation: &Path,
            compacted_manifest: &Value,
        ) {
            let old_field = vector_base_ref(base_manifest, "field");
            let new_field = vector_base_ref(compacted_manifest, "field");
            let local = new_field["local_rows"]
                .as_object()
                .expect("vector base compaction must publish a mapped base");
            assert_eq!(
                local["format"],
                json!("lumen-local-eids-cbor-v1"),
                "mapped vector base must declare the versioned local-row codec"
            );
            let rows = stage1_keyword_delta_read_rows(compacted_generation, new_field);
            assert_eq!(
                local["count"].as_u64(),
                Some(rows.len() as u64),
                "mapped vector base row count must match the decoded local-row map"
            );
            assert!(
            rows.contains(&VECTOR_BASE_APPENDED_ID.to_owned()),
            "mapped vector base must include an ID added after the original collection EID metadata"
        );
            assert!(
                !rows.contains(&VECTOR_BASE_ABSENT_ID.to_owned()),
                "mapped vector base must not invent a row for a document without this vector field"
            );
            assert_eq!(
                new_field["payload_sha256"].as_str().map(str::len),
                Some(64),
                "mapped vector base must checksum its payload and local-row map"
            );

            for role in ["field", "vector_eids"] {
                let old = vector_base_ref(base_manifest, role);
                let new = vector_base_ref(compacted_manifest, role);
                let old_path =
                    base_generation.join(old["path"].as_str().expect("old vector base path"));
                let new_path = compacted_generation
                    .join(new["path"].as_str().expect("compacted vector base path"));
                let old_metadata =
                    std::fs::symlink_metadata(&old_path).expect("inspect old vector base");
                let new_metadata =
                    std::fs::symlink_metadata(&new_path).expect("inspect compacted vector base");
                assert!(old_metadata.is_file() && !old_metadata.file_type().is_symlink());
                assert!(new_metadata.is_file() && !new_metadata.file_type().is_symlink());
                assert_ne!(
                old_metadata.ino(),
                new_metadata.ino(),
                "mapped vector base compaction must rewrite {role} for new or masked vector IDs"
            );
            }
        }

        fn vector_base_assert_mapped_base_hardlinked(
            prior_generation: &Path,
            prior_manifest: &Value,
            latest_generation: &Path,
            latest_manifest: &Value,
        ) {
            let prior_field = vector_base_ref(prior_manifest, "field");
            let latest_field = vector_base_ref(latest_manifest, "field");
            let paths = [
                (
                    prior_generation.join(
                        prior_field["path"]
                            .as_str()
                            .expect("prior mapped vector payload"),
                    ),
                    latest_generation.join(
                        latest_field["path"]
                            .as_str()
                            .expect("latest mapped vector payload"),
                    ),
                    "mapped vector payload",
                ),
                (
                    vector_base_mapped_rows_path(prior_generation, prior_field),
                    vector_base_mapped_rows_path(latest_generation, latest_field),
                    "mapped vector local-row map",
                ),
                (
                    prior_generation.join(
                        vector_base_ref(prior_manifest, "vector_eids")["path"]
                            .as_str()
                            .expect("prior vector EID sidecar"),
                    ),
                    latest_generation.join(
                        vector_base_ref(latest_manifest, "vector_eids")["path"]
                            .as_str()
                            .expect("latest vector EID sidecar"),
                    ),
                    "vector EID sidecar",
                ),
            ];
            for (prior, latest, label) in paths {
                let old =
                    std::fs::symlink_metadata(&prior).expect("inspect prior mapped base file");
                let new =
                    std::fs::symlink_metadata(&latest).expect("inspect latest mapped base file");
                assert!(old.is_file() && !old.file_type().is_symlink());
                assert!(new.is_file() && !new.file_type().is_symlink());
                assert_eq!(
                    old.ino(),
                    new.ino(),
                    "unchanged {label} must be hard linked into the next generation"
                );
                assert!(
                    new.nlink() >= 2,
                    "hard-linked {label} must retain multiple links"
                );
            }
        }

        fn vector_base_live_ids() -> std::collections::BTreeSet<String> {
            (0..VECTOR_BASE_ROWS)
                .map(vector_base_id)
                .chain(std::iter::once(VECTOR_BASE_HOT_ID.to_owned()))
                .chain(std::iter::once(VECTOR_BASE_DELETED_ID.to_owned()))
                .collect()
        }

        fn vector_base_current_ids() -> std::collections::BTreeSet<String> {
            let mut ids = vector_base_live_ids();
            ids.remove(VECTOR_BASE_DELETED_ID);
            ids.insert(VECTOR_BASE_APPENDED_ID.to_owned());
            ids
        }

        async fn vector_base_knn_ids(server: &TestServer, x: f32, k: usize) -> Vec<String> {
            let response = server
                .post(&format!("/collections/{VECTOR_BASE_COLLECTION}/search"))
                .json(&json!({
                    "query": { "knn": {
                        "field": VECTOR_BASE_FIELD,
                        "vector": vector_base_vector(x),
                        "k": k,
                    }},
                    "limit": k,
                    "track_total": true,
                }))
                .await;
            response.assert_status_ok();
            let body: Value = response.json();
            body["hits"]
                .as_array()
                .expect("vector base kNN hits")
                .iter()
                .map(|hit| {
                    hit["external_id"]
                        .as_str()
                        .expect("vector base kNN external ID")
                        .to_owned()
                })
                .collect()
        }

        async fn vector_base_assert_backend(server: &TestServer, backend: &str, phase: &str) {
            let response = server.get("/admin/backup").await;
            response.assert_status_ok();
            let snapshot: Value = response.json();
            assert_eq!(
                snapshot["collections"][VECTOR_BASE_COLLECTION]["fields"][VECTOR_BASE_FIELD]
                    ["spec"]["backend"],
                json!(backend),
                "{phase}: vector backend schema must survive live and cold states"
            );
        }

        async fn vector_base_assert_ids(
            server: &TestServer,
            expected: std::collections::BTreeSet<String>,
            phase: &str,
        ) {
            let ids = vector_base_knn_ids(server, 0.0, 256).await;
            let unique: std::collections::BTreeSet<_> = ids.iter().cloned().collect();
            assert_eq!(
                unique.len(),
                ids.len(),
                "{phase}: vector search must not duplicate external IDs"
            );
            assert_eq!(
                unique, expected,
                "{phase}: vector search must expose exact live IDs"
            );
            assert!(
                !ids.iter().any(|id| id == VECTOR_BASE_ABSENT_ID),
                "{phase}: a document without a base vector must not become searchable"
            );
        }

        async fn vector_base_assert_nearest(
            server: &TestServer,
            x: f32,
            expected: &str,
            phase: &str,
        ) {
            assert_eq!(
                vector_base_knn_ids(server, x, 1).await,
                vec![expected.to_owned()],
                "{phase}: exact vector query must return the newest expected external ID"
            );
        }

        async fn vector_base_assert_base_state(server: &TestServer, backend: &str, phase: &str) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_live_ids(), phase).await;
            vector_base_assert_nearest(server, vector_base_initial_x(0), &vector_base_id(0), phase)
                .await;
            vector_base_assert_nearest(server, vector_base_hot_x(0), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(server, 500.0, VECTOR_BASE_DELETED_ID, phase).await;
        }

        async fn vector_base_assert_round_state(
            server: &TestServer,
            backend: &str,
            round: usize,
            phase: &str,
        ) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_current_ids(), phase).await;
            assert!(
                !vector_base_knn_ids(server, 0.0, 256)
                    .await
                    .iter()
                    .any(|id| id == VECTOR_BASE_DELETED_ID),
                "{phase}: a deleted base vector must stay masked"
            );
            vector_base_assert_nearest(
                server,
                vector_base_round_x(round, 0),
                &vector_base_id(0),
                phase,
            )
            .await;
            vector_base_assert_nearest(
                server,
                vector_base_round_x(round, VECTOR_BASE_ROWS - 1),
                &vector_base_id(VECTOR_BASE_ROWS - 1),
                phase,
            )
            .await;
            vector_base_assert_nearest(server, vector_base_hot_x(round), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(
                server,
                vector_base_appended_x(round),
                VECTOR_BASE_APPENDED_ID,
                phase,
            )
            .await;
        }

        async fn vector_base_assert_newer_state(server: &TestServer, backend: &str, phase: &str) {
            vector_base_assert_backend(server, backend, phase).await;
            vector_base_assert_ids(server, vector_base_current_ids(), phase).await;
            vector_base_assert_nearest(
                server,
                vector_base_round_x(4, 0),
                &vector_base_id(0),
                phase,
            )
            .await;
            vector_base_assert_nearest(server, vector_base_hot_x(4), VECTOR_BASE_HOT_ID, phase)
                .await;
            vector_base_assert_nearest(
                server,
                vector_base_appended_x(5),
                VECTOR_BASE_APPENDED_ID,
                phase,
            )
            .await;
        }

        async fn vector_base_publish_four_checkpoints(
            fixture: &VectorBaseFixture,
        ) -> (String, String) {
            let base_generation = fixture.root.join(&fixture.base_name);
            let base_manifest = stage1_read_manifest(&base_generation);
            for round in 1..=3 {
                vector_base_apply_round(&fixture.server, round).await;
                fixture
                    .store
                    .save(&fixture.engine, VECTOR_BASE_SEQUENCE + round as u64)
                    .expect("publish vector base-eligible delta checkpoint");
            }
            let third_name = stage1_reuse_current_name(&fixture.root);
            let third_generation = fixture.root.join(&third_name);
            let third_manifest = stage1_read_manifest(&third_generation);
            let third_deltas = vector_base_field_refs(&third_manifest, "delta");
            assert_eq!(
                third_deltas.len(),
                3,
                "the fourth checkpoint alone must request vector base compaction"
            );
            stage1_assert_delta_sequence_order(
                &third_deltas,
                &[
                    VECTOR_BASE_SEQUENCE + 1,
                    VECTOR_BASE_SEQUENCE + 2,
                    VECTOR_BASE_SEQUENCE + 3,
                ],
                "vector base pre-merge layers",
            );
            let base_bytes = vector_base_complete_base_bytes(&base_generation, &base_manifest);
            let delta_bytes = vector_base_delta_bytes(&third_generation, &third_manifest);
            assert!(
            delta_bytes >= base_bytes,
            "fixture precondition: three actual vector delta-plus-row-map bytes ({delta_bytes}) must meet or exceed the complete vector base-plus-EID-sidecar bytes ({base_bytes}) before the fourth checkpoint"
        );

            vector_base_apply_round(&fixture.server, 4).await;
            let publication = fixture
                .store
                .save(&fixture.engine, VECTOR_BASE_SEQUENCE + 4);
            assert!(
            publication.is_ok(),
            "base-eligible vector publication must preserve the deleted base-vector tombstone and publish a mapped base: {publication:?}"
        );
            fixture
                .store
                .wait_for_merges(Duration::from_secs(30))
                .expect("wait for base-eligible vector compaction");
            let compacted_name = stage1_reuse_current_name(&fixture.root);
            let compacted_generation = fixture.root.join(&compacted_name);
            let compacted_manifest = stage1_read_manifest(&compacted_generation);
            assert!(
                vector_base_field_refs(&compacted_manifest, "delta").is_empty(),
                "base-eligible vector publication must consume all four captured vector deltas"
            );
            vector_base_assert_new_mapped_base(
                &base_generation,
                &base_manifest,
                &compacted_generation,
                &compacted_manifest,
            );
            (third_name, compacted_name)
        }

        async fn vector_base_behavior_contract(backend: &str) {
            let fixture = vector_base_fixture(backend).await;
            vector_base_assert_base_state(&fixture.server, backend, "live original base").await;
            let (third_name, compacted_name) = vector_base_publish_four_checkpoints(&fixture).await;
            vector_base_assert_round_state(&fixture.server, backend, 4, "live mapped base").await;

            let (cold_engine, cold_sequence) = stage1_reuse_cold_load_current(&fixture.root);
            assert_eq!(
                cold_sequence,
                VECTOR_BASE_SEQUENCE + 4,
                "cold CURRENT carries the mapped-base checkpoint sequence"
            );
            let cold_server = TestServer::new(router(AppState::open(cold_engine.clone())))
                .expect("cold mapped vector base server");
            vector_base_assert_round_state(&cold_server, backend, 4, "cold mapped base").await;

            vector_base_apply_newer_update(&cold_server).await;
            SegmentRdbStore::new(&fixture.root)
                .expect("reopen store after cold mapped-base update")
                .save(&cold_engine, VECTOR_BASE_SEQUENCE + 5)
                .expect("publish newer vector delta above mapped base");
            let latest_generation = stage1_current_generation_dir(&fixture.root);
            let latest_manifest = stage1_read_manifest(&latest_generation);
            let latest_deltas = vector_base_field_refs(&latest_manifest, "delta");
            assert_eq!(
                latest_deltas.len(),
                1,
                "newer vector update remains a single delta above the mapped base"
            );
            stage1_assert_delta_sequence_order(
                &latest_deltas,
                &[VECTOR_BASE_SEQUENCE + 5],
                "newer vector delta above mapped base",
            );
            let compacted_generation = fixture.root.join(&compacted_name);
            let compacted_manifest = stage1_read_manifest(&compacted_generation);
            vector_base_assert_mapped_base_hardlinked(
                &compacted_generation,
                &compacted_manifest,
                &latest_generation,
                &latest_manifest,
            );
            vector_base_assert_newer_state(&cold_server, backend, "live newer mapped-base update")
                .await;

            let (latest_engine, latest_sequence) = stage1_reuse_cold_load_current(&fixture.root);
            assert_eq!(
                latest_sequence,
                VECTOR_BASE_SEQUENCE + 5,
                "cold latest generation carries the newer vector update"
            );
            let latest_server = TestServer::new(router(AppState::open(latest_engine)))
                .expect("cold newer mapped vector base server");
            vector_base_assert_newer_state(
                &latest_server,
                backend,
                "cold newer mapped-base update",
            )
            .await;

            let (third_engine, third_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &third_name);
            assert_eq!(
                third_sequence,
                VECTOR_BASE_SEQUENCE + 3,
                "retained third vector generation keeps its sequence"
            );
            let third_server = TestServer::new(router(AppState::open(third_engine)))
                .expect("retained third vector server");
            vector_base_assert_round_state(
                &third_server,
                backend,
                3,
                "retained third vector generation",
            )
            .await;

            let (compacted_engine, compacted_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &compacted_name);
            assert_eq!(
                compacted_sequence,
                VECTOR_BASE_SEQUENCE + 4,
                "retained mapped vector base keeps its sequence"
            );
            let compacted_server = TestServer::new(router(AppState::open(compacted_engine)))
                .expect("retained mapped vector base server");
            vector_base_assert_round_state(
                &compacted_server,
                backend,
                4,
                "retained mapped vector base",
            )
            .await;

            let (base_engine, base_sequence) =
                stage1_reuse_cold_load_named(&fixture.root, &fixture.base_name);
            assert_eq!(
                base_sequence, VECTOR_BASE_SEQUENCE,
                "retained original vector base keeps its sequence"
            );
            let base_server = TestServer::new(router(AppState::open(base_engine)))
                .expect("retained original vector base server");
            vector_base_assert_base_state(&base_server, backend, "retained original vector base")
                .await;
        }

        fn vector_base_assert_current_refuses_mapped_base(root: &Path, mutation: &str) {
            let current_before = std::fs::read(root.join("CURRENT"))
                .expect("read CURRENT before mapped-base refusal");
            let opened = SegmentRdbStore::new(root)
                .expect("reopen mapped-base checkpoint root")
                .load_current_generation();
            assert!(
            opened.is_err(),
            "CURRENT must explicitly refuse {mutation}, not open a predecessor or malformed mapped base"
        );
            assert_eq!(
                std::fs::read(root.join("CURRENT"))
                    .expect("read CURRENT after mapped-base refusal"),
                current_before,
                "refusing {mutation} must leave CURRENT unchanged"
            );
        }

        #[tokio::test]
        async fn v2_flat_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained() {
            vector_base_behavior_contract("flat-cpu").await;
        }

        #[tokio::test]
        async fn v2_hnsw_cpu_vector_base_compaction_maps_new_ids_live_cold_and_retained() {
            vector_base_behavior_contract("hnsw-cpu").await;
        }

        #[tokio::test]
        async fn v2_current_refuses_malformed_mapped_flat_vector_base_without_fallback() {
            let fixture = vector_base_fixture("flat-cpu").await;
            let (_, compacted_name) = vector_base_publish_four_checkpoints(&fixture).await;
            let generation = fixture.root.join(compacted_name);
            let original_manifest = stage1_read_manifest(&generation);
            let original_rows_path = vector_base_mapped_rows_path(
                &generation,
                vector_base_ref(&original_manifest, "field"),
            );
            let original_rows =
                std::fs::read(&original_rows_path).expect("read mapped vector rows");
            let original_checksum =
                vector_base_ref(&original_manifest, "field")["payload_sha256"].clone();

            let mut bad_format = original_manifest.clone();
            let local = vector_base_ref_mut(&mut bad_format, "field")["local_rows"]
                .as_object_mut()
                .expect("mapped vector base local rows");
            local.insert("format".to_owned(), json!("untrusted-local-row-format"));
            assert_eq!(
                vector_base_ref(&bad_format, "field")["payload_sha256"],
                original_checksum,
                "format corruption leaves the valid mapped payload checksum in place"
            );
            stage1_write_manifest(&generation, &bad_format);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "an unknown mapped local-row format",
            );
            stage1_write_manifest(&generation, &original_manifest);

            let mut bad_count = original_manifest.clone();
            let local = vector_base_ref_mut(&mut bad_count, "field")["local_rows"]
                .as_object_mut()
                .expect("mapped vector base local rows");
            let count = local["count"].as_u64().expect("mapped vector row count");
            local.insert("count".to_owned(), json!(count + 1));
            assert_eq!(
                vector_base_ref(&bad_count, "field")["payload_sha256"],
                original_checksum,
                "count corruption leaves the valid mapped payload checksum in place"
            );
            stage1_write_manifest(&generation, &bad_count);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a mapped local-row count mismatch",
            );
            stage1_write_manifest(&generation, &original_manifest);

            std::fs::remove_file(&original_rows_path).expect("remove mapped vector local-row map");
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a missing mapped local-row file",
            );
            std::fs::write(&original_rows_path, &original_rows)
                .expect("restore mapped vector local-row map");

            // This rewrites the compacted vector-EID sidecar with a separately
            // encoded, same-sequence segment whose IDs have the same count but a
            // different order. Its payload checksum is recomputed in the catalog.
            // The field payload and its mapped rows remain valid, so refusal must
            // reach the named vector-map consistency check.
            let original_eids = vector_base_ref(&original_manifest, "vector_eids");
            let original_eids_path = generation.join(
                original_eids["path"]
                    .as_str()
                    .expect("mapped vector EID sidecar path"),
            );
            let compacted_eids = std::fs::read(&original_eids_path)
                .expect("read mapped vector EID sidecar before mismatch mutation");
            let compacted_sequence = original_eids["applied_seq"]
                .as_u64()
                .expect("compacted vector EID sidecar sequence");
            let mapped_rows = stage1_keyword_delta_read_rows(
                &generation,
                vector_base_ref(&original_manifest, "field"),
            );
            assert_eq!(
                mapped_rows.len(),
                vector_base_ref(&original_manifest, "field")["local_rows"]["count"]
                    .as_u64()
                    .expect("mapped vector row count") as usize,
                "mapped vector rows must match their catalogued count before the mismatch mutation"
            );
            // FrozenField::capture sorts vector rows by external ID. A rotation of
            // the same IDs would therefore be normalized back to the valid order.
            // Use a distinct, zero-padded set instead: it retains the count and
            // gives the forged EID segment a deterministic row order that cannot
            // agree with the original mapped-base local rows.
            assert!(
                !mapped_rows.is_empty(),
                "mismatch fixture needs at least one compacted vector row"
            );
            let forged_ids: Vec<_> = mapped_rows
                .iter()
                .enumerate()
                .map(|(row, id)| format!("forged-mapped-vector-{row:06}-{id}"))
                .collect();
            assert_ne!(
                forged_ids, mapped_rows,
                "the valid forged vector EID sidecar must use distinct stable IDs"
            );
            let forged_eids = vector_base_forged_eid_sidecar(compacted_sequence, &forged_ids).await;
            assert_ne!(
            forged_eids, compacted_eids,
            "the sidecar mismatch fixture must replace bytes, not merely rewrite the same vector IDs"
        );
            std::fs::write(&original_eids_path, &forged_eids)
                .expect("install independently valid same-sequence vector EID sidecar");
            let mut bad_eid_sidecar = original_manifest.clone();
            vector_base_ref_mut(&mut bad_eid_sidecar, "vector_eids")["payload_sha256"] =
                json!(vector_base_payload_sha256(&original_eids_path));
            stage1_write_manifest(&generation, &bad_eid_sidecar);
            let current_before_mismatch = std::fs::read(fixture.root.join("CURRENT"))
                .expect("read CURRENT before vector map mismatch");
            let mismatch = match SegmentRdbStore::new(&fixture.root)
            .expect("reopen mapped-base root for vector map mismatch")
            .load_current_generation()
        {
            Ok(_) => panic!("CURRENT must refuse a checksum-valid vector EID sidecar that disagrees with mapped rows"),
            Err(error) => error,
        };
            assert!(
            format!("{mismatch:#}").contains("mapped vector row map differs from EID sidecar"),
            "a valid same-sequence vector EID sidecar with permuted stable IDs must reach the mapped-vector row mismatch refusal, got: {mismatch:#}"
        );
            assert_eq!(
                std::fs::read(fixture.root.join("CURRENT"))
                    .expect("read CURRENT after vector map mismatch"),
                current_before_mismatch,
                "refusing a checksum-valid mapped-vector row mismatch must leave CURRENT unchanged"
            );
            std::fs::write(&original_eids_path, &compacted_eids)
                .expect("restore mapped vector EID sidecar");
            stage1_write_manifest(&generation, &original_manifest);

            let mut bad_checksum = original_manifest.clone();
            vector_base_ref_mut(&mut bad_checksum, "field")["payload_sha256"] =
                json!("0".repeat(64));
            stage1_write_manifest(&generation, &bad_checksum);
            vector_base_assert_current_refuses_mapped_base(
                &fixture.root,
                "a mapped-base payload checksum mismatch",
            );
            stage1_write_manifest(&generation, &original_manifest);

            assert!(
                SegmentRdbStore::new(&fixture.root)
                    .expect("reopen restored mapped-base root")
                    .load_current_generation()
                    .is_ok(),
                "restoring exact mapped-base bytes must restore a cold-loadable CURRENT"
            );
        }
    }
}

