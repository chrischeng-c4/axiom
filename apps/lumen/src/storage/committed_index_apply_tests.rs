use super::*;
use crate::log_entry::RaftLogEntry;
use crate::types::{CreateCollectionRequest, IndexItem};
use crate::wal::WalRecord;

fn engine() -> Arc<Engine> {
    let engine = Arc::new(Engine::with_change_budget(
        crate::change_budget::ChangeBudget::with_hard_limit(32 * 1024 * 1024),
    ));
    engine
        .create_collection_inner(
            "docs",
            CreateCollectionRequest {
                fields: serde_json::from_value(serde_json::json!({
                    "kw": {"type":"keyword"}, "n":{"type":"number"}, "tags":{"type":"set"}, "body":{"type":"text", "analyzer":"whitespace_lower"}
                }))
                .unwrap(),
            },
        )
        .unwrap();
    engine
}
fn item(id: &str, field: &str, value: FieldValue, version: Option<u64>) -> IndexItem {
    IndexItem {
        external_id: id.into(),
        field: field.into(),
        value,
        version,
    }
}
fn request(items: Vec<IndexItem>, id: Option<&str>) -> IndexRequest {
    IndexRequest {
        items,
        request_id: id.map(str::to_owned),
    }
}
fn borrowed(engine: &Engine, req: &IndexRequest, sequence: u64) -> Result<IndexResponse> {
    let bytes = WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: req.clone(),
    })
    .encode()
    .unwrap();
    let scanner = FastIndexScanner::parse(&bytes).unwrap();
    let mut outcome = None;
    assert!(engine
        .try_apply_committed_index(&scanner, sequence, |apply, result| {
            apply.advance_sequence(sequence);
            outcome = Some(result);
        })
        .unwrap());
    match outcome.expect("record must complete")? {
        ApplyOutcome::Indexed(response) => Ok(response),
        _ => panic!("Index result expected"),
    }
}
fn state_fingerprint(engine: &Engine) -> serde_json::Value {
    let state = engine.state.read().unwrap();
    let coll = &state.collections["docs"];
    let rows: Vec<_> = coll
        .interner
        .to_eid
        .iter()
        .enumerate()
        .map(|(id, eid)| {
            let id = id as u32;
            let kw = match &coll.fields["kw"] {
                FieldIndex::Keyword(k) => k.keyword_at(id),
                _ => unreachable!(),
            };
            let n = match &coll.fields["n"] {
                FieldIndex::Number(n) => n.number_at(id).map(SortableF64::to_f64),
                _ => unreachable!(),
            };
            let tags = match &coll.fields["tags"] {
                FieldIndex::Set(s) => s.set_members(id),
                _ => unreachable!(),
            };
            let coverage: Vec<_> = ["kw", "n", "tags"]
                .into_iter()
                .filter(|field| {
                    coll.eid_fields
                        .get(&id)
                        .is_some_and(|set| set.contains(field))
                })
                .collect();
            serde_json::json!([eid, kw, n, tags, coverage, coll.cell_versions.get(&id)])
        })
        .collect();
    let sizes: BTreeMap<_, _> = ["kw", "n", "tags"]
        .into_iter()
        .map(|name| (name.to_owned(), scalar_bytes(&coll.fields[name])))
        .collect();
    serde_json::json!({"rows":rows,"bytes":sizes,"data_version":coll.data_version,
        "last_indexed":coll.last_indexed_at.is_some(),
        "requests":coll.seen_requests.iter().map(|(id, _)|id).collect::<Vec<_>>()})
}
fn compare_apply(actual: &Engine, reference: &Engine, req: &IndexRequest, sequence: u64) {
    let got = borrowed(actual, req, sequence);
    let wanted = reference.index_inner("docs", req.clone(), None, None);
    match (got, wanted) {
        (Ok(got), Ok(wanted)) => assert_eq!(
            serde_json::to_value(got).unwrap(),
            serde_json::to_value(wanted).unwrap()
        ),
        (Err(got), Err(wanted)) => assert_eq!(got.to_string(), wanted.to_string()),
        (got, wanted) => panic!("borrowed/owned outcome differs: {got:?} / {wanted:?}"),
    }
    assert_eq!(state_fingerprint(actual), state_fingerprint(reference));
}

#[test]
fn borrowed_apply_matches_owned_versions_duplicates_and_error_prefixes() {
    let actual = engine();
    let reference = engine();
    let requests = [
        request(
            vec![
                item("old", "kw", FieldValue::String("base".into()), Some(7)),
                item("old", "n", FieldValue::Number(3.5), None),
                item(
                    "old",
                    "tags",
                    FieldValue::StringList(vec!["a".into(), "b".into()]),
                    None,
                ),
            ],
            None,
        ),
        request(
            vec![
                item("old", "kw", FieldValue::String("arrival".into()), None),
                item("old", "kw", FieldValue::String("stale".into()), Some(7)),
                item(
                    "new",
                    "tags",
                    FieldValue::StringList(vec!["z".into(), "a".into(), "z".into()]),
                    None,
                ),
                item("new", "kw", FieldValue::String("first".into()), Some(1)),
                item("new", "kw", FieldValue::String("last".into()), Some(2)),
                item("new", "n", FieldValue::Number(-0.0), None),
            ],
            Some("once"),
        ),
        request(
            vec![
                item("old", "n", FieldValue::Number(9.0), None),
                item(
                    "unknown-id",
                    "missing",
                    FieldValue::String("bad".into()),
                    None,
                ),
            ],
            None,
        ),
        request(
            vec![
                item(
                    "old",
                    "kw",
                    FieldValue::String("valid-prefix".into()),
                    Some(8),
                ),
                item("old", "kw", FieldValue::Number(99.0), Some(9)),
            ],
            None,
        ),
        request(
            vec![item(
                "new",
                "kw",
                FieldValue::String("must-not-apply".into()),
                Some(10),
            )],
            Some("once"),
        ),
        request(Vec::new(), Some("empty")),
    ];
    for (sequence, req) in requests.iter().enumerate() {
        compare_apply(&actual, &reference, req, sequence as u64 + 1);
    }
    let state = actual.state.read().unwrap();
    let FieldIndex::Set(tags) = &state.collections["docs"].fields["tags"] else {
        unreachable!()
    };
    assert!(
        tags.forward.is_empty(),
        "staged values must not remain in owned overlay"
    );
    assert!(
        tags.segment.is_some(),
        "queries must use attached immutable layers"
    );
    assert!(
        actual.changes.budget.snapshot().total > 0,
        "journal retains pending metadata charge"
    );
}

#[test]
fn concurrent_apply_invalidates_detached_scalar_plan_before_install() {
    let actual = engine();
    let initial = request(
        vec![item("id", "kw", FieldValue::String("base".into()), Some(1))],
        None,
    );
    borrowed(&actual, &initial, 1).unwrap();
    let newer = actual.clone();
    BEFORE_ATTACH.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(move || {
            newer
                .index_inner(
                    "docs",
                    request(
                        vec![item(
                            "id",
                            "kw",
                            FieldValue::String("newer".into()),
                            Some(3),
                        )],
                        None,
                    ),
                    None,
                    None,
                )
                .unwrap();
        }))
    });
    let response = borrowed(
        &actual,
        &request(
            vec![item(
                "id",
                "kw",
                FieldValue::String("staged-old".into()),
                Some(2),
            )],
            None,
        ),
        2,
    )
    .unwrap();
    assert_eq!(
        response.indexed, 0,
        "a stale detached plan must be re-evaluated against the newer version"
    );
    let state = actual.state.read().unwrap();
    let coll = &state.collections["docs"];
    let FieldIndex::Keyword(k) = &coll.fields["kw"] else {
        unreachable!()
    };
    assert_eq!(
        k.keyword_at(coll.interner.id("id").unwrap()).as_deref(),
        Some("newer")
    );
}

#[test]
fn vector_type_error_keeps_the_valid_scalar_prefix_on_borrowed_route() {
    let actual = engine();
    let reference = engine();
    for engine in [&actual, &reference] {
        engine.add_field_inner("docs", "vector", serde_json::from_value(
            serde_json::json!({"type":"vector", "dim":2, "metric":"l2", "backend":"flat-cpu"})
        ).unwrap()).unwrap();
    }
    let req = request(
        vec![
            item("id", "kw", FieldValue::String("prefix".into()), None),
            item("id", "vector", FieldValue::Number(7.0), None),
        ],
        None,
    );
    compare_apply(&actual, &reference, &req, 1);
    let state = actual.state.read().unwrap();
    let coll = &state.collections["docs"];
    let FieldIndex::Keyword(keyword) = &coll.fields["kw"] else {
        unreachable!()
    };
    assert_eq!(
        keyword
            .keyword_at(coll.interner.id("id").unwrap())
            .as_deref(),
        Some("prefix")
    );
}

fn text_search(engine: &Engine, text: &str) -> serde_json::Value {
    let request = serde_json::from_value(serde_json::json!({
        "query": {"match": {"field":"body", "text": text}},
        "limit": 10
    }))
    .unwrap();
    let mut value = serde_json::to_value(engine.search("docs", request).unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("took_ms");
    value.as_object_mut().unwrap().remove("took_us");
    value
}

fn text_stats(engine: &Engine) -> (u64, u64, u64) {
    let state = engine.state.read().unwrap();
    let FieldIndex::Text { idx, .. } = &state.collections["docs"].fields["body"] else {
        unreachable!()
    };
    (idx.doc_count, idx.total_doc_len, idx.bytes)
}

#[test]
fn borrowed_mixed_text_and_scalars_match_owned_bytes_stats_duplicates_and_workspace_retry() {
    let actual = engine();
    let reference = engine();
    // The single Unicode token needs more than the initial metadata spare
    // space. The real committed wrapper must grow outside apply, then retry.
    let unicode = "İ".repeat(1_500);
    let req = request(
        vec![
            item("same", "body", FieldValue::String(unicode), Some(1)),
            item("same", "kw", FieldValue::String("first".into()), Some(1)),
            item("same", "n", FieldValue::Number(3.5), Some(1)),
            item(
                "same",
                "tags",
                FieldValue::StringList(vec!["a".into(), "b".into()]),
                Some(1),
            ),
            item(
                "same",
                "body",
                FieldValue::String("final body term".into()),
                Some(2),
            ),
            item("same", "kw", FieldValue::String("final".into()), Some(2)),
        ],
        Some("mixed-once"),
    );
    reset_text_workspace_retries_for_test();
    let actual_response = borrowed(&actual, &req, 17).unwrap();
    assert!(
        text_workspace_retries_for_test() >= 2,
        "the Unicode token must require a second retry beyond the fixed Text scratch"
    );
    let expected_response = reference.index_inner("docs", req, None, None).unwrap();
    assert_eq!(
        serde_json::to_value(actual_response).unwrap(),
        serde_json::to_value(expected_response).unwrap()
    );
    assert_eq!(
        text_search(&actual, "final body"),
        text_search(&reference, "final body")
    );
    assert_eq!(text_stats(&actual), text_stats(&reference));
    assert_eq!(state_fingerprint(&actual), state_fingerprint(&reference));
}

#[test]
fn borrowed_mixed_text_wrong_type_and_unknown_field_keep_owned_valid_prefix() {
    let actual = engine();
    let reference = engine();
    let wrong_type = request(
        vec![
            item(
                "id",
                "body",
                FieldValue::String("kept text".into()),
                Some(1),
            ),
            item(
                "id",
                "kw",
                FieldValue::String("kept keyword".into()),
                Some(1),
            ),
            item("id", "body", FieldValue::Number(9.0), Some(2)),
        ],
        None,
    );
    compare_apply(&actual, &reference, &wrong_type, 18);
    assert_eq!(
        text_search(&actual, "kept text"),
        text_search(&reference, "kept text")
    );
    assert_eq!(text_stats(&actual), text_stats(&reference));
    // Seed a live staged Text row, then make a wrong-type-only replacement.
    // This exercises the Text Drop ledger branch with no successful Text Apply.
    let seed = request(
        vec![item(
            "drop",
            "body",
            FieldValue::String("seeded text".into()),
            Some(1),
        )],
        None,
    );
    compare_apply(&actual, &reference, &seed, 19);
    // Warm the rank cache before the Drop-only invalidation.
    assert_eq!(
        text_search(&actual, "seeded text"),
        text_search(&reference, "seeded text")
    );
    let drop_only = request(
        vec![item("drop", "body", FieldValue::Number(1.0), Some(2))],
        None,
    );
    compare_apply(&actual, &reference, &drop_only, 20);
    assert_eq!(
        text_search(&actual, "seeded text"),
        text_search(&reference, "seeded text")
    );
    assert_eq!(text_stats(&actual), text_stats(&reference));
    let unknown = request(
        vec![
            item("id", "body", FieldValue::String("new text".into()), Some(3)),
            item("id", "missing", FieldValue::String("bad".into()), None),
        ],
        None,
    );
    compare_apply(&actual, &reference, &unknown, 21);
    assert_eq!(
        text_search(&actual, "new text"),
        text_search(&reference, "new text")
    );
    assert_eq!(text_stats(&actual), text_stats(&reference));
}

#[test]
fn frozen_checkpoint_slot_forces_real_maintenance_before_sixteenth_private_append() {
    let engine = engine();
    let mut window = Some(engine.layer_maintenance.freeze());
    for sequence in 1..=15 {
        borrowed(
            &engine,
            &request(
                vec![item(
                    "same",
                    "kw",
                    FieldValue::String(format!("value-{sequence}")),
                    None,
                )],
                None,
            ),
            sequence,
        )
        .unwrap();
    }
    let pinned = {
        let state = engine.state.read().unwrap();
        let view = composed(&state.collections["docs"].fields["kw"]).unwrap();
        assert_eq!(view.incremental_layer_count(), 15);
        view.clone()
    };
    let raw = WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: request(
            vec![item(
                "same",
                "kw",
                FieldValue::String("value-16".into()),
                None,
            )],
            None,
        ),
    })
    .encode()
    .unwrap();
    let scanner = FastIndexScanner::parse(&raw).unwrap();
    let mut fallback = None;
    let mut requested = false;
    let mut completions = 0;
    assert!(engine
        .try_apply_committed_index_with_capacity_owner(
            &scanner,
            16,
            || {
                requested = true;
                assert!(
                    engine.state.try_write().is_ok(),
                    "capacity wait must release state ownership"
                );
                assert!(
                    engine.capture_barrier.capture(0).is_ok(),
                    "capacity bootstrap must be outside apply"
                );
                assert!(
                    engine.changes.budget.snapshot().reserved > 0,
                    "committed input keeps its reservation while asking for maintenance"
                );
                // Complete the simulated older cut. The actual capacity worker still
                // needs to checkpoint the 15 private readers before the append resumes.
                drop(window.take());
                crate::segment_capacity::Fallback::ensure(&mut fallback, &engine, None)
            },
            |apply, outcome| {
                outcome.unwrap();
                completions += 1;
                apply.advance_sequence(16);
            }
        )
        .unwrap());
    assert!(
        requested,
        "frozen publication owns the last slot, so the sixteenth append must ask for maintenance"
    );
    assert_eq!(completions, 1);
    assert!(engine.metrics().segment_checkpoint_completed_total.get() > 0);
    assert_eq!(pinned.keyword_at(0).as_deref(), Some("value-15"));
    let state = engine.state.read().unwrap();
    let view = composed(&state.collections["docs"].fields["kw"]).unwrap();
    assert!(view.incremental_layer_count() <= 16);
    assert_eq!(view.keyword_at(0).as_deref(), Some("value-16"));
}

fn hash_engine() -> Arc<Engine> {
    let engine = engine();
    engine
        .create_collection_inner(
            "docs",
            CreateCollectionRequest {
                fields: serde_json::from_value(serde_json::json!({"hash":{"type":"hash"}}))
                    .unwrap(),
            },
        )
        .unwrap();
    engine
}

fn hash_fingerprint(engine: &Engine) -> serde_json::Value {
    let state = engine.state.read().unwrap();
    let coll = &state.collections["docs"];
    let FieldIndex::Hash(hash) = &coll.fields["hash"] else {
        unreachable!()
    };
    let rows: Vec<_> = coll
        .interner
        .to_eid
        .iter()
        .enumerate()
        .map(|(id, eid)| {
            serde_json::json!([
                eid,
                hash.hash_at(id as u32),
                coll.cell_versions.get(&(id as u32)),
                coll.eid_fields
                    .get(&(id as u32))
                    .is_some_and(|fields| fields.contains("hash"))
            ])
        })
        .collect();
    serde_json::json!({"rows": rows, "bytes":hash.bytes})
}

#[test]
fn borrowed_hash_matches_owned_prefix_errors_versions_and_duplicates() {
    let actual = hash_engine();
    let reference = hash_engine();
    let requests = [
        request(
            vec![
                item("one", "kw", FieldValue::String("prefix".into()), Some(1)),
                item(
                    "one",
                    "hash",
                    FieldValue::String(" 0X000042 ".into()),
                    Some(1),
                ),
                item("one", "hash", FieldValue::String("+ff".into()), Some(2)),
                item(
                    "one",
                    "hash",
                    FieldValue::String("wrong but stale".into()),
                    Some(1),
                ),
            ],
            Some("once"),
        ),
        request(
            vec![item(
                "one",
                "hash",
                FieldValue::String("01".into()),
                Some(3),
            )],
            Some("once"),
        ),
        request(
            vec![
                item("two", "hash", FieldValue::String("ff".into()), None),
                item("one", "hash", FieldValue::Number(2.0), Some(3)),
            ],
            None,
        ),
        request(
            vec![
                item("two", "hash", FieldValue::String("10".into()), None),
                item("two", "hash", FieldValue::String("invalid".into()), None),
                item("never", "kw", FieldValue::String("suffix".into()), None),
            ],
            Some("failed"),
        ),
        request(
            vec![
                item("one", "hash", FieldValue::String("17".into()), Some(4)),
                item("one", "missing", FieldValue::String("unknown".into()), None),
            ],
            None,
        ),
    ];
    for (offset, req) in requests.iter().enumerate() {
        compare_apply(&actual, &reference, req, 100 + offset as u64);
        assert_eq!(hash_fingerprint(&actual), hash_fingerprint(&reference));
    }
}

#[test]
fn borrowed_hash_large_leading_zero_source_retains_only_small_changes() {
    // The encoded value is larger than this Engine's entire change budget.
    let actual = hash_engine();
    let root = tempfile::tempdir().unwrap();
    let store = crate::segment_rdb::SegmentRdbStore::new(root.path()).unwrap();
    {
        let apply = actual.capture_barrier.apply();
        actual
            .index_inner(
                "docs",
                request(
                    vec![item("one", "hash", FieldValue::String("01".into()), None)],
                    None,
                ),
                None,
                None,
            )
            .unwrap();
        apply.advance_sequence(22);
    }
    store.save(&actual, 22).unwrap();
    {
        let state = actual.state.read().unwrap();
        let FieldIndex::Hash(hash) = &state.collections["docs"].fields["hash"] else {
            unreachable!()
        };
        assert!(
            hash.segment.is_some(),
            "the next checkpoint must update an existing Hash base"
        );
    }
    let value = format!("0x{}42", "0".repeat(33 * 1024 * 1024));
    let bytes = WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".into(),
        req: request(
            vec![
                item("one", "kw", FieldValue::String("prefix".into()), None),
                item("one", "hash", FieldValue::String(value), None),
            ],
            None,
        ),
    })
    .encode()
    .unwrap();
    let scanner = FastIndexScanner::parse(&bytes).unwrap();
    let mut completed = false;
    assert!(
        actual
            .try_apply_committed_index(&scanner, 23, |apply, result| {
                let ApplyOutcome::Indexed(response) = result.unwrap() else {
                    panic!("Index outcome")
                };
                assert_eq!(response.indexed, 2);
                assert_eq!(response.bytes_written["hash"], 12);
                apply.advance_sequence(23);
                completed = true;
            })
            .unwrap(),
        "valid Hash must use borrowed apply without owning the source string"
    );
    assert!(completed);
    assert_eq!(hash_fingerprint(&actual)["rows"][0][1], 66);
    let pending = actual.changes.budget.snapshot();
    assert!(
        pending.active + pending.frozen + pending.reserved < 1024 * 1024,
        "the retained journal owns the parsed u64, not the source string: {pending:?}"
    );
    store.save(&actual, 23).unwrap();
    let (cold, sequence) = store.load_latest().unwrap().unwrap();
    assert_eq!(sequence, 23);
    assert_eq!(hash_fingerprint(&actual)["rows"][0][1], 66);
    assert_eq!(
        hash_fingerprint(&cold)["rows"][0][1],
        66,
        "incremental checkpoint must persist the parsed Hash value"
    );
    // A cold mmap reader has a different resident-byte footprint. Logical
    // rows, versions and coverage must still match the live view exactly.
    assert_eq!(
        hash_fingerprint(&cold)["rows"],
        hash_fingerprint(&actual)["rows"]
    );
}

// Vector cases use their own schema and keep the scalar fixtures unchanged.
// self-contained so it does not change the existing scalar test schema.
#[cfg(test)]
mod borrowed_vector_apply_tests {
    use super::*;
    use crate::types::VectorBackend;

    fn vector_engine(backend: VectorBackend, sq: bool) -> Arc<Engine> {
        let engine = Arc::new(Engine::with_change_budget(
            crate::change_budget::ChangeBudget::with_hard_limit(32 * 1024 * 1024),
        ));
        engine.create_collection_inner("docs", CreateCollectionRequest { fields: serde_json::from_value(serde_json::json!({
            "vec": {"type":"vector", "dim":3, "metric":"l2", "backend": if matches!(backend, VectorBackend::FlatCpu) { "flat-cpu" } else { "hnsw-cpu" }, "quantize": if sq { serde_json::json!("sq") } else { serde_json::Value::Null } }
        })).unwrap() }).unwrap();
        engine
    }

    fn vector_request(items: Vec<IndexItem>, request_id: Option<&str>) -> IndexRequest {
        IndexRequest {
            items,
            request_id: request_id.map(str::to_owned),
        }
    }

    fn borrowed_vector(
        engine: &Engine,
        req: &IndexRequest,
        sequence: u64,
    ) -> (bool, Option<Result<ApplyOutcome>>) {
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: req.clone(),
        })
        .encode()
        .unwrap();
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let mut outcome = None;
        let handled = engine
            .try_apply_committed_index(&scanner, sequence, |apply, result| {
                apply.advance_sequence(sequence);
                outcome = Some(result);
            })
            .unwrap();
        (handled, outcome)
    }

    fn vector_bits(engine: &Engine, eid: &str) -> Option<Vec<u32>> {
        let state = engine.state.read().unwrap();
        let coll = &state.collections["docs"];
        let FieldIndex::Vector { idx, .. } = &coll.fields["vec"] else {
            unreachable!()
        };
        idx.checkpoint_vector(eid)
            .unwrap()
            .map(|row| row.into_iter().map(f32::to_bits).collect())
    }

    #[test]
    fn borrowed_vector_matches_owned_ledger_for_each_backend_and_quantizer() {
        for backend in [VectorBackend::FlatCpu, VectorBackend::HnswCpu] {
            for sq in [false, true] {
                let actual = vector_engine(backend, sq);
                let owned = vector_engine(backend, sq);
                let ledger = [
                    vector_request(
                        vec![item(
                            "id",
                            "vec",
                            FieldValue::Vector(vec![0.0, 1.0, 2.0]),
                            Some(2),
                        )],
                        None,
                    ),
                    // Older wrong-dimension action is stale and must not replace the first vector.
                    vector_request(
                        vec![item("id", "vec", FieldValue::Vector(vec![9.0]), Some(1))],
                        None,
                    ),
                    // Both actions are valid. The v4 range must affect SQ decoding
                    // of the v5 final winner; collapsing to only the winner is wrong.
                    vector_request(
                        vec![
                            item(
                                "id",
                                "vec",
                                FieldValue::Vector(vec![-100.0, 0.0, 100.0]),
                                Some(4),
                            ),
                            item(
                                "id",
                                "vec",
                                FieldValue::Vector(vec![0.0, 1.0, 2.0]),
                                Some(5),
                            ),
                            item(
                                "third",
                                "vec",
                                FieldValue::Vector(vec![3.0, 4.0, 5.0]),
                                Some(1),
                            ),
                        ],
                        None,
                    ),
                    vector_request(
                        vec![item(
                            "id",
                            "vec",
                            FieldValue::Vector(vec![-4.0, 3.0, 9.0]),
                            Some(6),
                        )],
                        Some("once"),
                    ),
                    // A non-stale dimension error keeps the preceding valid cell.
                    vector_request(
                        vec![
                            item(
                                "dimension-prefix",
                                "vec",
                                FieldValue::Vector(vec![6.0, 6.0, 6.0]),
                                Some(1),
                            ),
                            item(
                                "dimension-bad",
                                "vec",
                                FieldValue::Vector(vec![1.0]),
                                Some(1),
                            ),
                        ],
                        None,
                    ),
                    // Valid prefix is retained, then the existing id is dropped before
                    // its wrong-type error, matching owned apply's error-prefix order.
                    vector_request(
                        vec![
                            item(
                                "next",
                                "vec",
                                FieldValue::Vector(vec![2.0, 2.0, 2.0]),
                                Some(1),
                            ),
                            item(
                                "id",
                                "vec",
                                FieldValue::String("wrong type".into()),
                                Some(7),
                            ),
                        ],
                        None,
                    ),
                    vector_request(
                        vec![item(
                            "id",
                            "vec",
                            FieldValue::Vector(vec![7.0, 7.0, 7.0]),
                            Some(8),
                        )],
                        Some("once"),
                    ),
                ];
                for (sequence, req) in ledger.iter().enumerate() {
                    let (handled, outcome) = borrowed_vector(&actual, req, sequence as u64 + 1);
                    assert!(handled, "Vector must stay on the borrowed fast ledger");
                    let wanted = owned.index_inner("docs", req.clone(), None, None);
                    match (outcome.expect("handled record completes"), wanted) {
                        (Ok(ApplyOutcome::Indexed(got)), Ok(want)) => assert_eq!(
                            serde_json::to_value(got).unwrap(),
                            serde_json::to_value(want).unwrap()
                        ),
                        (Err(got), Err(want)) => assert_eq!(got.to_string(), want.to_string()),
                        (got, want) => panic!("borrowed/owned outcome differs: {got:?} / {want:?}"),
                    }
                    assert_eq!(vector_bits(&actual, "id"), vector_bits(&owned, "id"));
                    assert_eq!(vector_bits(&actual, "next"), vector_bits(&owned, "next"));
                    assert_eq!(vector_bits(&actual, "third"), vector_bits(&owned, "third"));
                }
            }
        }
    }

    #[test]
    fn committed_vector_apply_records_lock_wait_and_hnsw_add_by_backend() {
        for (backend, expected_hnsw_adds) in
            [(VectorBackend::HnswCpu, 1), (VectorBackend::FlatCpu, 0)]
        {
            let engine = vector_engine(backend, false);
            let (handled, outcome) = borrowed_vector(
                &engine,
                &vector_request(
                    vec![item(
                        "id",
                        "vec",
                        FieldValue::Vector(vec![0.0, 1.0, 2.0]),
                        Some(1),
                    )],
                    None,
                ),
                1,
            );
            assert!(handled, "Vector must stay on the borrowed fast ledger");
            assert!(matches!(outcome, Some(Ok(ApplyOutcome::Indexed(_)))));
            assert_eq!(
                engine
                    .metrics()
                    .engine_state_write_lock_wait_seconds_count
                    .get(),
                1,
                "every committed Vector apply acquires the state write lock once"
            );
            assert_eq!(
                engine.metrics().hnsw_add_seconds_count.get(),
                expected_hnsw_adds,
                "only the HNSW backend records live graph-add work"
            );
        }
    }

    fn frozen_vector_bits(engine: &Engine, eid: &str) -> Option<Vec<u32>> {
        let frozen = engine.freeze_checkpoint_collections(None).unwrap();
        let row = frozen.capture.frozen_changes["docs"].row("vec", eid)?;
        match &**row.value()? {
            CheckpointValue::StagedVector(value) => Some(
                value
                    .as_f32_slice()
                    .iter()
                    .map(|value| value.to_bits())
                    .collect(),
            ),
            CheckpointValue::Vector(value) => {
                Some(value.iter().map(|value| value.to_bits()).collect())
            }
            value => panic!("expected vector journal value, got {value:?}"),
        }
    }

    #[test]
    fn stale_sq_vector_preparation_retries_after_concurrent_codebook_widen() {
        for backend in [VectorBackend::FlatCpu, VectorBackend::HnswCpu] {
            let actual = vector_engine(backend, true);
            let reference = vector_engine(backend, true);
            // Reuse existing IDs so the data/apply version checks detect the
            // codebook race without help from the interner watermark.
            let initial = vector_request(
                vec![
                    item(
                        "range",
                        "vec",
                        FieldValue::Vector(vec![0.0, 0.0, 0.0]),
                        None,
                    ),
                    item(
                        "pending",
                        "vec",
                        FieldValue::Vector(vec![0.0, 0.0, 0.0]),
                        None,
                    ),
                ],
                None,
            );
            actual
                .index_inner("docs", initial.clone(), None, None)
                .unwrap();
            reference.index_inner("docs", initial, None, None).unwrap();
            let widening = vector_request(
                vec![item(
                    "range",
                    "vec",
                    FieldValue::Vector(vec![-100.0, 0.0, 100.0]),
                    Some(1),
                )],
                None,
            );
            let pending = vector_request(
                vec![item(
                    "pending",
                    "vec",
                    FieldValue::Vector(vec![0.0, 1.0, 2.0]),
                    Some(1),
                )],
                None,
            );

            // This is the owned order the eventual retry must reproduce. It is an
            // independent oracle: it does not use detached staging or BEFORE_ATTACH.
            reference
                .index_inner("docs", widening.clone(), None, None)
                .unwrap();
            reference
                .index_inner("docs", pending.clone(), None, None)
                .unwrap();

            let concurrent = actual.clone();
            BEFORE_ATTACH.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    concurrent
                        .index_inner("docs", widening, None, None)
                        .unwrap();
                }));
            });
            let (handled, outcome) = borrowed_vector(&actual, &pending, 41);
            assert!(
                handled,
                "the retried Vector record must stay on the borrowed route"
            );
            assert!(matches!(outcome, Some(Ok(ApplyOutcome::Indexed(_)))));

            assert_eq!(
                vector_bits(&actual, "pending"),
                vector_bits(&reference, "pending"),
                "live SQ storage must use the codebook widened by the concurrent apply"
            );
            assert_eq!(
                frozen_vector_bits(&actual, "pending"),
                frozen_vector_bits(&reference, "pending"),
                "the retained staged checkpoint row must be rebuilt from the widened codebook"
            );
        }
    }
}
