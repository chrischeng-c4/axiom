//! Price decoded WAL storage before allocating it. The fast scanner borrows the
//! frame. Generic serde visitors retain scalar costs, never a record or array.
//! Callers reserve the scanner bound before typed decoding, then
//! retain the returned decoded peak plus the transport's actual byte owners.

use crate::log_entry::RaftLogEntry;
use crate::types::{IndexItem, MAX_BATCH_UNINDEX_DOCS_SIZE};
use anyhow::{bail, ensure, Result};
use std::mem::size_of;

#[cfg(test)]
mod allocation_tests;
mod bounded_serde;
mod generic;
mod token_stats;

const HEADER: usize = 2 * size_of::<usize>();
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b)
        .ok_or_else(|| anyhow::anyhow!("WAL memory bound overflow"))
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b)
        .ok_or_else(|| anyhow::anyhow!("WAL memory bound overflow"))
}
fn allocation(bytes: usize) -> Result<usize> {
    if bytes == 0 {
        Ok(0)
    } else {
        add(bytes, HEADER)
    }
}

/// Generic CBOR text growth and JSON escaped-token scratch can allocate while
/// scanning. Six wire lengths cover their old/new token buffers; CBOR's fixed
/// scratch and the typed visitor stack have a separate fixed allowance.
pub(crate) fn scan_workspace_bytes(encoded_len: usize) -> Result<usize> {
    add(mul(encoded_len, 6)?, 8192)
}

/// The first pass borrows token bytes and keeps only counters. It does not
/// create token buffers. The returned bound covers the typed cost visitor's
/// largest growing token, including the old buffer during reallocation.
pub(crate) fn scan_workspace_bound(bytes: &[u8]) -> Result<usize> {
    if bytes.starts_with(b"LWAL") {
        return Ok(8192);
    }
    let stats = token_stats::scan(bytes)?;
    add(allocation(mul(stats.largest_token_bytes.max(8), 3)?)?, 8192)
}

/// Validate one complete generic CBOR item without creating a decoded payload.
/// This is for a private durable-stage receipt. Public WAL preflight keeps its
/// historical CBOR-or-JSON and suffix-tolerant behavior through `scan`.
pub(crate) fn validate_exact_cbor(bytes: &[u8]) -> Result<()> {
    token_stats::scan_exact_cbor(bytes).map(|_| ())
}

pub(crate) fn decoded_peak_bound(bytes: &[u8]) -> Result<usize> {
    if !bytes.starts_with(b"LWAL") {
        return generic::decoded_peak_bound(bytes);
    }
    let mut c = Cursor { bytes, at: 4 };
    let version = c.u8()?;
    let tag = c.u8()?;
    let mut total = size_of::<RaftLogEntry>();
    match (version, tag) {
        (1, 1 | 2) => {
            total = add(total, allocation(c.string()?.len())?)?;
            match c.u8()? {
                0 => (),
                1 => total = add(total, allocation(c.string()?.len())?)?,
                _ => bail!("invalid WAL fast request_id tag"),
            }
            let count = c.u32()? as usize;
            total = add(total, allocation(mul(count, size_of::<IndexItem>())?)?)?;
            for _ in 0..count {
                total = add(total, allocation(c.string()?.len())?)?;
                total = add(total, allocation(c.string()?.len())?)?;
                if tag == 2 {
                    match c.u8()? {
                        0 => (),
                        1 => {
                            c.take(8)?;
                        }
                        _ => bail!("invalid WAL fast item version tag"),
                    }
                }
                total = add(
                    total,
                    match c.u8()? {
                        1 => allocation(c.string()?.len())?,
                        2 => {
                            c.take(8)?;
                            0
                        }
                        3 => {
                            let bytes = mul(c.u32()? as usize, size_of::<f32>())?;
                            c.take(bytes)?;
                            allocation(bytes)?
                        }
                        4 => {
                            let count = c.u32()? as usize;
                            let mut list = allocation(mul(count, size_of::<String>())?)?;
                            for _ in 0..count {
                                list = add(list, allocation(c.string()?.len())?)?;
                            }
                            list
                        }
                        _ => bail!("invalid WAL fast field value tag"),
                    },
                )?;
            }
        }
        (2, 3) => total = add(total, allocation(c.string()?.len())?)?,
        (2, 4) => {
            total = add(total, allocation(c.string()?.len())?)?;
            let count = c.u32()? as usize;
            ensure!(
                (1..=MAX_BATCH_UNINDEX_DOCS_SIZE).contains(&count),
                "invalid WAL UnindexDocs item count"
            );
            total = add(total, allocation(mul(count, size_of::<String>())?)?)?;
            let ids_start = c.at;
            // The production decoder keeps cloned IDs in a BTreeSet while it
            // validates uniqueness. Charge one full B-tree node per ID plus
            // the original and cloned strings. Compare borrowed ranges here.
            let node = allocation(add(
                mul(11, size_of::<String>())?,
                mul(12, size_of::<usize>())?,
            )?)?;
            for i in 0..count {
                let id = c.string()?;
                let mut previous = Cursor {
                    bytes,
                    at: ids_start,
                };
                for _ in 0..i {
                    ensure!(
                        previous.string()? != id,
                        "duplicate external_id in WAL UnindexDocs record"
                    );
                }
                total = add(total, add(mul(2, allocation(id.len())?)?, node)?)?;
            }
        }
        (1 | 2, _) => bail!("unsupported WAL fast record tag"),
        _ => bail!("unsupported WAL fast record version"),
    }
    ensure!(c.at == bytes.len(), "trailing bytes in WAL fast record");
    Ok(total)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = add(self.at, n)?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| anyhow::anyhow!("truncated WAL fast record"))?;
        self.at = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> Result<&'a str> {
        let n = self.u32()? as usize;
        Ok(std::str::from_utf8(self.take(n)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Engine;
    use crate::types::*;
    use crate::wal::WalRecord;
    use std::collections::BTreeMap;

    fn verify(record: &WalRecord, bytes: &[u8]) {
        let price = decoded_peak_bound(bytes).unwrap();
        let decoded = WalRecord::decode(bytes).unwrap();
        let owned = Engine::record_owned_bytes(&decoded.entry).unwrap();
        assert!(
            price >= owned,
            "decoded {owned} exceeds pre-decode bound {price}"
        );
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(record).unwrap()
        );
    }

    #[test]
    fn large_generic_token_bound_fits_pending_budget_without_content_copies() {
        let record = WalRecord::new(RaftLogEntry::Index {
            collection_id: "large-compatible".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "document".into(),
                    field: "keyword".into(),
                    value: FieldValue::String("x".repeat(60 * 1024 * 1024)),
                    version: None,
                }],
                request_id: None,
            },
        });
        for json in [false, true] {
            let mut bytes = Vec::new();
            if json {
                serde_json::to_writer(&mut bytes, &record).unwrap();
            } else {
                ciborium::ser::into_writer(&record, &mut bytes).unwrap();
            }
            assert!(bytes.len() < storage_durable::MAX_FRAME_PAYLOAD_BYTES);
            let bound = decoded_peak_bound(&bytes).unwrap();
            assert!(bound >= Engine::record_owned_bytes(&record.entry).unwrap());
            assert!(
                bound <= crate::change_budget::HARD_LIMIT,
                "one valid legacy frame must fit the change budget: json={json}, bound={bound}"
            );
        }
    }

    #[test]
    fn wire_bound_covers_tiny_and_large_fast_values_and_controls() {
        for count in [1, 33, 1000] {
            for value in [
                FieldValue::String(String::new()),
                FieldValue::Number(1.0),
                FieldValue::Vector(vec![0.25; 4097]),
                FieldValue::StringList(vec!["x".into(); 33]),
            ] {
                for version in [None, Some(7)] {
                    let record = WalRecord::new(RaftLogEntry::Index {
                        collection_id: "c".into(),
                        req: IndexRequest {
                            items: (0..count)
                                .map(|i| IndexItem {
                                    external_id: i.to_string(),
                                    field: "f".into(),
                                    value: value.clone(),
                                    version,
                                })
                                .collect(),
                            request_id: Some("request".into()),
                        },
                    });
                    verify(&record, &record.encode().unwrap());
                }
            }
        }
        for entry in [
            RaftLogEntry::TruncateDocs {
                collection_id: "c".into(),
            },
            RaftLogEntry::UnindexDocs {
                collection_id: "c".into(),
                req: BatchUnindexDocsRequest {
                    external_ids: (0..1000).map(|i| i.to_string()).collect(),
                },
            },
        ] {
            let record = WalRecord::new(entry);
            verify(&record, &record.encode().unwrap());
        }
    }

    #[test]
    fn wire_bound_covers_generic_variants_and_untagged_value_scratch() {
        let spec: FieldSpec = serde_json::from_str(r#"{"type":"keyword"}"#).unwrap();
        let mut entries = vec![
            RaftLogEntry::CreateCollection {
                collection_id: "c".into(),
                req: CreateCollectionRequest {
                    fields: BTreeMap::from([("f".into(), spec.clone())]),
                },
            },
            RaftLogEntry::Delete {
                collection_id: "c".into(),
                external_id: "d".into(),
                field: Some("f".into()),
            },
            RaftLogEntry::DropCollection {
                collection_id: "c".into(),
                force: false,
            },
            RaftLogEntry::AddField {
                collection_id: "c".into(),
                field_name: "f".into(),
                spec,
            },
            RaftLogEntry::DropField {
                collection_id: "c".into(),
                field_name: "f".into(),
            },
        ];
        for len in [0, 1, 33, 5001] {
            for value in [
                FieldValue::String("x".repeat(len)),
                FieldValue::Vector(vec![0.5; len]),
                FieldValue::StringList(vec!["escaped\n\"\t".into(); len]),
            ] {
                entries.push(RaftLogEntry::ReplaceDocs {
                    collection_id: "c".into(),
                    req: ReplaceDocsRequest {
                        docs: vec![ReplaceDocItem {
                            external_id: "d".into(),
                            version: Some(1),
                            fields: BTreeMap::from([("f".into(), value.clone())]),
                        }],
                    },
                });
                entries.push(RaftLogEntry::Index {
                    collection_id: "c".into(),
                    req: IndexRequest {
                        items: vec![IndexItem {
                            external_id: "d".into(),
                            field: "f".into(),
                            value,
                            version: None,
                        }],
                        request_id: None,
                    },
                });
            }
        }
        for entry in entries {
            let record = WalRecord::new(entry);
            let mut cbor = Vec::new();
            ciborium::ser::into_writer(&record, &mut cbor).unwrap();
            verify(&record, &cbor);
            verify(&record, &serde_json::to_vec(&record).unwrap());
        }
    }

    #[test]
    fn malformed_wire_cannot_allocate_from_a_declared_count_or_unknown_version() {
        let record = WalRecord::new(RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![],
                request_id: None,
            },
        });
        let mut bytes = record.encode().unwrap();
        let end = bytes.len();
        bytes[end - 4..].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decoded_peak_bound(&bytes).is_err());
        for version in [0, 3, 255] {
            bytes[4] = version;
            assert!(decoded_peak_bound(&bytes).is_err());
        }
        assert!(scan_workspace_bytes(usize::MAX).is_err());
        assert!(decoded_peak_bound(
            br#"{"version":99,"entry":{"DropField":{"collection_id":"c","field_name":"f"}}}"#
        )
        .is_err());
        assert!(decoded_peak_bound(
            br#"{"version":1,"entry":{"TruncateDocs":{"collection_id":"c"}}}"#
        )
        .is_err());
    }
}
