//! One mutation dispatch boundary shared by direct Engine calls and WAL apply.
//! Requests move into this boundary; no field payload is cloned for routing.

use super::*;
use crate::log_entry::RaftLogEntry;

impl Engine {
    /// Create-or-merge a collection schema.
    ///
    /// * If the collection does not exist, it is created with the
    ///   declared fields at version 1.
    /// * If it exists but is soft-deleted, the tombstone is superseded
    ///   (#3953): the result is a fresh, empty collection holding only the
    ///   declared fields — but its version CONTINUES the tombstone's rather
    ///   than restarting at 1, so a caller watching the id cannot mistake a
    ///   supersede for a stale read. This holds across a *soft* delete only:
    ///   `force` and `sweep_deleted` drop the entry outright, and the next
    ///   create answers 1 again — that id has no history left to continue, and
    ///   `e2e/collection_version_never_moves_backwards.rs` pins the reset. A
    ///   caller that keys a cache on `version` must therefore treat a
    ///   *decrease* as "different collection", not as a stale response.
    /// * If it exists, fields **missing** from the existing schema are
    ///   appended online (version bumps once per call regardless of how
    ///   many fields were added). Re-declaring an existing field with a
    ///   **different** type / analyzer / multi flag is rejected — type
    ///   changes are an offline op (collection version bump + reindex)
    ///   not covered by this surface in v1.
    ///
    /// `collection_id` must not contain `:` — that character is reserved for
    /// custom-method routes such as `POST /collections:search`, so keeping
    /// it out of collection ids means the `:search` verb syntax can never be
    /// ambiguous with a collection id.
    pub fn create_collection(
        &self,
        collection_id: &str,
        req: CreateCollectionRequest,
    ) -> Result<CreateCollectionResponse> {
        match self.apply_local_record(RaftLogEntry::CreateCollection {
            collection_id: collection_id.to_owned(),
            req,
        })? {
            ApplyOutcome::Created(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// Drop a collection.
    ///
    /// - `force = true`: physically remove now.
    /// - `force = false`: soft-delete — mark `deleted_at = now()` so
    ///   reads/writes start returning 410 Gone and the periodic
    ///   `sweep_deleted` task removes the data after the grace window.
    pub fn drop_collection(&self, collection_id: &str, force: bool) -> Result<DropOutcome> {
        match self.apply_local_record(RaftLogEntry::DropCollection {
            collection_id: collection_id.to_owned(),
            force,
        })? {
            ApplyOutcome::Dropped(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// Drop a field from an existing collection. Online — postings are
    /// freed immediately and the schema version bumps. Returns the new
    /// collection version.
    pub fn drop_field(&self, collection_id: &str, field_name: &str) -> Result<u32> {
        match self.apply_local_record(RaftLogEntry::DropField {
            collection_id: collection_id.to_owned(),
            field_name: field_name.to_owned(),
        })? {
            ApplyOutcome::FieldChanged(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// Append a new field to an existing collection. Online — existing
    /// documents simply have no postings on the new field until they are
    /// re-indexed. Returns the new collection version.
    pub fn add_field(&self, collection_id: &str, field_name: &str, spec: FieldSpec) -> Result<u32> {
        match self.apply_local_record(RaftLogEntry::AddField {
            collection_id: collection_id.to_owned(),
            field_name: field_name.to_owned(),
            spec,
        })? {
            ApplyOutcome::FieldChanged(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    pub fn index(&self, collection_id: &str, req: IndexRequest) -> Result<IndexResponse> {
        match self.apply_local_record(RaftLogEntry::Index {
            collection_id: collection_id.to_owned(),
            req,
        })? {
            ApplyOutcome::Indexed(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    pub fn delete(
        &self,
        collection_id: &str,
        external_id: &str,
        field: Option<&str>,
    ) -> Result<()> {
        match self.apply_local_record(RaftLogEntry::Delete {
            collection_id: collection_id.to_owned(),
            external_id: external_id.to_owned(),
            field: field.map(str::to_owned),
        })? {
            ApplyOutcome::Deleted => Ok(()),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// Remove complete indexed rows for a bounded group of external ids.
    ///
    /// Validation runs before the state lock.  Under the lock we resolve every
    /// known id and its recorded fields before changing caches or postings.
    /// Once that preparation succeeds, the remainder only removes owned map
    /// entries and postings, so one durable command is atomic within this
    /// physical shard.  Unknown ids deliberately never enter the interner and
    /// are successful no-ops.
    pub fn unindex_docs(&self, collection_id: &str, req: BatchUnindexDocsRequest) -> Result<()> {
        match self.apply_local_record(RaftLogEntry::UnindexDocs {
            collection_id: collection_id.to_owned(),
            req,
        })? {
            ApplyOutcome::DocsUnindexed => Ok(()),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// Replace one live collection's document state with a fresh empty state.
    ///
    /// The map-slot replacement is the shard-local linearization point.  It
    /// costs only the declared schema, not the number of existing documents:
    /// old postings, interned ids, HNSW state, and mmap handles move to the
    /// process-wide reclaimer after the new state is visible.  This is not a
    /// loop over [`Self::delete`].
    pub fn truncate_docs(&self, collection_id: &str) -> Result<()> {
        match self.apply_local_record(RaftLogEntry::TruncateDocs {
            collection_id: collection_id.to_owned(),
        })? {
            ApplyOutcome::DocsTruncated => Ok(()),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    /// `PUT /collections/{id}/docs:replace`: each item's `fields` becomes
    /// the doc's entire indexed state, implicitly deleting any declared
    /// schema field the doc has today but that is absent from `fields`.
    ///
    /// Batch-level result stays `Ok` (HTTP 200) unless the batch itself is
    /// malformed or over [`MAX_BATCH_REPLACE_SIZE`] — a single bad item
    /// (unknown field, type mismatch, stale version) is reported per-item
    /// in [`ReplaceDocResult`] and never fails its siblings.
    pub fn replace_docs(
        &self,
        collection_id: &str,
        req: ReplaceDocsRequest,
    ) -> Result<ReplaceDocsResponse> {
        match self.apply_local_record(RaftLogEntry::ReplaceDocs {
            collection_id: collection_id.to_owned(),
            req,
        })? {
            ApplyOutcome::Replaced(response) => Ok(response),
            _ => unreachable!("mutation dispatcher returned a different operation outcome"),
        }
    }

    fn apply_local_record(&self, mut entry: RaftLogEntry) -> Result<ApplyOutcome> {
        // Direct calls have no committed log source yet. Capacity refusal is
        // therefore safe only before the first state mutation. Never wait for
        // checkpoint work from a caller that may not have a background driver.
        let admission_error = |error| match error {
            RecordAdmissionError::Capacity(error) => {
                match crate::change_admission::PendingChangeCapacity::from_prepublication(error) {
                    Ok(error) => anyhow::Error::new(error),
                    Err(error) => anyhow::Error::new(RecordAdmissionError::Capacity(error)),
                }
            }
            error => anyhow::Error::new(error),
        };
        let mut reservation = self
            .try_reserve_record(&entry, 0)
            .map_err(admission_error)?;
        reservation.before_publication();
        loop {
            match self.begin_admitted_record(entry, reservation) {
                Ok(mut prepared) => return self.apply_prepared_raft_entry(&mut prepared),
                Err(mut reprice) => {
                    let Some(required) = reprice.required else {
                        return Err(admission_error(reprice.error));
                    };
                    reprice
                        .reservation
                        .try_grow_to(required)
                        .map_err(|error| admission_error(RecordAdmissionError::Capacity(error)))?;
                    entry = reprice.entry;
                    reservation = reprice.reservation;
                }
            }
        }
    }

    /// Apply one ordered log record at the same mutation boundary used by direct calls.
    pub fn apply_raft_entry(&self, entry: RaftLogEntry) -> Result<ApplyOutcome> {
        self.dispatch_raft_entry(entry, None, None)
    }
}
