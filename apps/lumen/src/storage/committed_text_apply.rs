//! Apply borrowed Text input through immutable prepared rows.
//!
//! The small Index entry carries identity, type and version metadata only.
//! Its String placeholders are never persistence input. The caller retains
//! the original scanner bytes through the same-lease completion callback.

use super::*;
use crate::capture_barrier::ApplyLease;
use crate::wal::fast_index_scanner::FastIndexScanner;

impl Engine {
    pub(crate) fn try_apply_committed_index(
        &self,
        scanner: &FastIndexScanner<'_>,
        sequence: u64,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        self.try_apply_committed_index_with_capacity_owner(
            scanner,
            sequence,
            || bail!("committed layer capacity needs a caller-owned maintainer"),
            complete,
        )
    }

    pub(crate) fn try_apply_committed_index_with_capacity_owner(
        &self,
        scanner: &FastIndexScanner<'_>,
        sequence: u64,
        mut ensure_owner: impl FnMut() -> Result<()>,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        let mut complete = Some(complete);
        if self.try_apply_committed_scalar_with_capacity_owner(
            scanner,
            sequence,
            &mut ensure_owner,
            |apply, outcome| {
                complete.take().expect("one completion")(apply, outcome);
            },
        )? {
            return Ok(true);
        }
        self.try_apply_committed_text(scanner, |apply, outcome| {
            complete.take().expect("one completion")(apply, outcome);
        })
    }

    fn try_apply_committed_text(
        &self,
        scanner: &FastIndexScanner<'_>,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        let metadata = text_preparation::borrowed_text_metadata_bound(scanner)?
            .checked_mul(2)
            .ok_or(RecordAdmissionError::Overflow)?;
        let initial = metadata
            .checked_add(text_preparation::TEXT_SCRATCH_BYTES)
            .ok_or(RecordAdmissionError::Overflow)?;
        let request = self.record_ram_request_from_bound(initial, 0);
        let mut reservation = self.wait_reserve_record_ram(&request)?;
        'prepare: loop {
            if reservation.bytes() < initial {
                reservation
                    .wait_grow_to(initial)
                    .map_err(RecordAdmissionError::Capacity)?;
            }
            let (entry, rows) = match self.prepare_borrowed_text_rows(scanner, &mut reservation) {
                Ok(Some(prepared)) => prepared,
                Ok(None) => return Ok(false),
                Err(error) => {
                    let Some(required) = error
                        .downcast_ref::<text_preparation::RequiredBorrowedTextWorkspace>()
                        .map(|workspace| workspace.required_bytes)
                    else {
                        return Err(error);
                    };
                    reservation
                        .wait_grow_to(required)
                        .map_err(RecordAdmissionError::Capacity)?;
                    continue 'prepare;
                }
            };
            loop {
                // Pin the old file owners that dispatch can replace. Their
                // last drop must happen after apply, not during a state write.
                let (revision, old_rows) = {
                    let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
                    let mut old_rows = Vec::with_capacity(scanner.cost().item_count);
                    if let Some(coll) = state.collections.get(scanner.collection_id()) {
                        for item in scanner.items() {
                            let Some(id) = coll.interner.id(item.external_id) else {
                                continue;
                            };
                            if let Some(FieldIndex::Text { idx, .. }) = coll.fields.get(item.field)
                            {
                                if let Some(row) = idx.staged_rows.get(&id) {
                                    old_rows.push(row.clone());
                                }
                            }
                        }
                    }
                    (self.capture_barrier.apply_revision(), old_rows)
                };
                // Placeholder strings allocate no token map. Normal live
                // metadata and error-prefix costs still come from the same
                // estimator used by ordinary Index admission.
                let cost = match self.estimate_record_cost(&entry) {
                    crate::change_record_cost::RecordEstimate::Ready(cost) => cost,
                    crate::change_record_cost::RecordEstimate::Retain { cause } => {
                        return Err(RecordAdmissionError::NeedsPreparation(cause).into());
                    }
                };
                let retained = Self::record_owned_bytes(&entry)?
                    .checked_add(cost.active)
                    .and_then(|n| n.checked_add(cost.frozen))
                    .and_then(|n| n.checked_add(cost.prepublish))
                    .and_then(|n| n.checked_add(metadata))
                    .and_then(|n| n.checked_add(rows.retained_reader_bytes()))
                    .ok_or(RecordAdmissionError::Overflow)?;
                if reservation.bytes() < retained {
                    drop(old_rows);
                    reservation
                        .wait_grow_to(retained)
                        .map_err(RecordAdmissionError::Capacity)?;
                    continue;
                }
                // All IO and token workspaces have gone. Only the private
                // entry, row owners, reader metadata and future changes remain.
                reservation
                    .finish_borrowed_preparation(retained)
                    .map_err(RecordAdmissionError::Capacity)?;
                let apply = self.capture_barrier.apply();
                if !rows.matches(self) {
                    drop(apply);
                    drop(old_rows);
                    drop(rows);
                    drop(entry);
                    reservation
                        .finish_borrowed_preparation(reservation.bytes().min(initial))
                        .map_err(RecordAdmissionError::Capacity)?;
                    continue 'prepare;
                }
                if self.capture_barrier.apply_revision() != revision {
                    drop(apply);
                    drop(old_rows);
                    continue;
                }
                let charge = self.retain_borrowed_reservation(reservation)?;
                let outcome = self.dispatch_raft_entry(entry, Some(&charge), Some(&rows));
                complete(&apply, outcome);
                drop(apply);
                drop(old_rows);
                drop(rows);
                return Ok(true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{
        CreateCollectionRequest, FieldValue, IndexItem, IndexRequest, StatsResponse,
    };
    use crate::wal::fast_index_scanner::FastIndexScanner;
    use crate::wal::WalRecord;

    fn engine() -> Engine {
        let engine = Engine::new();
        engine
            .create_collection_inner(
                "docs",
                CreateCollectionRequest {
                    fields: serde_json::from_value(serde_json::json!({
                        "body": {"type": "text", "analyzer": "whitespace_lower"},
                        "title": {"type": "text", "analyzer": "whitespace_lower"}
                    }))
                    .unwrap(),
                },
            )
            .unwrap();
        engine
    }

    fn item(id: &str, field: &str, value: FieldValue, version: Option<u64>) -> IndexItem {
        IndexItem {
            external_id: id.to_owned(),
            field: field.to_owned(),
            value,
            version,
        }
    }

    fn request(items: Vec<IndexItem>, request_id: Option<&str>) -> IndexRequest {
        IndexRequest {
            items,
            request_id: request_id.map(str::to_owned),
        }
    }

    fn search_request(text: &str) -> crate::types::SearchRequest {
        serde_json::from_value(serde_json::json!({
            "query": {"match": {"field": "body", "text": text, "op": "and"}},
            "limit": 10
        }))
        .unwrap()
    }

    fn search_json(engine: &Engine, text: &str) -> serde_json::Value {
        let mut value =
            serde_json::to_value(engine.search("docs", search_request(text)).unwrap()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("took_ms");
        object.remove("took_us");
        value
    }

    fn stats_json(engine: &Engine) -> serde_json::Value {
        let mut value = serde_json::to_value(engine.stats("docs").unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("last_indexed_at");
        value
    }

    fn owned(engine: &Engine, request: IndexRequest) -> Result<ApplyOutcome> {
        engine
            .index_inner("docs", request, None, None)
            .map(ApplyOutcome::Indexed)
    }

    fn borrowed(
        engine: &Engine,
        request: IndexRequest,
        sequence: u64,
        before_watermark: impl FnOnce(&Engine),
    ) -> Result<ApplyOutcome> {
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: request,
        })
        .encode()
        .unwrap();
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let mut completed = None;
        assert!(
            engine.try_apply_committed_index(&scanner, sequence, |apply, outcome| {
                // Dispatch has already changed the public index.  The durable
                // source callback receives that same enclosing apply lease and
                // moves the watermark only after it can observe those changes.
                before_watermark(engine);
                apply.advance_sequence(sequence);
                completed = Some(outcome);
            })?
        );
        completed.expect("borrowed Index must invoke its completion callback")
    }

    fn assert_same_result(actual: Result<ApplyOutcome>, expected: Result<ApplyOutcome>) {
        match (actual, expected) {
            (Ok(ApplyOutcome::Indexed(actual)), Ok(ApplyOutcome::Indexed(expected))) => {
                assert_eq!(
                    serde_json::to_value(actual).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                );
            }
            (Ok(actual), Ok(expected)) => {
                panic!("borrowed and owned returned non-Index outcomes: {actual:?} / {expected:?}")
            }
            (Err(actual), Err(expected)) => assert_eq!(actual.to_string(), expected.to_string()),
            (actual, expected) => panic!("borrowed and owned differ: {actual:?} / {expected:?}"),
        }
    }

    #[test]
    fn borrowed_text_matches_owned_bm25_stats_duplicate_ids_and_versions() {
        let actual = engine();
        let expected = engine();
        let request = request(
            vec![
                item(
                    "alpha",
                    "body",
                    FieldValue::String("rust rust".into()),
                    Some(1),
                ),
                item(
                    "bravo",
                    "body",
                    FieldValue::String("rust search".into()),
                    Some(1),
                ),
                item(
                    "alpha",
                    "body",
                    FieldValue::String("systems engineer".into()),
                    Some(2),
                ),
                item(
                    "alpha",
                    "body",
                    FieldValue::String("stale value".into()),
                    Some(1),
                ),
            ],
            Some("text-request"),
        );

        let actual_result = borrowed(&actual, request.clone(), 41, |engine| {
            let visible = search_json(engine, "engineer");
            assert_eq!(visible["hits"][0]["external_id"].as_str(), Some("alpha"));
        });
        let expected_result = owned(&expected, request);
        assert_same_result(actual_result, expected_result);
        assert_eq!(search_json(&actual, "rust"), search_json(&expected, "rust"));
        assert_eq!(
            search_json(&actual, "engineer"),
            search_json(&expected, "engineer")
        );
        assert_eq!(stats_json(&actual), stats_json(&expected));

        let stats: StatsResponse = actual.stats("docs").unwrap();
        assert_eq!(stats.documents_indexed, 2);
        assert_eq!(stats.fields["body"].avg_doc_len, Some(2.0));
        assert_eq!(stats.fields["body"].unique_terms, 4);
    }

    #[test]
    fn borrowed_text_wrong_type_keeps_the_owned_valid_prefix_and_error() {
        let actual = engine();
        let expected = engine();
        let request = request(
            vec![
                item(
                    "alpha",
                    "body",
                    FieldValue::String("valid prefix".into()),
                    None,
                ),
                item("alpha", "title", FieldValue::Vector(vec![0.1, 0.2]), None),
            ],
            None,
        );

        assert_same_result(
            borrowed(&actual, request.clone(), 42, |_| {}),
            owned(&expected, request),
        );
        assert_eq!(
            search_json(&actual, "valid"),
            search_json(&expected, "valid")
        );
        assert_eq!(stats_json(&actual), stats_json(&expected));
        assert_eq!(
            search_json(&actual, "valid")["hits"][0]["external_id"].as_str(),
            Some("alpha"),
        );
    }

    #[test]
    fn borrowed_text_unknown_field_keeps_the_owned_valid_prefix_and_error() {
        let actual = engine();
        let expected = engine();
        let request = request(
            vec![
                item(
                    "alpha",
                    "body",
                    FieldValue::String("valid prefix".into()),
                    Some(1),
                ),
                item(
                    "alpha",
                    "later",
                    FieldValue::String("unknown".into()),
                    Some(1),
                ),
            ],
            None,
        );

        assert_same_result(
            borrowed(&actual, request.clone(), 43, |_| {}),
            owned(&expected, request),
        );
        assert_eq!(
            search_json(&actual, "valid"),
            search_json(&expected, "valid")
        );
        assert_eq!(stats_json(&actual), stats_json(&expected));
        assert_eq!(
            search_json(&actual, "valid")["hits"][0]["external_id"].as_str(),
            Some("alpha"),
        );
    }

    #[test]
    fn borrowed_text_duplicate_request_does_not_replace_the_first_value() {
        let actual = engine();
        let expected = engine();
        let first = request(
            vec![item(
                "alpha",
                "body",
                FieldValue::String("first text".into()),
                Some(1),
            )],
            Some("same-request"),
        );
        let duplicate = request(
            vec![item(
                "alpha",
                "body",
                FieldValue::String("later text".into()),
                Some(2),
            )],
            Some("same-request"),
        );

        assert_same_result(
            borrowed(&actual, first.clone(), 44, |_| {}),
            owned(&expected, first),
        );
        assert_same_result(
            borrowed(&actual, duplicate.clone(), 45, |_| {}),
            owned(&expected, duplicate),
        );
        assert_eq!(
            search_json(&actual, "first"),
            search_json(&expected, "first")
        );
        assert_eq!(
            search_json(&actual, "later"),
            search_json(&expected, "later")
        );
        assert_eq!(stats_json(&actual), stats_json(&expected));
        assert_eq!(
            search_json(&actual, "later")["hits"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
    }
}
