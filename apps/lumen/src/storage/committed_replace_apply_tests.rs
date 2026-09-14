// Append as `storage::committed_replace_apply_tests` after the forced route.
#[cfg(test)]
mod committed_replace_apply_tests {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::{
        change_budget::ChangeBudget,
        log_entry::RaftLogEntry,
        storage::{ApplyOutcome, Engine},
        types::{CreateCollectionRequest, FieldValue, ReplaceDocItem, ReplaceDocsRequest},
        wal::WalRecord,
    };
    use anyhow::Result;

    fn engine() -> Arc<Engine> {
        let engine = Arc::new(Engine::with_change_budget(ChangeBudget::with_hard_limit(
            32 * 1024 * 1024,
        )));
        engine.create_collection_inner("docs", CreateCollectionRequest { fields: serde_json::from_value(serde_json::json!({
            "kw":{"type":"keyword"}, "n":{"type":"number"}, "body":{"type":"text","analyzer":"whitespace_lower"},
            "v":{"type":"vector","dim":2,"metric":"cosine","backend":"flat-cpu"}
        })).unwrap() }).unwrap();
        engine
    }
    fn wire(req: ReplaceDocsRequest) -> Vec<u8> {
        let record = WalRecord::new(RaftLogEntry::ReplaceDocs {
            collection_id: "docs".into(),
            req,
        });
        // Generic CBOR is intentionally used: this route sits below the size
        // threshold and therefore must not need an oversized fixture.
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&record, &mut bytes).unwrap();
        bytes
    }
    fn borrowed(
        engine: &Engine,
        req: ReplaceDocsRequest,
        sequence: u64,
    ) -> Result<crate::types::ReplaceDocsResponse> {
        let bytes = wire(req);
        let mut owner = || Ok(());
        let mut completed = None;
        assert!(engine.try_apply_committed_replace_with_capacity_owner(
            &bytes,
            sequence,
            &mut owner,
            |apply, outcome| {
                apply.advance_sequence(sequence);
                completed = Some(outcome);
            }
        )?);
        match completed.expect("borrowed replacement must complete")? {
            ApplyOutcome::Replaced(response) => Ok(response),
            other => panic!("expected replacement outcome, got {other:?}"),
        }
    }
    fn owned(
        engine: &Engine,
        req: ReplaceDocsRequest,
    ) -> Result<crate::types::ReplaceDocsResponse> {
        engine.replace_docs_inner("docs", req, None, None)
    }
    fn parity(actual: &Engine, reference: &Engine, req: ReplaceDocsRequest, sequence: u64) {
        let got = borrowed(actual, req.clone(), sequence);
        let want = owned(reference, req);
        match (got, want) {
            (Ok(got), Ok(want)) => assert_eq!(
                serde_json::to_value(got).unwrap(),
                serde_json::to_value(want).unwrap()
            ),
            (Err(got), Err(want)) => assert_eq!(got.to_string(), want.to_string()),
            (got, want) => panic!("borrowed/owned replacement differ: {got:?} / {want:?}"),
        }
        assert_visible_parity(actual, reference);
    }
    fn doc(
        id: &str,
        version: Option<u64>,
        fields: impl IntoIterator<Item = (&'static str, FieldValue)>,
    ) -> ReplaceDocItem {
        ReplaceDocItem {
            external_id: id.into(),
            version,
            fields: fields
                .into_iter()
                .map(|(name, value)| (name.into(), value))
                .collect(),
        }
    }

    #[test]
    fn forced_borrowed_replace_matches_owned_prefix_errors_duplicate_order_and_versions() {
        let actual = engine();
        let reference = engine();
        let seed = ReplaceDocsRequest {
            docs: vec![doc(
                "same",
                Some(30),
                [
                    ("kw", FieldValue::String("old".into())),
                    ("n", FieldValue::Number(1.0)),
                    ("v", FieldValue::Vector(vec![1.0, 2.0])),
                ],
            )],
        };
        parity(&actual, &reference, seed, 1);
        // The first item has a valid field before a typed error. Its sibling
        // still runs, and the owned route is the full response/state oracle.
        parity(
            &actual,
            &reference,
            ReplaceDocsRequest {
                docs: vec![
                    doc(
                        "same",
                        Some(31),
                        [
                            ("kw", FieldValue::String("changed".into())),
                            ("n", FieldValue::String("wrong".into())),
                        ],
                    ),
                    doc(
                        "other",
                        None,
                        [("kw", FieldValue::String("sibling".into()))],
                    ),
                    doc(
                        "same",
                        Some(20),
                        [("kw", FieldValue::String("stale".into()))],
                    ),
                    doc(
                        "same",
                        None,
                        [("kw", FieldValue::String("unversioned".into()))],
                    ),
                ],
            },
            2,
        );
        let actual_state = actual.state.read().unwrap();
        let reference_state = reference.state.read().unwrap();
        let actual_collection = &actual_state.collections["docs"];
        let reference_collection = &reference_state.collections["docs"];
        let same = actual_collection
            .interner
            .to_eid
            .iter()
            .position(|id| id == "same")
            .unwrap() as u32;
        assert_eq!(
            actual_collection.doc_versions, reference_collection.doc_versions,
            "Replace owns document versions"
        );
        assert_eq!(
            actual_collection.field_checksums, reference_collection.field_checksums,
            "Replace checksum ledger differs"
        );
        assert!(
            actual_collection.cell_versions.get(&same).is_none(),
            "Replace must not create per-cell LWW versions"
        );
    }

    #[test]
    fn empty_duplicate_replacement_clears_prior_fields_and_text_vector_replacement_parity() {
        let actual = engine();
        let reference = engine();
        parity(
            &actual,
            &reference,
            ReplaceDocsRequest {
                docs: vec![doc(
                    "id",
                    Some(1),
                    [
                        ("body", FieldValue::String("snow snow".into())),
                        ("v", FieldValue::Vector(vec![0.0, 1.0])),
                        ("kw", FieldValue::String("present".into())),
                    ],
                )],
            },
            1,
        );
        // Empty later duplicate is a full replacement. It must remove the
        // omitted vector cell as well as its checksum and text presence.
        parity(
            &actual,
            &reference,
            ReplaceDocsRequest {
                docs: vec![
                    doc("id", Some(2), []),
                    doc(
                        "id",
                        Some(3),
                        [
                            ("body", FieldValue::String("next".into())),
                            ("v", FieldValue::Vector(vec![1.0, 0.0])),
                        ],
                    ),
                    doc("id", Some(4), [("body", FieldValue::String("next".into()))]),
                ],
            },
            2,
        );
    }

    #[test]
    fn outer_bulk_and_collection_errors_remain_distinct_from_item_results() {
        let actual = engine();
        let reference = engine();
        let too_many = ReplaceDocsRequest {
            docs: (0..33)
                .map(|number| ReplaceDocItem {
                    external_id: format!("id-{number}"),
                    version: None,
                    fields: BTreeMap::new(),
                })
                .collect(),
        };
        assert!(borrowed(&actual, too_many.clone(), 1).is_err());
        assert!(owned(&reference, too_many).is_err());
        let missing = WalRecord::new(RaftLogEntry::ReplaceDocs {
            collection_id: "missing".into(),
            req: ReplaceDocsRequest { docs: vec![] },
        });
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&missing, &mut bytes).unwrap();
        let mut owner = || Ok(());
        let mut completed = None;
        assert!(actual
            .try_apply_committed_replace_with_capacity_owner(
                &bytes,
                2,
                &mut owner,
                |apply, outcome| {
                    apply.advance_sequence(2);
                    completed = Some(outcome);
                }
            )
            .unwrap());
        assert!(completed.expect("missing collection completes").is_err());
    }

    fn state_fingerprint(engine: &Engine) -> serde_json::Value {
        use crate::storage::{FieldIndex, SortableF64};
        let state = engine.state.read().unwrap();
        let coll = &state.collections["docs"];
        let rows: Vec<_> = coll
            .interner
            .to_eid
            .iter()
            .enumerate()
            .map(|(id, eid)| {
                let id = id as u32;
                let values: BTreeMap<_, _> = coll
                    .fields
                    .iter()
                    .map(|(name, index)| {
                        let value = match index {
                            FieldIndex::Keyword(index) => serde_json::json!(index.keyword_at(id)),
                            FieldIndex::Number(index) => {
                                serde_json::json!(index.number_at(id).map(SortableF64::to_f64))
                            }
                            FieldIndex::Set(index) => serde_json::json!(index.set_members(id)),
                            FieldIndex::Hash(index) => serde_json::json!(index.hash_at(id)),
                            FieldIndex::Text { idx, .. } => serde_json::json!([
                                idx.doc_len(id),
                                idx.doc_count,
                                idx.total_doc_len
                            ]),
                            FieldIndex::Vector { idx, .. } => {
                                serde_json::json!(idx.checkpoint_vector(eid).unwrap())
                            }
                        };
                        (name, value)
                    })
                    .collect();
                let fields = coll
                    .eid_fields
                    .get(&id)
                    .map(|coverage| coverage.to_btree_set());
                serde_json::json!([
                    eid,
                    values,
                    fields,
                    coll.cell_versions.get(&id),
                    coll.doc_versions.get(&id),
                    coll.field_checksums.get(&id)
                ])
            })
            .collect();
        serde_json::json!({"rows": rows, "last_indexed": coll.last_indexed_at.is_some()})
    }

    fn assert_visible_parity(actual: &Engine, reference: &Engine) {
        assert_eq!(state_fingerprint(actual), state_fingerprint(reference));
        let has_text = actual.state.read().unwrap().collections["docs"]
            .fields
            .contains_key("body");
        if has_text {
            for text in ["snow", "next", "old", "same", "other"] {
                let query: crate::types::SearchRequest =
                    serde_json::from_value(serde_json::json!({
                        "query":{"match":{"field":"body","text":text,"op":"and"}},"limit":20
                    }))
                    .unwrap();
                let normalize = |engine: &Engine| {
                    let mut value =
                        serde_json::to_value(engine.search("docs", query.clone()).unwrap())
                            .unwrap();
                    value.as_object_mut().unwrap().remove("took_ms");
                    value.as_object_mut().unwrap().remove("took_us");
                    value
                };
                assert_eq!(normalize(actual), normalize(reference));
            }
        }
    }

    #[test]
    fn accepted_version_survives_unversioned_duplicate_and_stale_schema_is_dropped() {
        let actual = engine();
        let reference = engine();
        parity(
            &actual,
            &reference,
            ReplaceDocsRequest {
                docs: vec![
                    doc("id", Some(30), [("kw", FieldValue::String("first".into()))]),
                    doc("id", None, [("kw", FieldValue::String("second".into()))]),
                    doc("id", Some(20), [("missing", FieldValue::Number(0.))]),
                ],
            },
            1,
        );
        let state = actual.state.read().unwrap();
        let coll = &state.collections["docs"];
        let id = coll.interner.id("id").unwrap();
        assert_eq!(coll.doc_versions.get(&id), Some(&30));
    }

    #[test]
    fn index_then_equal_text_vector_replace_writes_once_then_skips() {
        let actual = engine();
        let reference = engine();
        let req = crate::types::IndexRequest {
            request_id: None,
            items: vec![
                crate::types::IndexItem {
                    external_id: "id".into(),
                    field: "body".into(),
                    version: Some(7),
                    value: FieldValue::String("same same".into()),
                },
                crate::types::IndexItem {
                    external_id: "id".into(),
                    field: "v".into(),
                    version: Some(7),
                    value: FieldValue::Vector(vec![0., 1.]),
                },
            ],
        };
        actual.index_inner("docs", req.clone(), None, None).unwrap();
        reference.index_inner("docs", req, None, None).unwrap();
        let replacement = ReplaceDocsRequest {
            docs: vec![doc(
                "id",
                None,
                [
                    ("body", FieldValue::String("same same".into())),
                    ("v", FieldValue::Vector(vec![0., 1.])),
                ],
            )],
        };
        let first = borrowed(&actual, replacement.clone(), 1).unwrap();
        let first_owned = owned(&reference, replacement.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&first_owned).unwrap()
        );
        assert!(matches!(
            first.results[0],
            crate::types::ReplaceDocResult::Ok {
                fields_written: 2,
                fields_skipped: 0
            }
        ));
        assert_visible_parity(&actual, &reference);
        let second = borrowed(&actual, replacement.clone(), 2).unwrap();
        let second_owned = owned(&reference, replacement).unwrap();
        assert_eq!(
            serde_json::to_value(&second).unwrap(),
            serde_json::to_value(&second_owned).unwrap()
        );
        assert!(matches!(
            second.results[0],
            crate::types::ReplaceDocResult::Ok {
                fields_written: 0,
                fields_skipped: 2
            }
        ));
        assert_visible_parity(&actual, &reference);
    }

    #[test]
    fn thirty_two_documents_can_replace_more_than_one_thousand_fields() {
        let fields: BTreeMap<String, crate::types::FieldSpec> = (0..33)
            .map(|i| {
                (
                    format!("f{i:02}"),
                    serde_json::from_value(serde_json::json!({"type":"keyword"})).unwrap(),
                )
            })
            .collect();
        let make = || {
            let engine =
                Engine::with_change_budget(ChangeBudget::with_hard_limit(64 * 1024 * 1024));
            engine
                .create_collection_inner(
                    "docs",
                    CreateCollectionRequest {
                        fields: fields.clone(),
                    },
                )
                .unwrap();
            engine
        };
        let actual = make();
        let reference = make();
        let req = ReplaceDocsRequest {
            docs: (0..32)
                .map(|i| ReplaceDocItem {
                    external_id: format!("id-{i}"),
                    version: Some(1),
                    fields: fields
                        .keys()
                        .map(|name| (name.clone(), FieldValue::String(format!("value-{i}"))))
                        .collect(),
                })
                .collect(),
        };
        parity(&actual, &reference, req, 1);
        assert_eq!(actual.stats("docs").unwrap().documents_indexed, 32);
    }

    #[test]
    fn malformed_wire_is_not_a_stale_noop_and_fragmented_fallback_releases_reservation() {
        let actual = engine();
        borrowed(
            &actual,
            ReplaceDocsRequest {
                docs: vec![doc(
                    "id",
                    Some(30),
                    [("kw", FieldValue::String("old".into()))],
                )],
            },
            1,
        )
        .unwrap();
        let baseline = state_fingerprint(&actual);
        let before = actual.changes.budget.snapshot();
        let mut malformed = wire(ReplaceDocsRequest {
            docs: vec![doc(
                "id",
                Some(20),
                [("missing", FieldValue::String("bad".into()))],
            )],
        });
        malformed.pop();
        let mut completed = false;
        assert!(actual
            .try_apply_committed_replace_with_capacity_owner(
                &malformed,
                2,
                &mut || Ok(()),
                |_, _| completed = true
            )
            .is_err());
        assert!(!completed);
        assert_eq!(state_fingerprint(&actual), baseline);
        assert_eq!(actual.changes.budget.snapshot().total, before.total);

        let raw = wire(ReplaceDocsRequest {
            docs: vec![doc(
                "id",
                None,
                [("kw", FieldValue::String("fallback-token".into()))],
            )],
        });
        let needle = b"fallback-token";
        let pos = raw
            .windows(needle.len())
            .position(|window| window == needle)
            .unwrap();
        assert_eq!(raw[pos - 1], 0x60 + needle.len() as u8);
        let mut fragmented = raw[..pos - 1].to_vec();
        fragmented.push(0x7f);
        fragmented.extend_from_slice(&raw[pos - 1..pos + needle.len()]);
        fragmented.push(0xff);
        fragmented.extend_from_slice(&raw[pos + needle.len()..]);
        assert!(
            WalRecord::decode(&fragmented).is_ok(),
            "fallback fixture is valid legacy-compatible CBOR"
        );
        assert!(!actual
            .try_apply_committed_replace_with_capacity_owner(
                &fragmented,
                2,
                &mut || Ok(()),
                |_, _| completed = true
            )
            .unwrap());
        assert!(!completed);
        assert_eq!(actual.changes.budget.snapshot().total, before.total);
        assert_eq!(state_fingerprint(&actual), baseline);
    }
}
