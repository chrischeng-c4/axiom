//! Borrowed scanner for the CBOR v1 `ReplaceDocs` WAL entry.
//!
//! The scanner records only byte ranges and fixed-size descriptors.  It never
//! creates owned field strings, string-list members, or vector payloads.  The
//! ranges are valid only while the immutable source frame remains pinned.
//!
//! Indefinite-length text is deliberately `NotHandled`: CBOR permits it, but
//! lending it as one `str` needs a staged concatenation file.  The caller can
//! retain the existing owned decoder for that encoding until that slice exists.

use std::{error::Error, fmt, mem};

#[cfg(test)]
use std::cell::Cell;

use anyhow::{anyhow, bail, ensure, Result};

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
                .expect("UTF-8 validation counter overflow"),
        )
    });
    std::str::from_utf8(bytes).map_err(|_| anyhow!("invalid UTF-8 in CBOR text"))
}

#[cfg(test)]
mod numeric_parity_tests {
    use super::*;
    use crate::{log_entry::RaftLogEntry, types::FieldValue, wal::WalRecord};
    use ciborium::Value;

    #[test]
    fn integer_vector_components_keep_the_owned_decoders_single_rounding() {
        let text = |s: &str| Value::Text(s.into());
        let values = [
            (1u64 << 54) + (1 << 30) + 1,
            (1u64 << 54) + (1 << 30) - 1,
            u64::MAX,
            1u64 << 63,
        ];
        let wire = Value::Map(vec![
            (text("version"), Value::Integer(1.into())),
            (
                text("entry"),
                Value::Map(vec![(
                    text("ReplaceDocs"),
                    Value::Map(vec![
                        (text("collection_id"), text("docs")),
                        (
                            text("req"),
                            Value::Map(vec![(
                                text("docs"),
                                Value::Array(vec![Value::Map(vec![
                                    (text("external_id"), text("id")),
                                    (
                                        text("fields"),
                                        Value::Map(vec![(
                                            text("vector"),
                                            Value::Array(
                                                values
                                                    .into_iter()
                                                    .map(|v| Value::Integer(v.into()))
                                                    .collect(),
                                            ),
                                        )]),
                                    ),
                                ])]),
                            )]),
                        ),
                    ]),
                )]),
            ),
        ]);
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&wire, &mut bytes).unwrap();
        let RaftLogEntry::ReplaceDocs { req, .. } = WalRecord::decode(&bytes).unwrap().entry else {
            panic!("fixture is replacement")
        };
        let FieldValue::Vector(expected) = &req.docs[0].fields["vector"] else {
            panic!("owned decoder returns vector")
        };
        let scanned = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let field = scanned.docs().next().unwrap().fields().next().unwrap();
        let BorrowedReplaceValue::Vector(actual) = field.value() else {
            panic!("borrowed vector")
        };
        assert_eq!(
            actual.map(|v| v.unwrap().to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
}

#[cfg(test)]
fn reset_utf8_validation_bytes_for_test() {
    UTF8_VALIDATION_BYTES.with(|total| total.set(0));
}
#[cfg(test)]
fn utf8_validation_bytes_for_test() -> usize {
    UTF8_VALIDATION_BYTES.with(Cell::get)
}

#[derive(Debug)]
struct NotHandled(&'static str);

impl fmt::Display for NotHandled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "borrowed ReplaceDocs scanner does not handle {}", self.0)
    }
}

impl Error for NotHandled {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    fn as_str<'a>(self, bytes: &'a [u8]) -> &'a str {
        // Every ByteRange is built by `Cursor::text`, which validates this
        // exact immutable byte span before storing it.
        unsafe { std::str::from_utf8_unchecked(&bytes[self.start..self.end]) }
    }
}

#[derive(Clone, Copy, Debug)]
struct ArrayRange {
    start: usize,
    end: usize,
    len: usize,
}

#[derive(Debug)]
enum DescriptorValue {
    String(ByteRange),
    Number(f64),
    Vector(ArrayRange),
    StringList(ArrayRange),
}

#[derive(Debug)]
struct FieldDescriptor {
    name: ByteRange,
    value: DescriptorValue,
}

#[derive(Debug)]
struct DocDescriptor {
    external_id: ByteRange,
    version: Option<u64>,
    fields: Vec<FieldDescriptor>,
}

/// A parsed v1 generic `ReplaceDocs` record borrowing its original WAL bytes.
#[derive(Debug)]
pub(crate) struct BorrowedReplaceScanner<'a> {
    bytes: &'a [u8],
    collection_id: ByteRange,
    docs: Vec<DocDescriptor>,
    consumed_bytes: usize,
    retained_metadata_bytes: usize,
}

/// The field-value shape exposed by a document view.
pub(crate) enum BorrowedReplaceValue<'a> {
    String(&'a str),
    Number(f64),
    Vector(BorrowedVectorIter<'a>),
    StringList(BorrowedStringListIter<'a>),
}

/// One document in source order.
pub(crate) struct BorrowedReplaceDoc<'a> {
    bytes: &'a [u8],
    descriptor: &'a DocDescriptor,
}

/// One field in BTreeMap (lexical key) order.
pub(crate) struct BorrowedReplaceField<'a> {
    bytes: &'a [u8],
    descriptor: &'a FieldDescriptor,
}

impl<'a> BorrowedReplaceScanner<'a> {
    /// Returns `Ok(None)` for a non-CBOR record or an encoding that needs an
    /// owned/staged fallback.  `reserve` is called before each descriptor
    /// vector grows, with the complete next capacity cost.
    pub(crate) fn scan(
        bytes: &'a [u8],
        mut reserve: impl FnMut(usize) -> Result<()>,
    ) -> Result<Option<Self>> {
        // Match the owned generic route before lending any range.  This
        // bounded preflight checks recursive CBOR, malformed breaks/chunks,
        // all UTF-8, and the decoder's 256-level nesting boundary.
        crate::wal_wire_cost::scan_workspace_bound(bytes)?;
        if bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            .is_some_and(|byte| matches!(byte, b'{' | b'['))
        {
            return Ok(None); // legacy JSON remains on the existing decoder.
        }
        let mut accounting = MetadataAccounting::default();
        match Self::scan_cbor(bytes, &mut reserve, &mut accounting) {
            Ok(scanner) => Ok(Some(scanner)),
            Err(error) if error.downcast_ref::<NotHandled>().is_some() => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn scan_cbor(
        bytes: &'a [u8],
        reserve: &mut impl FnMut(usize) -> Result<()>,
        accounting: &mut MetadataAccounting,
    ) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let mut version = None;
        let mut entry = None;
        // This visits every value once.  In particular, a schema-invalid
        // field may later be a storage error, but malformed CBOR anywhere in
        // this record must still fail before an adapter can apply a prefix.
        cursor.map_each(|key, cursor| {
            match key {
                "version" => {
                    ensure!(version.is_none(), "duplicate generic WAL version");
                    version = Some(cursor.unsigned()?);
                }
                "entry" => {
                    ensure!(entry.is_none(), "duplicate generic WAL entry");
                    entry = Some(parse_entry(cursor, reserve, accounting)?);
                }
                _ => cursor.skip()?,
            }
            Ok(())
        })?;
        let version = version.ok_or_else(|| anyhow!("generic WAL record misses version"))?;
        if version != 1 {
            return Err(NotHandled("generic WAL version").into());
        }
        let mut scanner = entry
            .ok_or_else(|| anyhow!("generic WAL record misses entry"))?
            .ok_or_else(|| NotHandled("generic WAL entry"))?;
        scanner.consumed_bytes = cursor.pos;
        scanner.retained_metadata_bytes = accounting.retained_bytes;
        Ok(scanner)
    }

    pub(crate) fn collection_id(&self) -> &str {
        self.collection_id.as_str(self.bytes)
    }

    /// Byte length of the one CBOR item that was validated.  Generic replay
    /// may ignore a suffix like ciborium does; a private framed caller can
    /// require this to equal its decoded CBOR payload length.
    pub(crate) fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }

    pub(crate) fn docs(&self) -> impl Iterator<Item = BorrowedReplaceDoc<'_>> {
        self.docs.iter().map(|descriptor| BorrowedReplaceDoc {
            bytes: self.bytes,
            descriptor,
        })
    }

    pub(crate) fn retained_metadata_bytes(&self) -> usize {
        self.retained_metadata_bytes
    }
}

impl<'a> BorrowedReplaceDoc<'a> {
    pub(crate) fn external_id(&self) -> &'a str {
        self.descriptor.external_id.as_str(self.bytes)
    }
    pub(crate) fn version(&self) -> Option<u64> {
        self.descriptor.version
    }
    pub(crate) fn fields(&self) -> impl Iterator<Item = BorrowedReplaceField<'a>> {
        self.descriptor
            .fields
            .iter()
            .map(|descriptor| BorrowedReplaceField {
                bytes: self.bytes,
                descriptor,
            })
    }
}

impl<'a> BorrowedReplaceField<'a> {
    pub(crate) fn name(&self) -> &'a str {
        self.descriptor.name.as_str(self.bytes)
    }
    pub(crate) fn value(&self) -> BorrowedReplaceValue<'a> {
        match self.descriptor.value {
            DescriptorValue::String(range) => {
                BorrowedReplaceValue::String(range.as_str(self.bytes))
            }
            DescriptorValue::Number(number) => BorrowedReplaceValue::Number(number),
            DescriptorValue::Vector(range) => {
                BorrowedReplaceValue::Vector(BorrowedVectorIter::new(self.bytes, range))
            }
            DescriptorValue::StringList(range) => {
                BorrowedReplaceValue::StringList(BorrowedStringListIter::new(self.bytes, range))
            }
        }
    }
}

/// Re-parses one validated vector range with constant-size state.
#[derive(Clone)]
pub(crate) struct BorrowedVectorIter<'a> {
    cursor: Cursor<'a>,
    end: usize,
    len: usize,
}
impl<'a> BorrowedVectorIter<'a> {
    fn new(bytes: &'a [u8], range: ArrayRange) -> Self {
        Self {
            cursor: Cursor {
                bytes,
                pos: range.start,
            },
            end: range.end,
            len: range.len,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}
impl Iterator for BorrowedVectorIter<'_> {
    type Item = Result<f32>;
    fn next(&mut self) -> Option<Self::Item> {
        (self.cursor.pos < self.end).then(|| self.cursor.number_f32())
    }
}

/// Re-parses one validated string-list range with constant-size state.
#[derive(Clone)]
pub(crate) struct BorrowedStringListIter<'a> {
    cursor: Cursor<'a>,
    end: usize,
    len: usize,
}
impl<'a> BorrowedStringListIter<'a> {
    fn new(bytes: &'a [u8], range: ArrayRange) -> Self {
        Self {
            cursor: Cursor {
                bytes,
                pos: range.start,
            },
            end: range.end,
            len: range.len,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}
impl<'a> Iterator for BorrowedStringListIter<'a> {
    type Item = Result<&'a str>;
    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.cursor.bytes;
        // Parse-time and token preflight already validated this span.  A
        // rewind must not rescan a giant list member as UTF-8.
        (self.cursor.pos < self.end)
            .then(|| self.cursor.trusted_text().map(|range| range.as_str(bytes)))
    }
}

fn parse_entry<'a>(
    cursor: &mut Cursor<'a>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<Option<BorrowedReplaceScanner<'a>>> {
    let mut replace = None;
    let mut variants = 0usize;
    cursor.map_each(|key, cursor| {
        variants = variants
            .checked_add(1)
            .ok_or_else(|| anyhow!("entry variant count overflow"))?;
        if key == "ReplaceDocs" {
            ensure!(replace.is_none(), "duplicate ReplaceDocs entry");
            replace = Some(parse_replace_docs(cursor, reserve, accounting)?);
        } else {
            cursor.skip()?;
        }
        Ok(())
    })?;
    ensure!(
        variants == 1,
        "externally tagged entry must contain exactly one variant"
    );
    Ok(replace)
}

fn parse_replace_docs<'a>(
    cursor: &mut Cursor<'a>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<BorrowedReplaceScanner<'a>> {
    let mut collection_id = None;
    let mut docs = None;
    cursor.map_each(|key, cursor| {
        match key {
            "collection_id" => {
                ensure!(collection_id.is_none(), "duplicate collection_id");
                collection_id = Some(cursor.text()?);
            }
            "req" => {
                ensure!(docs.is_none(), "duplicate ReplaceDocs req");
                docs = Some(parse_request(cursor, reserve, accounting)?);
            }
            _ => cursor.skip()?,
        }
        Ok(())
    })?;
    Ok(BorrowedReplaceScanner {
        bytes: cursor.bytes,
        collection_id: collection_id.ok_or_else(|| anyhow!("ReplaceDocs misses collection_id"))?,
        docs: docs.ok_or_else(|| anyhow!("ReplaceDocs misses req"))?,
        consumed_bytes: 0,
        retained_metadata_bytes: 0,
    })
}

fn parse_request(
    cursor: &mut Cursor<'_>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<Vec<DocDescriptor>> {
    let mut docs = None;
    cursor.map_each(|key, cursor| {
        if key == "docs" {
            ensure!(docs.is_none(), "duplicate docs");
            docs = Some(parse_docs(cursor, reserve, accounting)?);
        } else {
            cursor.skip()?;
        }
        Ok(())
    })?;
    docs.ok_or_else(|| anyhow!("ReplaceDocs request misses docs"))
}

fn parse_docs(
    cursor: &mut Cursor<'_>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<Vec<DocDescriptor>> {
    let range = cursor.array()?;
    let mut docs = Vec::new();
    while cursor.pos < range.end {
        reserve_growth(&mut docs, reserve, accounting)?;
        docs.push(parse_doc(cursor, reserve, accounting)?);
    }
    cursor.finish(range)?;
    Ok(docs)
}

fn parse_doc(
    cursor: &mut Cursor<'_>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<DocDescriptor> {
    let mut external_id = None;
    let mut version = None;
    let mut fields = None;
    cursor.map_each(|key, cursor| {
        match key {
            "external_id" => {
                ensure!(external_id.is_none(), "duplicate external_id");
                external_id = Some(cursor.text()?);
            }
            "version" => {
                ensure!(version.is_none(), "duplicate version");
                version = Some(cursor.option_unsigned()?);
            }
            "fields" => {
                ensure!(fields.is_none(), "duplicate fields");
                fields = Some(parse_fields(cursor, reserve, accounting)?);
            }
            _ => cursor.skip()?,
        }
        Ok(())
    })?;
    Ok(DocDescriptor {
        external_id: external_id.ok_or_else(|| anyhow!("replace doc misses external_id"))?,
        version: version.unwrap_or(None),
        fields: fields.ok_or_else(|| anyhow!("replace doc misses fields"))?,
    })
}

fn parse_fields(
    cursor: &mut Cursor<'_>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<Vec<FieldDescriptor>> {
    let mut fields = Vec::<FieldDescriptor>::new();
    cursor.map_each(|key, cursor| {
        let value = parse_value(cursor)?;
        let start = key.as_ptr() as usize - cursor.bytes.as_ptr() as usize;
        let name = ByteRange {
            start,
            end: start + key.len(),
        };
        if let Some(position) = fields
            .iter()
            .position(|field| field.name.as_str(cursor.bytes) == key)
        {
            fields[position].value = value; // serde BTreeMap keeps the last duplicate key.
        } else {
            reserve_growth(&mut fields, reserve, accounting)?;
            fields.push(FieldDescriptor { name, value });
        }
        Ok(())
    })?;
    fields.sort_unstable_by(|left, right| {
        left.name
            .as_str(cursor.bytes)
            .cmp(right.name.as_str(cursor.bytes))
    });
    Ok(fields)
}

fn parse_value(cursor: &mut Cursor<'_>) -> Result<DescriptorValue> {
    let head = cursor.peek_tagged()?;
    match head.major {
        0 | 1 | 7 => Ok(DescriptorValue::Number(cursor.number()?)),
        3 => Ok(DescriptorValue::String(cursor.text()?)),
        4 => {
            let container = cursor.array()?;
            let range = ArrayRange {
                start: container.start,
                end: container.end,
                len: container.len,
            };
            let mut kind = None;
            while cursor.pos < range.end {
                let item = cursor.peek_tagged()?;
                let next = match item.major {
                    0 | 1 | 7 => {
                        cursor.number()?;
                        0
                    }
                    3 => {
                        cursor.text()?;
                        1
                    }
                    _ => bail!("field array has unsupported item"),
                };
                if let Some(previous) = kind {
                    ensure!(previous == next, "field array mixes strings and numbers");
                }
                kind = Some(next);
            }
            cursor.finish(container)?;
            match kind {
                Some(0) | None => Ok(DescriptorValue::Vector(range)),
                Some(1) => Ok(DescriptorValue::StringList(range)),
                _ => unreachable!(),
            }
        }
        _ => bail!("unsupported field value"),
    }
}

#[derive(Default)]
struct MetadataAccounting {
    retained_bytes: usize,
}

fn reserve_growth<T>(
    items: &mut Vec<T>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
    accounting: &mut MetadataAccounting,
) -> Result<()> {
    if items.len() == items.capacity() {
        let next = items
            .capacity()
            .max(1)
            .checked_mul(2)
            .ok_or_else(|| anyhow!("descriptor capacity overflow"))?;
        let old = items
            .capacity()
            .checked_mul(mem::size_of::<T>())
            .ok_or_else(|| anyhow!("descriptor cost overflow"))?;
        let new = next
            .checked_mul(mem::size_of::<T>())
            .ok_or_else(|| anyhow!("descriptor cost overflow"))?;
        ensure!(
            accounting.retained_bytes >= old,
            "metadata accounting lost vector capacity"
        );
        // Vec can retain the old allocation while it acquires the new one.
        // Reserve the complete aggregate peak before that allocation starts.
        reserve(
            accounting
                .retained_bytes
                .checked_add(new)
                .ok_or_else(|| anyhow!("metadata peak overflow"))?,
        )?;
        items.try_reserve_exact(next - items.len())?;
        accounting.retained_bytes = accounting
            .retained_bytes
            .checked_sub(old)
            .ok_or_else(|| anyhow!("metadata accounting underflow"))?
            .checked_add(new)
            .ok_or_else(|| anyhow!("metadata accounting overflow"))?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Head {
    major: u8,
    ai: u8,
    argument: Option<u64>,
}

#[derive(Clone, Copy)]
struct Container {
    start: usize,
    end: usize,
    after_end: usize,
    len: usize,
}

#[derive(Clone, Copy)]
struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    fn byte(&mut self) -> Result<u8> {
        let byte = *self
            .bytes
            .get(self.pos)
            .ok_or_else(|| anyhow!("truncated CBOR"))?;
        self.pos += 1;
        Ok(byte)
    }
    fn peek_tagged(&mut self) -> Result<Head> {
        let pos = self.pos;
        let head = self.tagged()?;
        self.pos = pos;
        Ok(head)
    }
    fn head(&mut self) -> Result<Head> {
        let initial = self.byte()?;
        let major = initial >> 5;
        let ai = initial & 0x1f;
        let argument = match ai {
            0..=23 => Some(ai as u64),
            24 => Some(self.byte()? as u64),
            25 => Some(u16::from_be_bytes([self.byte()?, self.byte()?]) as u64),
            26 => Some(
                u32::from_be_bytes([self.byte()?, self.byte()?, self.byte()?, self.byte()?]) as u64,
            ),
            27 => Some(u64::from_be_bytes([
                self.byte()?,
                self.byte()?,
                self.byte()?,
                self.byte()?,
                self.byte()?,
                self.byte()?,
                self.byte()?,
                self.byte()?,
            ])),
            31 => None,
            _ => bail!("reserved CBOR additional information"),
        };
        Ok(Head {
            major,
            ai,
            argument,
        })
    }
    fn tagged(&mut self) -> Result<Head> {
        loop {
            let head = self.head()?;
            if head.major != 6 {
                return Ok(head);
            }
        }
    }
    fn text(&mut self) -> Result<ByteRange> {
        let head = self.tagged()?;
        ensure!(head.major == 3, "expected CBOR text");
        let len = usize::try_from(
            head.argument
                .ok_or_else(|| NotHandled("fragmented CBOR text"))?,
        )
        .map_err(|_| anyhow!("CBOR text length exceeds usize"))?;
        let start = self.pos;
        let end = start
            .checked_add(len)
            .ok_or_else(|| anyhow!("CBOR text length overflow"))?;
        let text = self
            .bytes
            .get(start..end)
            .ok_or_else(|| anyhow!("truncated CBOR text"))?;
        validate_utf8(text)?;
        self.pos = end;
        Ok(ByteRange { start, end })
    }
    fn trusted_text(&mut self) -> Result<ByteRange> {
        let head = self.tagged()?;
        ensure!(head.major == 3, "expected CBOR text");
        let len = usize::try_from(
            head.argument
                .ok_or_else(|| NotHandled("fragmented CBOR text"))?,
        )
        .map_err(|_| anyhow!("CBOR text length exceeds usize"))?;
        let start = self.pos;
        let end = start
            .checked_add(len)
            .ok_or_else(|| anyhow!("CBOR text length overflow"))?;
        ensure!(end <= self.bytes.len(), "truncated CBOR text");
        self.pos = end;
        Ok(ByteRange { start, end })
    }
    fn unsigned(&mut self) -> Result<u64> {
        let head = self.tagged()?;
        ensure!(
            head.major == 0 && head.argument.is_some(),
            "expected unsigned integer"
        );
        Ok(head.argument.unwrap())
    }
    fn option_unsigned(&mut self) -> Result<Option<u64>> {
        let head = self.peek_tagged()?;
        if head.major == 7 && head.ai == 22 {
            self.tagged()?;
            Ok(None)
        } else {
            self.unsigned().map(Some)
        }
    }
    fn number(&mut self) -> Result<f64> {
        let head = self.tagged()?;
        match (head.major, head.ai, head.argument) {
            (0, _, Some(value)) => Ok(value as f64),
            // ciborium supplies negative integers through serde's i64 path;
            // values below i64::MIN are not accepted by WireFieldValue.
            (1, _, Some(value)) if value <= i64::MAX as u64 => Ok((-1i64 - value as i64) as f64),
            (7, 25, Some(bits)) => Ok(f16_to_f64(bits as u16)),
            (7, 26, Some(bits)) => Ok(f32::from_bits(bits as u32) as f64),
            (7, 27, Some(bits)) => Ok(f64::from_bits(bits)),
            _ => bail!("expected CBOR number"),
        }
    }
    fn number_f32(&mut self) -> Result<f32> {
        let head = self.tagged()?;
        match (head.major, head.ai, head.argument) {
            // Match serde's integer-to-f32 conversion directly. Going through
            // f64 first can round twice for a valid large integer component.
            (0, _, Some(value)) => Ok(value as f32),
            (1, _, Some(value)) if value <= i64::MAX as u64 => Ok((-1i64 - value as i64) as f32),
            (7, 25, Some(bits)) => Ok(f16_to_f64(bits as u16) as f32),
            (7, 26, Some(bits)) => Ok(f32::from_bits(bits as u32)),
            (7, 27, Some(bits)) => Ok(f64::from_bits(bits) as f32),
            _ => bail!("expected CBOR number"),
        }
    }
    fn array(&mut self) -> Result<Container> {
        let head = self.tagged()?;
        ensure!(head.major == 4, "expected CBOR array");
        let start = self.pos;
        match head.argument {
            Some(count) => {
                let len = usize::try_from(count)
                    .map_err(|_| anyhow!("CBOR array length exceeds usize"))?;
                for _ in 0..count {
                    self.skip()?;
                }
                Ok(Container {
                    start,
                    end: self.pos,
                    after_end: self.pos,
                    len,
                })
            }
            None => {
                let mut len = 0usize;
                while self.bytes.get(self.pos) != Some(&0xff) {
                    self.skip()?;
                    len = len
                        .checked_add(1)
                        .ok_or_else(|| anyhow!("CBOR array length overflow"))?;
                }
                let end = self.pos;
                self.pos += 1;
                Ok(Container {
                    start,
                    end,
                    after_end: self.pos,
                    len,
                })
            }
        }
        .map(|container| {
            self.pos = start;
            container
        })
    }
    fn finish(&mut self, container: Container) -> Result<()> {
        ensure!(
            self.pos == container.end,
            "CBOR container content was not fully consumed"
        );
        self.pos = container.after_end;
        Ok(())
    }
    fn map_each(&mut self, mut visit: impl FnMut(&str, &mut Self) -> Result<()>) -> Result<()> {
        let head = self.tagged()?;
        ensure!(head.major == 5, "expected CBOR map");
        let mut remaining = head.argument;
        while remaining.map_or(self.bytes.get(self.pos) != Some(&0xff), |count| count > 0) {
            let key = self.text()?;
            let key = key.as_str(self.bytes);
            visit(key, self)?;
            remaining = remaining.map(|count| count - 1);
        }
        if head.argument.is_none() {
            self.pos += 1;
        }
        Ok(())
    }
    fn map_find<T>(
        &mut self,
        wanted: &str,
        mut read: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Option<T>> {
        let mut found = None;
        self.map_each(|key, cursor| {
            if key == wanted {
                ensure!(found.is_none(), "duplicate {wanted}");
                found = Some(read(cursor)?);
            } else {
                cursor.skip()?;
            }
            Ok(())
        })?;
        Ok(found)
    }
    fn skip(&mut self) -> Result<()> {
        let head = self.head()?;
        match head.major {
            0 | 1 | 7 => Ok(()),
            2 | 3 => match head.argument {
                Some(len) => {
                    let len =
                        usize::try_from(len).map_err(|_| anyhow!("CBOR length exceeds usize"))?;
                    self.pos = self
                        .pos
                        .checked_add(len)
                        .ok_or_else(|| anyhow!("CBOR length overflow"))?;
                    ensure!(self.pos <= self.bytes.len(), "truncated CBOR bytes");
                    Ok(())
                }
                None => {
                    while self.bytes.get(self.pos) != Some(&0xff) {
                        self.skip()?;
                    }
                    self.pos += 1;
                    Ok(())
                }
            },
            4 => match head.argument {
                Some(count) => {
                    for _ in 0..count {
                        self.skip()?;
                    }
                    Ok(())
                }
                None => {
                    while self.bytes.get(self.pos) != Some(&0xff) {
                        self.skip()?;
                    }
                    self.pos += 1;
                    Ok(())
                }
            },
            5 => match head.argument {
                Some(count) => {
                    for _ in 0..count {
                        self.skip()?;
                        self.skip()?;
                    }
                    Ok(())
                }
                None => {
                    while self.bytes.get(self.pos) != Some(&0xff) {
                        self.skip()?;
                        self.skip()?;
                    }
                    self.pos += 1;
                    Ok(())
                }
            },
            6 => self.skip(),
            _ => bail!("invalid CBOR major type"),
        }
    }
}

fn f16_to_f64(bits: u16) -> f64 {
    let sign = if bits & 0x8000 == 0 { 1.0 } else { -1.0 };
    let exponent = ((bits >> 10) & 0x1f) as i32;
    let fraction = (bits & 0x03ff) as f64;
    match exponent {
        0 => sign * fraction * 2f64.powi(-24),
        31 if fraction == 0.0 => sign * f64::INFINITY,
        31 => f64::NAN,
        _ => sign * (1.0 + fraction / 1024.0) * 2f64.powi(exponent - 15),
    }
}

// Append inside `wal::borrowed_replace_scanner` after registering the module.
// These are unit tests; they deliberately exercise the scanner rather than a
// storage adapter.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::{bail, Result};

    use crate::{
        log_entry::RaftLogEntry,
        types::{FieldValue, ReplaceDocItem, ReplaceDocsRequest},
        wal::WalRecord,
    };

    use super::{BorrowedReplaceScanner, BorrowedReplaceValue};

    fn cbor(record: &WalRecord) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(record, &mut bytes).unwrap();
        bytes
    }

    fn record() -> WalRecord {
        WalRecord {
            version: 1,
            entry: RaftLogEntry::ReplaceDocs {
                collection_id: "docs".into(),
                req: ReplaceDocsRequest {
                    docs: vec![
                        ReplaceDocItem {
                            external_id: "first".into(),
                            version: Some(7),
                            fields: BTreeMap::from([
                                ("empty".into(), FieldValue::Vector(vec![])),
                                (
                                    "list".into(),
                                    FieldValue::StringList(vec!["one".into(), "雪".into()]),
                                ),
                                ("number".into(), FieldValue::Number(-2.5)),
                                ("text".into(), FieldValue::String("borrowed-value".into())),
                                (
                                    "vector".into(),
                                    FieldValue::Vector(vec![1.5, -0.0, f32::INFINITY]),
                                ),
                            ]),
                        },
                        ReplaceDocItem {
                            external_id: "second".into(),
                            version: None,
                            fields: BTreeMap::from([(
                                "text".into(),
                                FieldValue::String("later".into()),
                            )]),
                        },
                    ],
                },
            },
        }
    }

    #[test]
    fn borrows_every_replace_value_and_preserves_document_and_btreemap_order() {
        let bytes = cbor(&record());
        let owned = WalRecord::decode(&bytes).unwrap();
        let RaftLogEntry::ReplaceDocs { collection_id, req } = owned.entry else {
            panic!("fixture must decode as ReplaceDocs")
        };
        let source_start = bytes.as_ptr() as usize;
        let source_end = source_start + bytes.len();
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        assert_eq!(scanner.collection_id(), "docs");
        assert_eq!(scanner.collection_id(), collection_id);
        assert_eq!(scanner.consumed_bytes(), bytes.len());
        let docs: Vec<_> = scanner.docs().collect();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].external_id(), "first");
        assert_eq!(docs[0].version(), Some(7));
        assert_eq!(docs[0].external_id(), req.docs[0].external_id);
        assert_eq!(docs[0].version(), req.docs[0].version);
        assert_eq!(docs[1].external_id(), "second");
        assert_eq!(docs[1].version(), None);

        let fields: Vec<_> = docs[0].fields().collect();
        assert_eq!(
            fields.iter().map(|field| field.name()).collect::<Vec<_>>(),
            ["empty", "list", "number", "text", "vector"]
        );
        match fields[3].value() {
            BorrowedReplaceValue::String(value) => {
                assert_eq!(value, "borrowed-value");
                assert!(
                    matches!(req.docs[0].fields.get("text"), Some(FieldValue::String(owned)) if owned == value)
                );
                assert!(
                    (value.as_ptr() as usize) >= source_start
                        && (value.as_ptr() as usize) < source_end
                );
            }
            _ => panic!("text value changed shape"),
        }
        match fields[0].value() {
            BorrowedReplaceValue::Vector(values) => assert_eq!(
                values.collect::<Result<Vec<_>>>().unwrap(),
                Vec::<f32>::new()
            ),
            _ => panic!("empty array must remain Vector"),
        }
        match fields[1].value() {
            BorrowedReplaceValue::StringList(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(
                    values.clone().collect::<Result<Vec<_>>>().unwrap(),
                    ["one", "雪"]
                );
                assert_eq!(values.collect::<Result<Vec<_>>>().unwrap(), ["one", "雪"]);
            }
            _ => panic!("string list changed shape"),
        }
        match fields[2].value() {
            BorrowedReplaceValue::Number(value) => {
                assert_eq!(value, -2.5);
                assert!(
                    matches!(req.docs[0].fields.get("number"), Some(FieldValue::Number(owned)) if *owned == value)
                );
            }
            _ => panic!("number changed shape"),
        }
        match fields[4].value() {
            BorrowedReplaceValue::Vector(values) => {
                assert_eq!(values.len(), 3);
                let values = values.collect::<Result<Vec<_>>>().unwrap();
                assert_eq!(values[0].to_bits(), 1.5f32.to_bits());
                assert_eq!(values[1].to_bits(), (-0.0f32).to_bits());
                assert!(values[2].is_infinite());
                assert!(
                    matches!(req.docs[0].fields.get("vector"), Some(FieldValue::Vector(owned)) if owned.iter().map(|value| value.to_bits()).eq(values.iter().map(|value| value.to_bits())))
                );
            }
            _ => panic!("vector changed shape"),
        }
    }

    #[test]
    fn validates_the_entire_cbor_record_but_ignores_trailing_bytes_like_from_reader() {
        let mut bytes = cbor(&record());
        bytes.extend_from_slice(&[0xf6, 0xf6]);
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        assert_eq!(scanner.consumed_bytes() + 2, bytes.len());

        let mut invalid = cbor(&record());
        let offset = invalid
            .windows("borrowed-value".len())
            .position(|window| window == b"borrowed-value")
            .unwrap();
        invalid[offset] = 0xff;
        assert!(BorrowedReplaceScanner::scan(&invalid, |_| Ok(())).is_err());
    }

    #[test]
    fn reservation_refusal_happens_before_doc_descriptor_growth() {
        let bytes = cbor(&record());
        let mut requested = 0;
        let error = BorrowedReplaceScanner::scan(&bytes, |bytes| {
            requested += 1;
            bail!("reservation refuses {bytes} descriptor bytes")
        })
        .unwrap_err();
        assert_eq!(requested, 1);
        assert!(error.to_string().contains("reservation refuses"));
    }

    #[test]
    fn large_definite_text_is_a_borrowed_range_and_vector_iteration_is_constant_state() {
        let giant = "x".repeat(65 * 1024);
        let record = WalRecord {
            version: 1,
            entry: RaftLogEntry::ReplaceDocs {
                collection_id: "docs".into(),
                req: ReplaceDocsRequest {
                    docs: vec![ReplaceDocItem {
                        external_id: "id".into(),
                        version: None,
                        fields: BTreeMap::from([("giant".into(), FieldValue::String(giant))]),
                    }],
                },
            },
        };
        let bytes = cbor(&record);
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let field = scanner.docs().next().unwrap().fields().next().unwrap();
        match field.value() {
            BorrowedReplaceValue::String(value) => {
                assert_eq!(value.len(), 65 * 1024);
                let start = bytes.as_ptr() as usize;
                let end = start + bytes.len();
                assert!((value.as_ptr() as usize) >= start && (value.as_ptr() as usize) < end);
            }
            _ => panic!("giant text changed shape"),
        }
    }

    #[test]
    fn accepts_well_formed_more_than_32_docs_for_storage_to_apply_its_bulk_limit() {
        let docs = (0..33)
            .map(|number| ReplaceDocItem {
                external_id: format!("id-{number}"),
                version: None,
                fields: BTreeMap::from([("text".into(), FieldValue::String("x".into()))]),
            })
            .collect();
        let bytes = cbor(&WalRecord {
            version: 1,
            entry: RaftLogEntry::ReplaceDocs {
                collection_id: "docs".into(),
                req: ReplaceDocsRequest { docs },
            },
        });
        assert_eq!(
            BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
                .unwrap()
                .unwrap()
                .docs()
                .count(),
            33
        );
    }

    #[test]
    fn reserve_requests_complete_aggregate_metadata_peak_and_refuses_before_second_field_growth() {
        let bytes = cbor(&record());
        let mut requests = Vec::new();
        let scanner = BorrowedReplaceScanner::scan(&bytes, |need| {
            requests.push(need);
            Ok(())
        })
        .unwrap()
        .unwrap();
        assert!(requests.iter().copied().max().unwrap() >= scanner.retained_metadata_bytes());
        assert!(
            requests.windows(2).any(|pair| pair[1] > pair[0]),
            "field vectors must include earlier retained vectors"
        );

        let mut calls = 0;
        let error = BorrowedReplaceScanner::scan(&bytes, |_| {
            calls += 1;
            if calls == 3 {
                bail!("refuse before second field vector growth");
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(calls, 3);
        assert!(error.to_string().contains("second field vector"));
    }

    #[test]
    fn indefinite_containers_finish_before_following_map_members() {
        fn text(out: &mut Vec<u8>, value: &str) {
            out.push(0x60 | value.len() as u8);
            out.extend_from_slice(value.as_bytes());
        }
        let mut bytes = vec![0xbf]; // top-level indefinite map
        text(&mut bytes, "version");
        bytes.push(1);
        text(&mut bytes, "entry");
        bytes.push(0xbf);
        text(&mut bytes, "ReplaceDocs");
        bytes.push(0xbf);
        text(&mut bytes, "collection_id");
        text(&mut bytes, "docs");
        text(&mut bytes, "req");
        bytes.push(0xbf);
        text(&mut bytes, "docs");
        bytes.push(0x9f);
        bytes.push(0xbf);
        text(&mut bytes, "external_id");
        text(&mut bytes, "one");
        text(&mut bytes, "fields");
        bytes.push(0xbf);
        text(&mut bytes, "set");
        bytes.push(0x9f);
        text(&mut bytes, "a");
        text(&mut bytes, "b");
        bytes.push(0xff);
        bytes.push(0xff);
        bytes.push(0xff);
        bytes.push(0xff); // fields, doc, docs
        text(&mut bytes, "ignored");
        bytes.push(0xf6);
        bytes.push(0xff); // req
        bytes.push(0xff);
        bytes.push(0xff);
        bytes.push(0xff); // body, entry, top
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let field = scanner.docs().next().unwrap().fields().next().unwrap();
        match field.value() {
            BorrowedReplaceValue::StringList(values) => {
                assert_eq!(values.collect::<Result<Vec<_>>>().unwrap(), ["a", "b"])
            }
            _ => panic!("indefinite string list changed shape"),
        }
        assert_eq!(scanner.consumed_bytes(), bytes.len());
    }

    #[test]
    fn rejects_multi_variant_entry_and_reuses_validated_string_list_without_a_second_utf8_scan() {
        use ciborium::value::Value;
        let text = |value: &str| Value::Text(value.into());
        let doc = Value::Map(vec![
            (text("external_id"), text("id")),
            (text("fields"), Value::Map(vec![(text("text"), text("x"))])),
        ]);
        let replace = Value::Map(vec![
            (text("collection_id"), text("docs")),
            (
                text("req"),
                Value::Map(vec![(text("docs"), Value::Array(vec![doc]))]),
            ),
        ]);
        let multi_variant_record = Value::Map(vec![
            (text("version"), Value::Integer(1.into())),
            (
                text("entry"),
                Value::Map(vec![
                    (text("ReplaceDocs"), replace),
                    (text("Index"), Value::Null),
                ]),
            ),
        ]);
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&multi_variant_record, &mut bytes).unwrap();
        assert!(BorrowedReplaceScanner::scan(&bytes, |_| Ok(())).is_err());

        let bytes = cbor(&record());
        super::reset_utf8_validation_bytes_for_test();
        let scanner = BorrowedReplaceScanner::scan(&bytes, |_| Ok(()))
            .unwrap()
            .unwrap();
        let validated = super::utf8_validation_bytes_for_test();
        let list = scanner
            .docs()
            .next()
            .unwrap()
            .fields()
            .find(|field| field.name() == "list")
            .unwrap();
        for _ in 0..3 {
            match list.value() {
                BorrowedReplaceValue::StringList(values) => {
                    assert_eq!(values.collect::<Result<Vec<_>>>().unwrap(), ["one", "雪"]);
                }
                _ => unreachable!(),
            }
        }
        assert_eq!(super::utf8_validation_bytes_for_test(), validated);
    }

    #[test]
    fn bounded_preflight_rejects_break_bad_chunks_and_the_257th_container() {
        assert!(BorrowedReplaceScanner::scan(&[0xff], |_| Ok(())).is_err());
        assert!(BorrowedReplaceScanner::scan(&[0x7f, 0x41, b'x', 0xff], |_| Ok(())).is_err());
        let mut nested = vec![0x81; 257];
        nested.push(0xf6);
        assert!(BorrowedReplaceScanner::scan(&nested, |_| Ok(())).is_err());
    }
}
