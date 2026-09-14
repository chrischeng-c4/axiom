//! Private canonical fast-Index spool for one borrowed generic ReplaceDocs record.
//!
//! Values flow directly from `BorrowedReplaceScanner` into a private tempfile.
//! The resulting mmap is the only owned representation of the canonical Index
//! wire; document descriptors continue to borrow the pinned generic source.

use std::io::{BufWriter, Write};

use anyhow::{anyhow, Context, Result};
use memmap2::{Mmap, MmapOptions};
use tempfile::NamedTempFile;

use super::{
    borrowed_replace_scanner::{BorrowedReplaceScanner, BorrowedReplaceValue},
    WAL_FAST_INDEX, WAL_FAST_MAGIC, WAL_FORMAT_VERSION, WAL_VALUE_NUMBER, WAL_VALUE_STRING,
    WAL_VALUE_STRING_LIST, WAL_VALUE_VECTOR,
};

const WRITER_BYTES: usize = 64 * 1024;
const FILE_AND_MAP_METADATA_BYTES: usize = 8192;

/// A source-borrowing full-replacement document span in canonical item order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BorrowedReplaceSpoolDoc<'a> {
    pub(crate) external_id: &'a str,
    pub(crate) version: Option<u64>,
    /// Inclusive canonical Index-item ordinal.
    pub(crate) fields_start: usize,
    /// Exclusive canonical Index-item ordinal.
    pub(crate) fields_end: usize,
}

/// Private file-backed canonical Index bytes plus ordered ReplaceDocs spans.
pub(crate) struct BorrowedReplaceSpool<'a> {
    mmap: Mmap,
    _file: NamedTempFile,
    docs: Vec<BorrowedReplaceSpoolDoc<'a>>,
}

impl<'a> BorrowedReplaceSpool<'a> {
    /// Reserve every simultaneous owned population before allocating: scanner
    /// descriptor retention, document descriptors, the 64 KiB writer, and
    /// fixed tempfile/mmap metadata.  WAL payload values are never copied to
    /// heap memory; they stream into the private file.
    pub(crate) fn prepare(
        scanner: &'a BorrowedReplaceScanner<'a>,
        mut reserve_total: impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        let mut document_count = 0usize;
        let mut item_count = 0usize;
        for doc in scanner.docs() {
            document_count = document_count
                .checked_add(1)
                .ok_or_else(|| anyhow!("replace document count overflow"))?;
            item_count = item_count
                .checked_add(doc.fields().count())
                .ok_or_else(|| anyhow!("replace field count overflow"))?;
        }
        let item_count_u32 =
            u32::try_from(item_count).map_err(|_| anyhow!("fast WAL item count exceeds u32"))?;
        let docs_bytes = document_count
            .checked_mul(std::mem::size_of::<BorrowedReplaceSpoolDoc<'a>>())
            .ok_or_else(|| anyhow!("replace descriptor bytes overflow"))?;
        let required = scanner
            .retained_metadata_bytes()
            .checked_add(docs_bytes)
            .and_then(|value| value.checked_add(WRITER_BYTES))
            .and_then(|value| value.checked_add(FILE_AND_MAP_METADATA_BYTES))
            .ok_or_else(|| anyhow!("replace spool reservation overflow"))?;
        reserve_total(required)?;

        let mut docs = Vec::new();
        docs.try_reserve_exact(document_count)?;
        let mut file = NamedTempFile::new().context("create private ReplaceDocs spool")?;
        {
            let mut out = BufWriter::with_capacity(WRITER_BYTES, file.as_file_mut());
            out.write_all(WAL_FAST_MAGIC)?;
            out.write_all(&[WAL_FORMAT_VERSION, WAL_FAST_INDEX])?;
            write_string(&mut out, scanner.collection_id())?;
            out.write_all(&[0])?; // request_id: None
            out.write_all(&item_count_u32.to_le_bytes())?;

            let mut ordinal = 0usize;
            for doc in scanner.docs() {
                let fields_start = ordinal;
                for field in doc.fields() {
                    write_string(&mut out, doc.external_id())?;
                    write_string(&mut out, field.name())?;
                    match field.value() {
                        BorrowedReplaceValue::String(value) => {
                            out.write_all(&[WAL_VALUE_STRING])?;
                            write_string(&mut out, value)?;
                        }
                        BorrowedReplaceValue::Number(value) => {
                            out.write_all(&[WAL_VALUE_NUMBER])?;
                            out.write_all(&value.to_le_bytes())?;
                        }
                        BorrowedReplaceValue::Vector(values) => {
                            out.write_all(&[WAL_VALUE_VECTOR])?;
                            out.write_all(
                                &u32::try_from(values.len())
                                    .map_err(|_| anyhow!("fast WAL vector count exceeds u32"))?
                                    .to_le_bytes(),
                            )?;
                            for value in values {
                                out.write_all(&value?.to_le_bytes())?;
                            }
                        }
                        BorrowedReplaceValue::StringList(values) => {
                            out.write_all(&[WAL_VALUE_STRING_LIST])?;
                            out.write_all(
                                &u32::try_from(values.len())
                                    .map_err(|_| anyhow!("fast WAL set count exceeds u32"))?
                                    .to_le_bytes(),
                            )?;
                            for value in values {
                                write_string(&mut out, value?)?;
                            }
                        }
                    }
                    ordinal = ordinal
                        .checked_add(1)
                        .ok_or_else(|| anyhow!("replace field ordinal overflow"))?;
                }
                docs.push(BorrowedReplaceSpoolDoc {
                    external_id: doc.external_id(),
                    version: doc.version(),
                    fields_start,
                    fields_end: ordinal,
                });
            }
            debug_assert_eq!(ordinal, item_count);
            out.flush()?;
        }
        file.as_file().sync_all()?;
        // SAFETY: this private file is completely flushed and is never written
        // again while its mmap is retained by this spool.
        let mmap =
            unsafe { MmapOptions::new().map(file.as_file()) }.context("mmap ReplaceDocs spool")?;
        Ok(Self {
            mmap,
            _file: file,
            docs,
        })
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.mmap
    }
    pub(crate) fn descriptors(&self) -> &[BorrowedReplaceSpoolDoc<'a>] {
        &self.docs
    }
}

fn write_string(out: &mut dyn Write, value: &str) -> Result<()> {
    let len = u32::try_from(value.len()).map_err(|_| anyhow!("fast WAL string exceeds u32"))?;
    out.write_all(&len.to_le_bytes())?;
    out.write_all(value.as_bytes())?;
    Ok(())
}

// Append inside `wal::borrowed_replace_spool` after registering the scanner.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::bail;

    use crate::{
        log_entry::RaftLogEntry,
        types::{FieldValue, ReplaceDocItem, ReplaceDocsRequest},
        wal::WalRecord,
        wal::{
            borrowed_replace_scanner::BorrowedReplaceScanner,
            fast_index_scanner::{FastIndexScanner, FastIndexValue},
        },
    };

    use super::BorrowedReplaceSpool;

    fn cbor(record: &WalRecord) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(record, &mut bytes).unwrap();
        bytes
    }

    fn fixture() -> Vec<u8> {
        cbor(&WalRecord {
            version: 1,
            entry: RaftLogEntry::ReplaceDocs {
                collection_id: "docs".into(),
                req: ReplaceDocsRequest {
                    docs: vec![
                        ReplaceDocItem {
                            external_id: "same".into(),
                            version: Some(4),
                            fields: BTreeMap::from([
                                ("text".into(), FieldValue::String("雪".into())),
                                ("number".into(), FieldValue::Number(-2.5)),
                                ("vector".into(), FieldValue::Vector(vec![1.0, -0.0])),
                                (
                                    "set".into(),
                                    FieldValue::StringList(vec!["a".into(), "b".into()]),
                                ),
                            ]),
                        },
                        ReplaceDocItem {
                            external_id: "same".into(),
                            version: Some(5),
                            fields: BTreeMap::new(),
                        },
                    ],
                },
            },
        })
    }

    #[test]
    fn streams_sorted_fields_to_unversioned_fast_index_and_keeps_empty_duplicate_docs() {
        let bytes = fixture();
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let spool = BorrowedReplaceSpool::prepare(&scanner, |_| Ok(())).unwrap();
        let fast = FastIndexScanner::parse(spool.bytes()).unwrap();
        assert_eq!(fast.collection_id(), "docs");
        assert_eq!(fast.request_id(), None);
        let items: Vec<_> = fast.items().collect();
        assert_eq!(items.len(), 4);
        assert!(items.iter().all(|item| item.version.is_none()));
        assert_eq!(
            items.iter().map(|item| item.field).collect::<Vec<_>>(),
            ["number", "set", "text", "vector"]
        );
        assert!(matches!(&items[0].value, FastIndexValue::Number(value) if *value == -2.5));
        assert!(matches!(&items[2].value, FastIndexValue::String("雪")));
        assert!(matches!(
            &items[3].value,
            FastIndexValue::Vector { len: 2, .. }
        ));
        assert_eq!(
            spool.descriptors(),
            &[
                super::BorrowedReplaceSpoolDoc {
                    external_id: "same",
                    version: Some(4),
                    fields_start: 0,
                    fields_end: 4
                },
                super::BorrowedReplaceSpoolDoc {
                    external_id: "same",
                    version: Some(5),
                    fields_start: 4,
                    fields_end: 4
                },
            ]
        );
    }

    #[test]
    fn reservation_refusal_happens_before_tempfile_or_writer_allocation() {
        let bytes = fixture();
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let error = BorrowedReplaceSpool::prepare(&scanner, |_| bail!("spool reservation refused"))
            .err()
            .expect("reservation must refuse");
        assert!(error.to_string().contains("spool reservation refused"));
    }
}
