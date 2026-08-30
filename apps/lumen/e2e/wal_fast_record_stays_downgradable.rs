//! Black-box oracle for the downgrade half of #3952.
//!
//! #3952 added `WAL_FAST_INDEX_VERSIONED` (tag 2) so a fast `Index` record can
//! carry each item's external LWW `version` on the wire. The decode side kept
//! tag 1 working, so an AOF written by an older binary still replays — that
//! direction was covered. The other direction was not: `encode_fast_index`
//! emits tag 2 for EVERY record, including one whose items all carry
//! `version: None` and therefore have nothing new to say, while
//! `WAL_FORMAT_VERSION` stayed at 1.
//!
//! That combination is the hazard. The record's version byte still reads 1, so
//! a pre-#3952 binary's `anyhow::ensure!(version == WAL_FORMAT_VERSION)` passes
//! — the guard that exists precisely to refuse a format it cannot read waves
//! the record through — and the read dies one byte later on
//! `unsupported WAL fast record tag 2`. So rolling 0.4.30 back after it has
//! appended a single `Index` record breaks replay for the whole segment, even
//! for a deployment that never sets `version` and gains nothing from the new
//! tag.
//!
//! The oracle is a hand-rolled pre-#3952 reader: it accepts tag 1 only and
//! parses the legacy layout, which had no per-item version byte. It is the
//! decoder an older binary IS, expressed as a function, so it can be run
//! against the encoder in this tree without shipping an old binary.
//!
//! The control matters as much as the case. A reader that accepted anything
//! would pass case 1 for free, so `a_versioned_record_is_refused_by_the_old_reader`
//! shows the same reader still rejects a record that genuinely uses the new
//! tag — which is correct behaviour, not a regression: that record carries a
//! `version` an old binary cannot honour, and failing loudly beats replaying it
//! with the LWW ceiling silently dropped.

use anyhow::{anyhow, bail, Result};

use lumen::log_entry::RaftLogEntry;
use lumen::types::{FieldValue, IndexItem, IndexRequest};
use lumen::wal::WalRecord;

/// The pre-#3952 fast-record layout, transcribed from the encoder as it stood
/// before the versioned tag existed. Deliberately NOT written in terms of the
/// crate's own constants: this is what a binary in the field already compiled,
/// and a change to those constants must not silently change what "the old
/// reader" means.
mod legacy {
    use super::*;

    const MAGIC: &[u8; 4] = b"LWAL";
    const FORMAT_VERSION: u8 = 1;
    /// The only fast tag a pre-#3952 binary knew how to parse.
    const TAG_INDEX: u8 = 1;

    const VALUE_STRING: u8 = 1;
    const VALUE_NUMBER: u8 = 2;
    const VALUE_VECTOR: u8 = 3;
    const VALUE_STRING_LIST: u8 = 4;

    struct Cursor<'a> {
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> Cursor<'a> {
        fn take(&mut self, len: usize) -> Result<&'a [u8]> {
            let end = self
                .pos
                .checked_add(len)
                .ok_or_else(|| anyhow!("cursor overflow"))?;
            if end > self.bytes.len() {
                bail!("truncated record");
            }
            let out = &self.bytes[self.pos..end];
            self.pos = end;
            Ok(out)
        }

        fn u8(&mut self) -> Result<u8> {
            Ok(self.take(1)?[0])
        }

        fn u32(&mut self) -> Result<usize> {
            let raw: [u8; 4] = self.take(4)?.try_into().expect("4 bytes");
            Ok(u32::from_le_bytes(raw) as usize)
        }

        fn f32(&mut self) -> Result<f32> {
            let raw: [u8; 4] = self.take(4)?.try_into().expect("4 bytes");
            Ok(f32::from_le_bytes(raw))
        }

        fn string(&mut self) -> Result<String> {
            let len = self.u32()?;
            Ok(String::from_utf8(self.take(len)?.to_vec())?)
        }
    }

    /// Decode exactly as a pre-#3952 binary would: reject anything that is not
    /// the legacy fast tag, and parse items with NO per-item version byte.
    pub fn decode(bytes: &[u8]) -> Result<(String, IndexRequest)> {
        let mut cur = Cursor { bytes, pos: 0 };
        if cur.take(MAGIC.len())? != MAGIC {
            bail!("invalid magic");
        }
        let version = cur.u8()?;
        if version != FORMAT_VERSION {
            bail!("unsupported WAL record version {version}");
        }
        let tag = cur.u8()?;
        if tag != TAG_INDEX {
            bail!("unsupported WAL fast record tag {tag}");
        }
        let collection_id = cur.string()?;
        let request_id = match cur.u8()? {
            0 => None,
            1 => Some(cur.string()?),
            other => bail!("invalid request_id tag {other}"),
        };
        let count = cur.u32()?;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let external_id = cur.string()?;
            let field = cur.string()?;
            let value = match cur.u8()? {
                VALUE_STRING => FieldValue::String(cur.string()?),
                VALUE_NUMBER => FieldValue::Number(f64::from_le_bytes(
                    cur.take(8)?.try_into().expect("8 bytes"),
                )),
                VALUE_VECTOR => {
                    let n = cur.u32()?;
                    let mut v = Vec::with_capacity(n);
                    for _ in 0..n {
                        v.push(cur.f32()?);
                    }
                    FieldValue::Vector(v)
                }
                VALUE_STRING_LIST => {
                    let n = cur.u32()?;
                    let mut v = Vec::with_capacity(n);
                    for _ in 0..n {
                        v.push(cur.string()?);
                    }
                    FieldValue::StringList(v)
                }
                other => bail!("unknown value tag {other}"),
            };
            items.push(IndexItem {
                external_id,
                field,
                value,
                // The legacy wire had no version at all; every item it produced
                // reconstructed as `None`.
                version: None,
            });
        }
        if cur.pos != cur.bytes.len() {
            bail!("trailing bytes");
        }
        Ok((collection_id, IndexRequest { items, request_id }))
    }
}

fn record(items: Vec<IndexItem>, request_id: Option<&str>) -> WalRecord {
    WalRecord::new(RaftLogEntry::Index {
        collection_id: "docs".to_string(),
        req: IndexRequest {
            items,
            request_id: request_id.map(str::to_string),
        },
    })
}

fn plain_item(external_id: &str, value: &str) -> IndexItem {
    IndexItem {
        external_id: external_id.to_string(),
        field: "title".to_string(),
        value: FieldValue::String(value.to_string()),
        version: None,
    }
}

/// THE CASE. Every item carries `version: None`, so the record has nothing the
/// legacy wire could not express. It must therefore still be written on the
/// legacy wire, and a pre-#3952 reader must be able to replay it.
#[test]
fn a_version_free_record_still_decodes_on_a_pre_3952_reader() {
    let rec = record(
        vec![
            plain_item("d1", "alpha"),
            IndexItem {
                external_id: "d2".to_string(),
                field: "score".to_string(),
                value: FieldValue::Number(4.5),
                version: None,
            },
            IndexItem {
                external_id: "d3".to_string(),
                field: "tags".to_string(),
                value: FieldValue::StringList(vec!["x".into(), "y".into()]),
                version: None,
            },
            IndexItem {
                external_id: "d4".to_string(),
                field: "embedding".to_string(),
                value: FieldValue::Vector(vec![0.25, -0.5]),
                version: None,
            },
        ],
        Some("req-7"),
    );
    let bytes = rec.encode().expect("encode");

    let (collection_id, req) = legacy::decode(&bytes)
        .expect("a record with no per-item version must stay readable by a pre-#3952 binary");

    assert_eq!(collection_id, "docs");
    assert_eq!(req.request_id.as_deref(), Some("req-7"));
    assert_eq!(req.items.len(), 4);
    assert_eq!(req.items[0].external_id, "d1");
    assert!(matches!(&req.items[0].value, FieldValue::String(s) if s == "alpha"));
    assert!(matches!(&req.items[1].value, FieldValue::Number(n) if *n == 4.5));
    assert!(matches!(&req.items[2].value, FieldValue::StringList(v) if v == &["x", "y"]));
    assert!(matches!(&req.items[3].value, FieldValue::Vector(v) if v == &[0.25, -0.5]));
    assert!(req.items.iter().all(|i| i.version.is_none()));
}

/// THE CONTROL. The same reader must still refuse a record that genuinely uses
/// the new tag — otherwise case 1 proves nothing about which tag was written.
#[test]
fn a_versioned_record_is_refused_by_the_old_reader() {
    let rec = record(
        vec![IndexItem {
            external_id: "d1".to_string(),
            field: "title".to_string(),
            value: FieldValue::String("alpha".to_string()),
            version: Some(9),
        }],
        None,
    );
    let bytes = rec.encode().expect("encode");

    let err = legacy::decode(&bytes)
        .expect_err("a record that actually carries a version must not masquerade as legacy");
    assert!(
        err.to_string().contains("unsupported WAL fast record tag"),
        "expected a tag refusal, got: {err}"
    );
}

/// A record is only downgradable if it is also still correct here: the version
/// must survive the current decoder untouched.
#[test]
fn the_versioned_wire_still_round_trips_through_the_current_decoder() {
    let rec = record(
        vec![
            IndexItem {
                external_id: "d1".to_string(),
                field: "title".to_string(),
                value: FieldValue::String("alpha".to_string()),
                version: Some(9),
            },
            plain_item("d2", "beta"),
        ],
        Some("req-1"),
    );
    let bytes = rec.encode().expect("encode");

    let back = WalRecord::decode(&bytes).expect("decode");
    let RaftLogEntry::Index { collection_id, req } = back.entry else {
        panic!("expected an Index entry");
    };
    assert_eq!(collection_id, "docs");
    assert_eq!(req.request_id.as_deref(), Some("req-1"));
    assert_eq!(req.items[0].version, Some(9));
    assert_eq!(req.items[1].version, None);
}

/// A version-free record must also survive the CURRENT decoder, which is the
/// path a same-version replay actually takes. Without this, a fix that made the
/// legacy reader happy by breaking the modern one would still look green.
#[test]
fn a_version_free_record_round_trips_through_the_current_decoder() {
    let rec = record(vec![plain_item("d1", "alpha"), plain_item("d2", "beta")], None);
    let bytes = rec.encode().expect("encode");

    let back = WalRecord::decode(&bytes).expect("decode");
    let RaftLogEntry::Index { collection_id, req } = back.entry else {
        panic!("expected an Index entry");
    };
    assert_eq!(collection_id, "docs");
    assert_eq!(req.items.len(), 2);
    assert!(req.items.iter().all(|i| i.version.is_none()));
    assert!(matches!(&req.items[0].value, FieldValue::String(s) if s == "alpha"));
}
