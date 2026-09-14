//! Stage one complete generic replacement before the common apply boundary.
//!
//! The original journal remains authoritative. The private fast wire file is
//! only a borrowed input for the existing field preparation path. Its document
//! spans preserve full replacement and document versions through one apply.

use super::committed_replace_plan::{ParsedValue, ParsedValues, ReplaceDocDescriptor};
use super::*;
use crate::capture_barrier::ApplyLease;
use crate::wal::borrowed_replace_scanner::BorrowedReplaceScanner;
use crate::wal::borrowed_replace_spool::BorrowedReplaceSpool;
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue};
use std::hash::Hasher;

pub(super) struct ReplacementInput<'a> {
    pub(super) docs: &'a [ReplaceDocDescriptor<'a>],
    pub(super) parsed: ParsedValues,
}

pub(super) struct ReplacementLedger {
    pub(super) outcomes: Vec<super::committed_replace_plan::ReplaceItemOutcome>,
    pub(super) results: Vec<ReplaceDocResult>,
    pub(super) final_docs: Vec<super::committed_replace_plan::ReplaceDocFinal>,
    pub(super) final_checksums: Vec<super::committed_replace_plan::ReplaceChecksumFinal>,
    pub(super) fields_written: u64,
    pub(super) fields_skipped: u64,
}

impl ReplacementLedger {
    pub(super) fn render(&mut self, collection: &str, scanner: &FastIndexScanner<'_>) {
        use super::committed_replace_plan::{
            RenderedReplaceItemOutcome as Rendered, ReplaceItemOutcome as Raw,
        };
        self.results = std::mem::take(&mut self.outcomes)
            .into_iter()
            .map(|outcome| match outcome {
                Raw::Ok {
                    fields_written,
                    fields_skipped,
                } => ReplaceDocResult::Ok {
                    fields_written,
                    fields_skipped,
                },
                Raw::Dropped { current_version } => ReplaceDocResult::Dropped { current_version },
                Raw::Error { error } => match error.render(collection, scanner) {
                    Rendered::Error { code, message } => ReplaceDocResult::Error {
                        code: code.to_owned(),
                        message,
                    },
                    _ => unreachable!("rendered item error"),
                },
            })
            .collect();
    }
}

impl Engine {
    /// `false` preserves the existing decoder for unsupported wire shapes.
    /// All temporary reservation ownership is released before that fallback.
    pub(crate) fn try_apply_committed_replace_with_capacity_owner(
        &self,
        bytes: &[u8],
        sequence: u64,
        ensure_owner: &mut dyn FnMut() -> Result<()>,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        if bytes.starts_with(b"LWAL") {
            return Ok(false);
        }
        const FIXED: usize = 8192;
        let request = self.record_ram_request_from_bound(FIXED, 0);
        let mut reservation = match self.try_reserve_record_ram(&request) {
            Ok(reservation) => reservation,
            Err(RecordAdmissionError::Capacity(crate::change_budget::AdmissionError::Full {
                ..
            })) => self.wait_reserve_record_ram(&request)?,
            Err(error) => return Err(error.into()),
        };
        let mut reserve = |metadata: usize| -> Result<()> {
            let required = FIXED
                .checked_add(metadata)
                .ok_or(RecordAdmissionError::Overflow)?;
            if reservation.bytes() < required {
                reservation
                    .wait_grow_to(required)
                    .map_err(RecordAdmissionError::Capacity)?;
            }
            Ok(())
        };
        let Some(borrowed) = BorrowedReplaceScanner::scan(bytes, &mut reserve)? else {
            return Ok(false);
        };
        let spool = BorrowedReplaceSpool::prepare(&borrowed, &mut reserve)?;
        let scanner = FastIndexScanner::parse(spool.bytes())?;
        let required = scanner
            .cost()
            .item_count
            .checked_mul(256)
            .and_then(|n| n.checked_add(reservation.bytes()))
            .ok_or(RecordAdmissionError::Overflow)?;
        if reservation.bytes() < required {
            reservation
                .wait_grow_to(required)
                .map_err(RecordAdmissionError::Capacity)?;
        }
        // Checksums and Hash syntax are independent of the current schema.
        // Compute each once over the immutable source, outside state and apply.
        let mut parsed = ParsedValues::new();
        for (ordinal, item) in scanner.items().enumerate() {
            let value = match item.value {
                FastIndexValue::String(value) => ParsedValue {
                    hash: parse_hash_number(value).ok(),
                    checksum: Some(checksum_bytes(value.as_bytes())),
                },
                FastIndexValue::Vector { values, .. } => {
                    let mut hash = rustc_hash::FxHasher::default();
                    for value in values.chunks_exact(4) {
                        hash.write_u32(u32::from_le_bytes(
                            value.try_into().expect("validated f32 span"),
                        ));
                    }
                    ParsedValue {
                        hash: None,
                        checksum: Some(hash.finish()),
                    }
                }
                _ => continue,
            };
            parsed.insert(ordinal, value);
        }
        let replacement = ReplacementInput {
            docs: spool.descriptors(),
            parsed,
        };
        self.try_apply_committed_fields_with_capacity_owner(
            &scanner,
            Some(&replacement),
            sequence,
            ensure_owner,
            complete,
        )
        // The common adapter has released apply before these source owners and
        // the scoped scanner/spool reservation leave this function.
    }
}

#[cfg(test)]
#[path = "committed_replace_apply_tests.rs"]
mod tests;
