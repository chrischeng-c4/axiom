//! Private scalar files prepared from borrowed, already durable WAL bytes.
//!
//! This has no live attachment operation. The caller selects winners against a
//! captured collection version, reserves before allocation, and later rechecks
//! that version before it publishes all of the files in one apply interval.

use crate::segment::{
    stream::{self, ScalarProjectionScratch},
    SegmentReader,
};
use crate::types::FieldType;
use crate::wal::fast_index_scanner::{FastIndexItem, FastIndexScanner, FastIndexValue};
use anyhow::{anyhow, bail, ensure, Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(super) struct PreparedScalarFile {
    pub(super) reader: Arc<SegmentReader>,
    pub(super) external_ids: Vec<String>,
    pub(super) item_ordinals: Vec<usize>,
    pub(super) field: String,
    pub(super) bytes: u64,
    pub(super) retained_bytes: usize,
}

/// A total requirement, not an incremental allocation request. Both callbacks
/// run outside apply. A failed callback leaves the source entirely unchanged.
pub(super) fn prepare(
    scanner: &FastIndexScanner<'_>,
    field: &str,
    winning: &[usize],
    expected: FieldType,
    sequence: u64,
    parent: &Path,
    reserve_total: impl FnMut(usize) -> Result<()>,
) -> Result<PreparedScalarFile> {
    ensure!(
        scanner.cost().item_count <= super::MAX_INDEX_ITEMS,
        "committed scalar preparation exceeds existing Index item limit"
    );
    prepare_validated_fields(
        scanner,
        field,
        winning,
        expected,
        sequence,
        parent,
        reserve_total,
    )
}

/// Prepare scalar fields after the command-aware replacement planner has
/// checked its document and flattened-field limits. This retains every field,
/// ordinal, type, and reservation check; it only omits Index's item-count
/// policy so a bounded Replace command may flatten to more than 1,000 fields.
pub(super) fn prepare_validated_fields(
    scanner: &FastIndexScanner<'_>,
    field: &str,
    winning: &[usize],
    expected: FieldType,
    sequence: u64,
    parent: &Path,
    mut reserve_total: impl FnMut(usize) -> Result<()>,
) -> Result<PreparedScalarFile> {
    ensure!(
        matches!(
            expected,
            FieldType::Keyword | FieldType::Number | FieldType::Set
        ),
        "committed scalar preparation requires Keyword, Number, or Set"
    );
    u32::try_from(winning.len()).context("scalar local row count exceeds u32")?;
    for (row, ordinal) in winning.iter().enumerate() {
        ensure!(
            *ordinal < scanner.cost().item_count,
            "selected scalar ordinal is out of range"
        );
        ensure!(
            !winning[..row].contains(ordinal),
            "selected scalar ordinal repeats"
        );
    }
    // No allocation has occurred above or in this first borrowed pass.
    let mut identifier_bytes = field.len();
    let mut term_bytes = 0usize;
    let mut term_count = 0usize;
    let mut max_entry = scanner
        .cost()
        .item_count
        .checked_mul(5)
        .and_then(|n| n.checked_add(5))
        .context("scalar posting bound overflow")?
        .max(8);
    for (ordinal, item) in scanner.items().enumerate() {
        if !winning.contains(&ordinal) {
            continue;
        }
        ensure!(
            item.field == field,
            "selected scalar item belongs to a different field"
        );
        match (expected, &item.value) {
            (FieldType::Keyword, FastIndexValue::String(value)) => {
                max_entry = max_entry.max(value.len());
                term_bytes = term_bytes
                    .checked_add(value.len())
                    .context("scalar term byte bound overflow")?;
                term_count = term_count
                    .checked_add(1)
                    .context("scalar term count overflow")?
            }
            (FieldType::Number, FastIndexValue::Number(value)) => {
                super::SortableF64::new(*value)?;
                term_count = term_count
                    .checked_add(1)
                    .context("scalar key count overflow")?;
            }
            (FieldType::Set, FastIndexValue::StringList(list)) => {
                term_count = term_count
                    .checked_add(list.len())
                    .context("scalar set term count overflow")?;
                for value in list.values() {
                    max_entry = max_entry.max(value.len());
                    term_bytes = term_bytes
                        .checked_add(value.len())
                        .context("scalar set byte bound overflow")?;
                }
            }
            _ => bail!("selected scalar value does not match field type"),
        }
        identifier_bytes = identifier_bytes
            .checked_add(item.external_id.len())
            .context("scalar identifier bound overflow")?;
    }
    // Caller-owned descriptors, borrowed lookup nodes, final row maps and IDs.
    // Values are never collected. Codec/spool ownership is priced from the
    // format and the full-entry block boundary in the writer itself.
    let metadata = scanner
        .cost()
        .item_count
        .checked_mul(512)
        .and_then(|n| {
            identifier_bytes
                .checked_mul(4)
                .and_then(|ids| n.checked_add(ids))
        })
        .context("scalar metadata bound overflow")?;
    // Raw dictionaries lend term bytes directly to the file. Only postings
    // need a variable codec block, regardless of one term's length.
    let raw_dictionary = matches!(expected, FieldType::Keyword | FieldType::Set);
    let codec_entry = if raw_dictionary {
        scanner
            .cost()
            .item_count
            .checked_mul(5)
            .and_then(|n| n.checked_add(5))
            .context("scalar posting bound overflow")?
            .max(8)
    } else {
        max_entry
    };
    let workspace = stream::scalar_projection_peak_bound(
        codec_entry,
        if raw_dictionary { 0 } else { term_bytes },
        term_count,
        term_count,
    )?;
    let total = metadata
        .checked_add(workspace)
        .context("scalar reservation overflow")?;
    reserve_total(total)?;

    let items: Vec<_> = scanner.items().collect();
    let mut ids = BTreeSet::new();
    for &ordinal in winning {
        ensure!(
            ids.insert(items[ordinal].external_id),
            "selected scalar rows repeat external ID"
        );
    }
    let view = Selected {
        items: &items,
        winning,
    };
    let mut directory = StageDirectory::create(parent)?;
    let path = directory.path.join("field.lseg");
    let scratch = ScalarProjectionScratch::new(codec_entry);
    let scratch = if raw_dictionary {
        scratch.raw_scalar_dictionary()
    } else {
        scratch
    };
    match expected {
        FieldType::Keyword => stream::write_keyword_projection(&path, sequence, &view, scratch)?,
        FieldType::Number => stream::write_number_projection(&path, sequence, &view, scratch)?,
        FieldType::Set => stream::write_set_projection(&path, sequence, &view, scratch)?,
        _ => unreachable!("validated scalar field type"),
    }
    let reader_bytes = SegmentReader::staged_metadata_bound(&path)?;
    reserve_total(
        total
            .checked_add(reader_bytes)
            .context("scalar reader bound overflow")?,
    )?;
    let bytes = std::fs::metadata(&path)?.len();
    let reader = Arc::new(SegmentReader::open_owned_stage(
        &path,
        directory.path.clone(),
    )?);
    directory.transferred = true;
    Ok(PreparedScalarFile {
        reader,
        external_ids: winning
            .iter()
            .map(|&ordinal| items[ordinal].external_id.to_owned())
            .collect(),
        item_ordinals: winning.to_vec(),
        field: field.to_owned(),
        bytes,
        retained_bytes: reader_bytes
            .checked_add(
                winning
                    .len()
                    .checked_mul(512)
                    .and_then(|n| n.checked_add(identifier_bytes.checked_mul(4)?))
                    .context("scalar retained metadata bound overflow")?,
            )
            .context("scalar retained bound overflow")?,
    })
}

struct Selected<'view, 'source> {
    items: &'view [FastIndexItem<'source>],
    winning: &'view [usize],
}
impl<'source> Selected<'_, 'source> {
    fn item(&self, row: u32) -> &FastIndexItem<'source> {
        &self.items[self.winning[row as usize]]
    }
    fn terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>, set: bool) -> Result<()> {
        // Each pass keeps two borrowed terms. Set cardinality does not cause a
        // resident whole-dictionary allocation. Input is immutable across passes.
        let mut previous = None;
        loop {
            let mut next: Option<&str> = None;
            for &ordinal in self.winning {
                let mut consider = |value: &'source str| {
                    if previous.is_none_or(|old| value > old) && next.is_none_or(|old| value < old)
                    {
                        next = Some(value);
                    }
                };
                match &self.items[ordinal].value {
                    FastIndexValue::String(value) if !set => consider(value),
                    FastIndexValue::StringList(list) if set => {
                        for value in list.values() {
                            consider(value);
                        }
                    }
                    _ => unreachable!("validated scalar value"),
                }
            }
            match next {
                Some(value) => {
                    emit(value)?;
                    previous = Some(value);
                }
                None => return Ok(()),
            }
        }
    }
}
impl stream::KeywordStreamProjection for Selected<'_, '_> {
    fn n_docs(&self) -> u32 {
        self.winning.len() as u32
    }
    fn keyword_row(
        &self,
        row: u32,
        emit: &mut dyn FnMut(Option<&str>) -> Result<()>,
    ) -> Result<()> {
        let FastIndexValue::String(value) = self.item(row).value else {
            unreachable!()
        };
        emit(Some(value))
    }
    fn keyword_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        self.terms(emit, false)
    }
    fn keyword_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        let mut found = false;
        for (row, &ordinal) in self.winning.iter().enumerate() {
            if matches!(self.items[ordinal].value, FastIndexValue::String(value) if value == term) {
                emit(row as u32)?;
                found = true;
            }
        }
        Ok(found)
    }
}
impl stream::SetStreamProjection for Selected<'_, '_> {
    fn n_docs(&self) -> u32 {
        self.winning.len() as u32
    }
    fn set_row(&self, row: u32, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<bool> {
        let FastIndexValue::StringList(list) = self.item(row).value else {
            unreachable!()
        };
        let mut previous = None;
        loop {
            let next = list
                .values()
                .filter(|value| previous.is_none_or(|old| *value > old))
                .min();
            match next {
                Some(value) => {
                    emit(value)?;
                    previous = Some(value);
                }
                None => return Ok(true),
            }
        }
    }
    fn set_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        self.terms(emit, true)
    }
    fn set_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        let mut found = false;
        for (row, &ordinal) in self.winning.iter().enumerate() {
            let FastIndexValue::StringList(list) = self.items[ordinal].value else {
                unreachable!()
            };
            if list.values().any(|value| value == term) {
                emit(row as u32)?;
                found = true;
            }
        }
        Ok(found)
    }
}
impl stream::NumberStreamProjection for Selected<'_, '_> {
    fn n_docs(&self) -> u32 {
        self.winning.len() as u32
    }
    fn number_row(&self, row: u32, emit: &mut dyn FnMut(Option<f64>) -> Result<()>) -> Result<()> {
        let FastIndexValue::Number(value) = self.item(row).value else {
            unreachable!()
        };
        emit(Some(value))
    }
    fn number_keys(&self, emit: &mut dyn FnMut(u64) -> Result<()>) -> Result<()> {
        let mut previous = None;
        loop {
            let mut next = None;
            for &ordinal in self.winning {
                let FastIndexValue::Number(value) = self.items[ordinal].value else {
                    unreachable!()
                };
                let key = super::SortableF64::new(value)?.bits();
                if previous.is_none_or(|old| key > old) && next.is_none_or(|old| key < old) {
                    next = Some(key);
                }
            }
            match next {
                Some(key) => {
                    emit(key)?;
                    previous = Some(key);
                }
                None => return Ok(()),
            }
        }
    }
    fn number_posting(&self, key: u64, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        let mut found = false;
        for (row, &ordinal) in self.winning.iter().enumerate() {
            let FastIndexValue::Number(value) = self.items[ordinal].value else {
                unreachable!()
            };
            if super::SortableF64::new(value)?.bits() == key {
                emit(row as u32)?;
                found = true;
            }
        }
        Ok(found)
    }
}

struct StageDirectory {
    path: PathBuf,
    transferred: bool,
}
impl StageDirectory {
    fn create(parent: &Path) -> Result<Self> {
        for _ in 0..128 {
            let nonce = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(
                "lumen-staged-scalar-{}-{nonce}",
                std::process::id()
            ));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        transferred: false,
                    })
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("create private scalar stage directory"),
            }
        }
        bail!("could not allocate private scalar stage directory")
    }
}
impl Drop for StageDirectory {
    fn drop(&mut self) {
        if !self.transferred {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{FieldValue, IndexItem, IndexRequest};
    use crate::wal::WalRecord;

    fn command(rows: Vec<(&str, FieldValue)>) -> Vec<u8> {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".to_owned(),
            req: IndexRequest {
                request_id: None,
                items: rows
                    .into_iter()
                    .map(|(id, value)| IndexItem {
                        external_id: id.to_owned(),
                        field: "value".to_owned(),
                        value,
                        version: None,
                    })
                    .collect(),
            },
        })
        .encode()
        .unwrap()
    }
    fn staged(bytes: &[u8], rows: &[usize], kind: FieldType, parent: &Path) -> PreparedScalarFile {
        prepare(
            &FastIndexScanner::parse(bytes).unwrap(),
            "value",
            rows,
            kind,
            42,
            parent,
            |_| Ok(()),
        )
        .unwrap()
    }
    #[test]
    fn selected_keyword_rows_keep_sparse_identity_and_only_the_selected_last_value() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![
            ("doc-90000000", FieldValue::String("old".into())),
            ("skip", FieldValue::String("discard".into())),
            ("doc-90000000", FieldValue::String("new".into())),
            ("doc-7", FieldValue::String("".into())),
        ]);
        let output = staged(&bytes, &[3, 2], FieldType::Keyword, root.path());
        assert_eq!(output.external_ids, ["doc-7", "doc-90000000"]);
        assert_eq!(output.item_ordinals, [3, 2]);
        assert_eq!(output.reader.n_docs(), 2);
        assert_eq!(output.reader.keyword_at(0).as_deref(), Some(""));
        assert_eq!(output.reader.keyword_at(1).as_deref(), Some("new"));
        assert_eq!(
            output.reader.keyword_postings("new"),
            Some([1].into_iter().collect())
        );
        assert!(output.reader.keyword_postings("old").is_none());
    }
    #[test]
    fn selected_sets_are_sorted_unique_and_keep_empty_presence() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![
            (
                "one",
                FieldValue::StringList(vec!["z".into(), "a".into(), "z".into(), "".into()]),
            ),
            ("two", FieldValue::StringList(vec![])),
        ]);
        let output = staged(&bytes, &[1, 0], FieldType::Set, root.path());
        assert_eq!(output.reader.set_at(0), Some(vec![]));
        assert_eq!(
            output.reader.set_at(1),
            Some(vec!["".into(), "a".into(), "z".into()])
        );
        assert_eq!(
            output.reader.set_postings("z"),
            Some([1].into_iter().collect())
        );
    }
    #[test]
    fn selected_numbers_keep_row_order_and_numeric_key_order() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![
            ("a", FieldValue::Number(-2.5)),
            ("b", FieldValue::Number(9.0)),
            ("c", FieldValue::Number(0.0)),
        ]);
        let output = staged(&bytes, &[1, 0], FieldType::Number, root.path());
        assert_eq!(output.reader.number_at(0), Some(9.0));
        assert_eq!(output.reader.number_at(1), Some(-2.5));
    }
    #[test]
    fn invalid_selection_and_refused_reservation_create_no_files() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![
            ("same", FieldValue::String("one".into())),
            ("same", FieldValue::String("two".into())),
        ]);
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        for (rows, kind, field) in [
            (vec![0], FieldType::Number, "value"),
            (vec![0, 0], FieldType::Keyword, "value"),
            (vec![2], FieldType::Keyword, "value"),
            (vec![0, 1], FieldType::Keyword, "value"),
            (vec![0], FieldType::Keyword, "other"),
        ] {
            assert!(prepare(&scanner, field, &rows, kind, 1, root.path(), |_| Ok(())).is_err());
        }
        assert!(prepare(
            &scanner,
            "value",
            &[0],
            FieldType::Keyword,
            1,
            root.path(),
            |_| Err(anyhow!("reservation refused"))
        )
        .is_err());
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn reader_reservation_refusal_removes_private_output_before_mmap() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![("one", FieldValue::String("value".into()))]);
        let mut calls = 0;
        SegmentReader::reset_owned_stage_open_calls();
        let outcome = prepare(
            &FastIndexScanner::parse(&bytes).unwrap(),
            "value",
            &[0],
            FieldType::Keyword,
            1,
            root.path(),
            |_| {
                calls += 1;
                if calls == 2 {
                    bail!("reader reservation refused")
                }
                Ok(())
            },
        );
        assert!(outcome.is_err());
        assert_eq!(calls, 2);
        assert_eq!(SegmentReader::owned_stage_open_calls(), 0);
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn stage_file_lives_until_last_reader_reference_and_same_size_fields_do_not_collide() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![("one", FieldValue::String("value".into()))]);
        let first = staged(&bytes, &[0], FieldType::Keyword, root.path());
        let second = staged(&bytes, &[0], FieldType::Keyword, root.path());
        let path = first.reader.owned_stage_dir().unwrap().to_owned();
        assert_ne!(Some(path.as_path()), second.reader.owned_stage_dir());
        let held = first.reader.clone();
        drop(first);
        assert!(path.join("field.lseg").exists());
        assert_eq!(held.keyword_at(0).as_deref(), Some("value"));
        drop(held);
        assert!(!path.exists());
        assert!(second.reader.owned_stage_dir().unwrap().exists());
    }
    #[test]
    fn a_real_1000_item_oversized_source_uses_bounded_reservations_and_sparse_files() {
        let root = tempfile::tempdir().unwrap();
        let items = (0..1000)
            .map(|ordinal| IndexItem {
                external_id: format!("doc-{ordinal}"),
                field: "value".into(),
                version: None,
                value: FieldValue::String(format!("{ordinal:04}-{}", "x".repeat(270 * 1024))),
            })
            .collect();
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: None,
                items,
            },
        })
        .encode()
        .unwrap();
        assert!(bytes.len() > crate::change_budget::HARD_LIMIT);
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let rows: Vec<_> = (0..1000).rev().collect();
        let mut peak = 0;
        let output = prepare(
            &scanner,
            "value",
            &rows,
            FieldType::Keyword,
            42,
            root.path(),
            |required| {
                peak = peak.max(required);
                ensure!(
                    required <= crate::change_budget::HARD_LIMIT,
                    "private scalar reservation exceeds 256 MiB"
                );
                Ok(())
            },
        )
        .unwrap();
        assert!(peak > 0);
        assert_eq!(output.reader.n_docs(), 1000);
        for (local, ordinal) in rows.iter().enumerate() {
            assert_eq!(output.external_ids[local], format!("doc-{ordinal}"));
            let expected = format!("{ordinal:04}-{}", "x".repeat(270 * 1024));
            assert_eq!(
                output.reader.keyword_at(local as u32).as_deref(),
                Some(expected.as_str())
            );
        }
    }
    #[test]
    fn existing_full_index_item_limit_can_prepare_small_values_within_the_budget() {
        let root = tempfile::tempdir().unwrap();
        let items = (0..super::super::MAX_INDEX_ITEMS)
            .map(|ordinal| IndexItem {
                external_id: format!("doc-{ordinal}"),
                field: "value".into(),
                version: None,
                value: FieldValue::String("shared".into()),
            })
            .collect();
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: None,
                items,
            },
        })
        .encode()
        .unwrap();
        let rows: Vec<_> = (0..super::super::MAX_INDEX_ITEMS).collect();
        let output = prepare(
            &FastIndexScanner::parse(&bytes).unwrap(),
            "value",
            &rows,
            FieldType::Keyword,
            42,
            root.path(),
            |required| {
                ensure!(
                    required <= crate::change_budget::HARD_LIMIT,
                    "existing full Index item limit must remain reservable for small values"
                );
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            output.reader.n_docs() as usize,
            super::super::MAX_INDEX_ITEMS
        );
        assert_eq!(
            output.reader.keyword_postings("shared").unwrap().len() as usize,
            super::super::MAX_INDEX_ITEMS
        );
    }

    #[test]
    fn validated_replace_fields_can_exceed_index_limit_while_index_prepare_refuses() {
        const REPLACE_DOCUMENTS: usize = 32;
        const FIELDS_PER_DOCUMENT: usize = super::super::MAX_INDEX_ITEMS / REPLACE_DOCUMENTS + 1;
        let root = tempfile::tempdir().unwrap();
        let count = REPLACE_DOCUMENTS * FIELDS_PER_DOCUMENT;
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: None,
                items: (0..count)
                    .map(|ordinal| IndexItem {
                        external_id: format!("doc-{}", ordinal / FIELDS_PER_DOCUMENT),
                        field: format!("field-{}", ordinal % FIELDS_PER_DOCUMENT),
                        value: FieldValue::String("small".into()),
                        version: None,
                    })
                    .collect(),
            },
        })
        .encode()
        .unwrap();
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let rows: Vec<_> = (0..REPLACE_DOCUMENTS)
            .map(|document| document * FIELDS_PER_DOCUMENT)
            .collect();

        let rejected = prepare(
            &scanner,
            "field-0",
            &rows,
            FieldType::Keyword,
            42,
            root.path(),
            |_| Ok(()),
        )
        .unwrap_err();
        assert!(rejected.to_string().contains("Index item limit"));

        let prepared = prepare_validated_fields(
            &scanner,
            "field-0",
            &rows,
            FieldType::Keyword,
            42,
            root.path(),
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(prepared.reader.n_docs() as usize, REPLACE_DOCUMENTS);
    }

    #[test]
    fn one_264_mib_keyword_remains_borrowed_through_private_file_preparation() {
        let root = tempfile::tempdir().unwrap();
        let bytes = command(vec![(
            "large",
            FieldValue::String("x".repeat(264 * 1024 * 1024)),
        )]);
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        assert!(bytes.len() > crate::change_budget::HARD_LIMIT);
        let mut peak = 0;
        let output = prepare(
            &scanner,
            "value",
            &[0],
            FieldType::Keyword,
            9,
            root.path(),
            |need| {
                peak = peak.max(need);
                ensure!(
                    need <= crate::change_budget::HARD_LIMIT,
                    "single raw term must not need a whole-term heap reservation"
                );
                Ok(())
            },
        )
        .unwrap();
        let value = output.reader.keyword_at_cow(0).unwrap();
        assert!(
            matches!(value, std::borrow::Cow::Borrowed(_)),
            "giant staged term must remain mmap borrowed"
        );
        assert_eq!(value.len(), 264 * 1024 * 1024);
        assert!(value.as_bytes().iter().all(|&b| b == b'x'));
        assert_eq!(output.reader.keyword_postings(&value).unwrap().len(), 1);
        assert!(peak > 0);
    }
}
