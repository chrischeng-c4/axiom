//! Borrowed scanner for a committed v1 fast-Index WAL command.
//!
//! This is deliberately a scanner, not another `WalRecord` decoder. It validates
//! the complete command once, retains no per-item allocation, and can rewind to
//! produce borrowed items again for a later durable staging pass.

use anyhow::{anyhow, Result};

#[cfg(test)]
use std::cell::Cell;

use super::{
    WAL_FAST_INDEX, WAL_FAST_INDEX_VERSIONED, WAL_FAST_MAGIC, WAL_FORMAT_VERSION, WAL_VALUE_NUMBER,
    WAL_VALUE_STRING, WAL_VALUE_STRING_LIST, WAL_VALUE_VECTOR,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    /// # Safety invariant
    ///
    /// Parse validates the full record before it returns a scanner. A later
    /// iterator only reproduces that checked layout over the same immutable
    /// bytes, including the rebased slices used by StringList iteration.
    fn validated_str<'a>(self, bytes: &'a [u8]) -> &'a str {
        // SAFETY: parse validated this record layout and UTF-8 payloads. The
        // scanner retains the same immutable bytes for every iterator pass.
        unsafe { std::str::from_utf8_unchecked(&bytes[self.start..self.end]) }
    }
}

#[cfg(test)]
thread_local! {
    static UTF8_VALIDATION_BYTES: Cell<usize> = const { Cell::new(0) };
}

fn validate_utf8(bytes: &[u8]) -> Result<&str> {
    #[cfg(test)]
    UTF8_VALIDATION_BYTES.with(|total| {
        total.set(
            total
                .get()
                .checked_add(bytes.len())
                .expect("fast WAL UTF-8 validation byte count overflow"),
        );
    });
    std::str::from_utf8(bytes).map_err(|error| anyhow!("invalid WAL fast utf8: {error}"))
}

#[cfg(test)]
fn reset_utf8_validation_bytes_for_test() {
    UTF8_VALIDATION_BYTES.with(|total| total.set(0));
}

#[cfg(test)]
fn utf8_validation_bytes_for_test() -> usize {
    UTF8_VALIDATION_BYTES.with(Cell::get)
}

/// Bounded facts from the validation pass. These are enough for a later stage
/// to reserve scanner metadata and one decoded item chunk without retaining the
/// command's values in RAM.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FastIndexCost {
    pub(crate) wire_bytes: usize,
    pub(crate) item_count: usize,
    pub(crate) max_item_wire_bytes: usize,
    pub(crate) max_value_wire_bytes: usize,
}

/// A validated fast-Index command that borrows the Raft runtime's pinned bytes.
pub(crate) struct FastIndexScanner<'a> {
    bytes: &'a [u8],
    collection: ByteRange,
    request_id: Option<ByteRange>,
    items_start: usize,
    item_count: usize,
    cost: FastIndexCost,
}

impl<'a> FastIndexScanner<'a> {
    /// Accept only v1 fast Index tags. Control records and generic WAL records
    /// are intentionally left for the existing `WalRecord::decode` path.
    pub(crate) fn parse(bytes: &'a [u8]) -> Result<Self> {
        let mut cur = Cursor::new(bytes);
        cur.expect_magic(WAL_FAST_MAGIC)?;
        let version = cur.read_u8()?;
        anyhow::ensure!(
            version == WAL_FORMAT_VERSION,
            "fast Index scanner requires WAL v{} (got {})",
            WAL_FORMAT_VERSION,
            version,
        );
        let tag = cur.read_u8()?;
        anyhow::ensure!(
            tag == WAL_FAST_INDEX || tag == WAL_FAST_INDEX_VERSIONED,
            "fast Index scanner requires Index tag (got {tag})"
        );
        let collection = cur.read_utf8_range()?;
        let request_id = match cur.read_u8()? {
            0 => None,
            1 => Some(cur.read_utf8_range()?),
            other => return Err(anyhow!("invalid WAL fast request_id tag {other}")),
        };
        let item_count = cur.read_u32()? as usize;
        let items_start = cur.pos;
        let mut max_item_wire_bytes = 0usize;
        let mut max_value_wire_bytes = 0usize;
        for _ in 0..item_count {
            let item_start = cur.pos;
            cur.read_utf8_range()?; // external_id
            cur.read_utf8_range()?; // field
            if tag == WAL_FAST_INDEX_VERSIONED {
                match cur.read_u8()? {
                    0 => {}
                    1 => {
                        cur.read_u64()?;
                    }
                    other => return Err(anyhow!("invalid WAL fast item version tag {other}")),
                }
            }
            let value_bytes = cur.scan_value()?;
            max_value_wire_bytes = max_value_wire_bytes.max(value_bytes);
            max_item_wire_bytes = max_item_wire_bytes.max(
                cur.pos
                    .checked_sub(item_start)
                    .ok_or_else(|| anyhow!("WAL fast cursor overflow"))?,
            );
        }
        cur.expect_eof()?;
        Ok(Self {
            bytes,
            collection,
            request_id,
            items_start,
            item_count,
            cost: FastIndexCost {
                wire_bytes: bytes.len(),
                item_count,
                max_item_wire_bytes,
                max_value_wire_bytes,
            },
        })
    }

    pub(crate) fn collection_id(&self) -> &'a str {
        self.collection.validated_str(self.bytes)
    }

    pub(crate) fn request_id(&self) -> Option<&'a str> {
        self.request_id.map(|range| range.validated_str(self.bytes))
    }

    pub(crate) fn cost(&self) -> FastIndexCost {
        self.cost
    }

    /// A new iterator starts at the first item every time. It owns only a byte
    /// position and a remaining count.
    pub(crate) fn items(&self) -> FastIndexItems<'a> {
        FastIndexItems {
            bytes: self.bytes,
            pos: self.items_start,
            remaining: self.item_count,
        }
    }
}

pub(crate) struct FastIndexItems<'a> {
    bytes: &'a [u8],
    pos: usize,
    remaining: usize,
}

impl<'a> Iterator for FastIndexItems<'a> {
    type Item = FastIndexItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let mut cur = Cursor {
            bytes: self.bytes,
            pos: self.pos,
        };
        let external_id = cur
            .read_utf8_range_validated()
            .expect("validated fast WAL external_id");
        let field = cur
            .read_utf8_range_validated()
            .expect("validated fast WAL field");
        // The tag is encoded before `items_start`; determine version presence
        // from the byte at the fixed v1 fast-record tag position.
        let versioned = self.bytes[WAL_FAST_MAGIC.len() + 1] == WAL_FAST_INDEX_VERSIONED;
        let version = if versioned {
            match cur.read_u8().expect("validated fast WAL version tag") {
                0 => None,
                1 => Some(cur.read_u64().expect("validated fast WAL version")),
                _ => unreachable!("scanner validated fast WAL version tag"),
            }
        } else {
            None
        };
        let value = cur
            .read_value_validated()
            .expect("scanner validated fast WAL value");
        self.pos = cur.pos;
        self.remaining -= 1;
        Some(FastIndexItem {
            external_id: external_id.validated_str(self.bytes),
            field: field.validated_str(self.bytes),
            version,
            value,
        })
    }
}

#[derive(Debug)]
pub(crate) struct FastIndexItem<'a> {
    pub(crate) external_id: &'a str,
    pub(crate) field: &'a str,
    pub(crate) version: Option<u64>,
    pub(crate) value: FastIndexValue<'a>,
}

/// Values borrow the command. Vector payloads stay as wire bytes so later code
/// can stream/decode one f32 at a time. A string-list keeps its validated range
/// and can be rewound through `values()`.
#[derive(Debug)]
pub(crate) enum FastIndexValue<'a> {
    String(&'a str),
    Number(f64),
    Vector { values: &'a [u8], len: usize },
    StringList(FastStringList<'a>),
}

impl<'a> FastIndexValue<'a> {
    pub(crate) fn wire_bytes(&self) -> usize {
        match self {
            Self::String(value) => value.len(),
            Self::Number(_) => std::mem::size_of::<f64>(),
            Self::Vector { values, .. } => values.len(),
            Self::StringList(values) => values.bytes.len(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FastStringList<'a> {
    bytes: &'a [u8],
    count: usize,
}

impl<'a> FastStringList<'a> {
    pub(crate) fn len(&self) -> usize {
        self.count
    }

    pub(crate) fn values(self) -> FastStringValues<'a> {
        FastStringValues {
            bytes: self.bytes,
            pos: 0,
            remaining: self.count,
        }
    }
}

pub(crate) struct FastStringValues<'a> {
    bytes: &'a [u8],
    pos: usize,
    remaining: usize,
}

impl<'a> Iterator for FastStringValues<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let mut cur = Cursor {
            bytes: self.bytes,
            pos: self.pos,
        };
        let value = cur
            .read_utf8_range_validated()
            .expect("validated fast WAL string-list value");
        self.pos = cur.pos;
        self.remaining -= 1;
        Some(value.validated_str(self.bytes))
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn expect_magic(&mut self, magic: &[u8]) -> Result<()> {
        anyhow::ensure!(
            self.read_exact(magic.len())? == magic,
            "invalid WAL fast magic"
        );
        Ok(())
    }

    fn expect_eof(&self) -> Result<()> {
        anyhow::ensure!(
            self.pos == self.bytes.len(),
            "trailing bytes in WAL fast record"
        );
        Ok(())
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| anyhow!("WAL fast cursor overflow"))?;
        anyhow::ensure!(end <= self.bytes.len(), "truncated WAL fast record");
        let out = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.read_exact(4)?.try_into().expect("four bytes"),
        ))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.read_exact(8)?.try_into().expect("eight bytes"),
        ))
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(
            self.read_exact(8)?.try_into().expect("eight bytes"),
        ))
    }

    fn read_utf8_range(&mut self) -> Result<ByteRange> {
        let len = self.read_u32()? as usize;
        let start = self.pos;
        let bytes = self.read_exact(len)?;
        validate_utf8(bytes)?;
        Ok(ByteRange {
            start,
            end: self.pos,
        })
    }

    /// Advance a cursor over a range parse already validated. This is private
    /// to iterator paths that borrow an existing `FastIndexScanner` only.
    fn read_utf8_range_validated(&mut self) -> Result<ByteRange> {
        let len = self.read_u32()? as usize;
        let start = self.pos;
        self.read_exact(len)?;
        Ok(ByteRange {
            start,
            end: self.pos,
        })
    }

    /// Validate a value without constructing any user value.
    fn scan_value(&mut self) -> Result<usize> {
        let start = self.pos;
        match self.read_u8()? {
            WAL_VALUE_STRING => {
                self.read_utf8_range()?;
            }
            WAL_VALUE_NUMBER => {
                self.read_exact(8)?;
            }
            WAL_VALUE_VECTOR => {
                let len = self.read_u32()? as usize;
                let bytes = len
                    .checked_mul(std::mem::size_of::<f32>())
                    .ok_or_else(|| anyhow!("WAL fast vector byte length overflow"))?;
                self.read_exact(bytes)?;
            }
            WAL_VALUE_STRING_LIST => {
                let count = self.read_u32()? as usize;
                for _ in 0..count {
                    self.read_utf8_range()?;
                }
            }
            other => return Err(anyhow!("invalid WAL fast field value tag {other}")),
        }
        self.pos
            .checked_sub(start)
            .ok_or_else(|| anyhow!("WAL fast cursor overflow"))
    }

    fn read_value_validated(&mut self) -> Result<FastIndexValue<'a>> {
        match self.read_u8()? {
            WAL_VALUE_STRING => Ok(FastIndexValue::String(
                self.read_utf8_range_validated()?.validated_str(self.bytes),
            )),
            WAL_VALUE_NUMBER => Ok(FastIndexValue::Number(self.read_f64()?)),
            WAL_VALUE_VECTOR => {
                let len = self.read_u32()? as usize;
                let bytes = len
                    .checked_mul(std::mem::size_of::<f32>())
                    .ok_or_else(|| anyhow!("WAL fast vector byte length overflow"))?;
                Ok(FastIndexValue::Vector {
                    values: self.read_exact(bytes)?,
                    len,
                })
            }
            WAL_VALUE_STRING_LIST => {
                let count = self.read_u32()? as usize;
                let start = self.pos;
                for _ in 0..count {
                    self.read_utf8_range_validated()?;
                }
                Ok(FastIndexValue::StringList(FastStringList {
                    bytes: &self.bytes[start..self.pos],
                    count,
                }))
            }
            other => Err(anyhow!("invalid WAL fast field value tag {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{FieldValue, IndexItem, IndexRequest};
    use crate::wal::WalRecord;

    fn all_shapes(versioned: bool) -> Vec<u8> {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: Some("request".into()),
                items: vec![
                    IndexItem {
                        external_id: "one".into(),
                        field: "kw".into(),
                        value: FieldValue::String("value".into()),
                        version: versioned.then_some(7),
                    },
                    IndexItem {
                        external_id: "two".into(),
                        field: "number".into(),
                        value: FieldValue::Number(2.5),
                        version: None,
                    },
                    IndexItem {
                        external_id: "three".into(),
                        field: "vector".into(),
                        value: FieldValue::Vector(vec![1.0, 2.0]),
                        version: None,
                    },
                    IndexItem {
                        external_id: "four".into(),
                        field: "set".into(),
                        value: FieldValue::StringList(vec!["a".into(), "b".into()]),
                        version: None,
                    },
                ],
            },
        })
        .encode()
        .unwrap()
    }

    #[test]
    fn scanner_matches_existing_fast_encoder_for_every_value_shape_and_both_tags() {
        for versioned in [false, true] {
            let bytes = all_shapes(versioned);
            let scanner = FastIndexScanner::parse(&bytes).unwrap();
            assert_eq!(scanner.collection_id(), "docs");
            assert_eq!(scanner.request_id(), Some("request"));
            let items: Vec<_> = scanner.items().collect();
            assert_eq!(items.len(), 4);
            assert_eq!(items[0].external_id, "one");
            assert_eq!(items[0].version, versioned.then_some(7));
            assert!(matches!(items[0].value, FastIndexValue::String("value")));
            assert!(matches!(items[1].value, FastIndexValue::Number(value) if value == 2.5));
            assert!(matches!(
                items[2].value,
                FastIndexValue::Vector { len: 2, .. }
            ));
            let FastIndexValue::StringList(values) = items[3].value else {
                panic!("set")
            };
            assert_eq!(values.values().collect::<Vec<_>>(), ["a", "b"]);
            assert_eq!(
                scanner.items().count(),
                4,
                "iterator must rewind without a descriptor Vec"
            );
        }
    }

    #[test]
    fn scanner_rejects_malformed_fast_index_framing_before_iteration() {
        let good = all_shapes(false);
        for mut bad in [good[..good.len() - 1].to_vec(), good.clone()] {
            if bad.len() == good.len() {
                bad[5] = 99;
            }
            assert!(FastIndexScanner::parse(&bad).is_err());
        }
        let mut bad_utf8 = all_shapes(false);
        // Collection payload starts after magic, version, tag, and u32 length.
        bad_utf8[10] = 0xff;
        assert!(FastIndexScanner::parse(&bad_utf8).is_err());
        let mut trailing = all_shapes(false);
        trailing.push(0);
        assert!(
            FastIndexScanner::parse(&trailing).is_err(),
            "scanner must reject bytes after the final item"
        );
    }

    #[test]
    fn scanner_borrows_the_real_1000_by_270_kib_values() {
        const ITEMS: usize = 1_000;
        const VALUE_BYTES: usize = 270 * 1024;
        let value = "x".repeat(VALUE_BYTES);
        let bytes = WalRecord::new(RaftLogEntry::Index {
            collection_id: "large".into(),
            req: IndexRequest {
                request_id: None,
                items: (0..ITEMS)
                    .map(|ordinal| IndexItem {
                        external_id: format!("id-{ordinal}"),
                        field: "kw".into(),
                        value: FieldValue::String(value.clone()),
                        version: None,
                    })
                    .collect(),
            },
        })
        .encode()
        .unwrap();
        assert!(bytes.len() > 256 * 1024 * 1024);
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        assert_eq!(scanner.cost().item_count, ITEMS);
        // The wire span includes the value tag and u32 string length.
        assert_eq!(scanner.cost().max_value_wire_bytes, VALUE_BYTES + 5);
        let first = scanner.items().next().unwrap();
        let FastIndexValue::String(first_value) = first.value else {
            panic!("keyword")
        };
        let base = bytes.as_ptr() as usize;
        let end = base + bytes.len();
        let value_ptr = first_value.as_ptr() as usize;
        assert!(
            base <= value_ptr && value_ptr + first_value.len() <= end,
            "scanner value must borrow command bytes"
        );
        assert_eq!(scanner.items().count(), ITEMS);
    }
}

#[cfg(test)]
mod validated_iteration_tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::types::{FieldValue, IndexItem, IndexRequest};
    use crate::wal::WalRecord;

    fn all_shapes(versioned: bool) -> Vec<u8> {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: Some("request".into()),
                items: vec![
                    IndexItem {
                        external_id: "one".into(),
                        field: "kw".into(),
                        value: FieldValue::String("value".into()),
                        version: versioned.then_some(7),
                    },
                    IndexItem {
                        external_id: "two".into(),
                        field: "number".into(),
                        value: FieldValue::Number(2.5),
                        version: None,
                    },
                    IndexItem {
                        external_id: "three".into(),
                        field: "vector".into(),
                        value: FieldValue::Vector(vec![1.0, 2.0]),
                        version: None,
                    },
                    IndexItem {
                        external_id: "four".into(),
                        field: "set".into(),
                        value: FieldValue::StringList(vec!["a".into(), "b".into()]),
                        version: None,
                    },
                ],
            },
        })
        .encode()
        .unwrap()
    }

    fn corrupt_first(bytes: &mut [u8], needle: &[u8]) {
        let mut encoded = (needle.len() as u32).to_le_bytes().to_vec();
        encoded.extend_from_slice(needle);
        let offset = bytes
            .windows(encoded.len())
            .position(|window| window == encoded)
            .expect("fixture string must occur once");
        bytes[offset + 4] = 0xff;
    }

    #[test]
    fn parse_refuses_invalid_utf8_in_every_fast_string_position() {
        for needle in [
            b"docs".as_slice(),
            b"request".as_slice(),
            b"one".as_slice(),
            b"kw".as_slice(),
            b"value".as_slice(),
            b"a".as_slice(),
            b"b".as_slice(),
        ] {
            let mut bytes = all_shapes(true);
            corrupt_first(&mut bytes, needle);
            assert!(
                FastIndexScanner::parse(&bytes).is_err(),
                "parse must reject malformed UTF-8 in {needle:?}"
            );
        }
    }

    #[test]
    fn validated_iteration_rewinds_lists_for_both_fast_tags() {
        for versioned in [false, true] {
            let bytes = all_shapes(versioned);
            let scanner = FastIndexScanner::parse(&bytes).unwrap();
            let first: Vec<_> = scanner
                .items()
                .map(|item| match item.value {
                    FastIndexValue::StringList(values) => values.values().collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .collect();
            let second: Vec<_> = scanner
                .items()
                .map(|item| match item.value {
                    FastIndexValue::StringList(values) => values.values().collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .collect();
            assert_eq!(first, second);
            assert_eq!(first[3], ["a", "b"]);
        }
    }

    #[test]
    fn iteration_does_not_repeat_parse_utf8_validation() {
        reset_utf8_validation_bytes_for_test();
        let bytes = all_shapes(true);
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let validated_at_parse = utf8_validation_bytes_for_test();
        const FIXTURE_UTF8_BYTES: usize = "docs".len()
            + "request".len()
            + "one".len()
            + "kw".len()
            + "value".len()
            + "two".len()
            + "number".len()
            + "three".len()
            + "vector".len()
            + "four".len()
            + "set".len()
            + "a".len()
            + "b".len();
        assert_eq!(
            validated_at_parse, FIXTURE_UTF8_BYTES,
            "parse must validate every fixture string exactly once",
        );
        for _ in 0..3 {
            assert_eq!(scanner.collection_id(), "docs");
            assert_eq!(scanner.request_id(), Some("request"));
            for item in scanner.items() {
                if let FastIndexValue::StringList(values) = item.value {
                    assert_eq!(values.values().collect::<Vec<_>>(), ["a", "b"]);
                }
            }
        }
        assert_eq!(
            utf8_validation_bytes_for_test(),
            validated_at_parse,
            "later scanner passes must use only parse-validated ranges"
        );
    }
}
