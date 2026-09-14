//! Scalar checkpoint rows can lend values from immutable temporary segments.
//!
//! This projection never rehydrates a staged field.  Its reader sources keep
//! only a map from staged row number to output row number.  A dictionary pass
//! holds one term and one decoded posting per reader; owned checkpoint strings
//! are borrowed through a small `BTreeMap<&str, Vec<u32>>`.

use super::*;
use crate::segment::stream::{
    write_keyword_projection, write_number_projection, write_set_projection,
    KeywordStreamProjection, NumberStreamProjection, ScalarProjectionScratch, SetStreamProjection,
};
use crate::segment::{ScalarPayloadKind, SegmentReader};
use crate::types::FieldType;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::sync::Arc;

pub(crate) fn write_checkpoint_rows(
    path: &std::path::Path,
    seq: u64,
    field_type: FieldType,
    rows: &[(
        String,
        Option<crate::change_journal::SharedValue<CheckpointValue>>,
    )],
) -> Result<()> {
    let projection = RowsProjection::new(field_type, rows)?;
    // The stream writer's temporary posting is the only newly owned field
    // payload.  Do not invent a checkpoint value limit here.
    let scratch = ScalarProjectionScratch::new(usize::MAX).raw_scalar_dictionary();
    match field_type {
        FieldType::Keyword => write_keyword_projection(path, seq, &projection, scratch),
        FieldType::Number => write_number_projection(path, seq, &projection, scratch),
        FieldType::Set => write_set_projection(path, seq, &projection, scratch),
        _ => bail!("scalar checkpoint projection needs keyword, number, or set field"),
    }
}

/// One immutable reader, deduplicated by its Arc allocation.  `selected` is
/// deliberately indexed by source row: dictionary postings remain on disk and
/// only selected dirty rows can enter the new local posting.
struct ReaderSource {
    reader: Arc<SegmentReader>,
    selected: HashMap<u32, u32>,
}

struct RowsProjection<'a> {
    field_type: FieldType,
    rows: &'a [(
        String,
        Option<crate::change_journal::SharedValue<CheckpointValue>>,
    )],
    readers: Vec<ReaderSource>,
    /// This owns no strings.  The bounded tail is already held by the frozen
    /// checkpoint; only its term-to-local-row index is new.
    owned: BTreeMap<&'a str, Vec<u32>>,
    owned_numbers: BTreeMap<u64, Vec<u32>>,
}

impl<'a> RowsProjection<'a> {
    fn new(
        field_type: FieldType,
        rows: &'a [(
            String,
            Option<crate::change_journal::SharedValue<CheckpointValue>>,
        )],
    ) -> Result<Self> {
        let mut readers = Vec::new();
        let mut reader_by_ptr = HashMap::<usize, usize>::new();
        let mut owned = BTreeMap::<&str, Vec<u32>>::new();
        let mut owned_numbers = BTreeMap::<u64, Vec<u32>>::new();
        for (local, (_, value)) in rows.iter().enumerate() {
            let local =
                u32::try_from(local).map_err(|_| anyhow!("scalar delta rows exceed u32"))?;
            match (field_type, value.as_deref()) {
                (_, None) => {}
                (FieldType::Keyword, Some(CheckpointValue::Keyword(value))) => {
                    owned.entry(value).or_default().push(local);
                }
                (FieldType::Set, Some(CheckpointValue::Set(values))) => {
                    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
                        bail!("owned Set delta values must be sorted and unique");
                    }
                    for value in values {
                        owned.entry(value).or_default().push(local);
                    }
                }
                (FieldType::Number, Some(CheckpointValue::Number(value))) => {
                    owned_numbers
                        .entry(Self::number_key(*value)?)
                        .or_default()
                        .push(local);
                }
                (_, Some(CheckpointValue::StagedScalar { reader, row })) => {
                    if *row >= reader.n_docs() {
                        bail!("staged scalar row is outside its segment");
                    }
                    let expected = match field_type {
                        FieldType::Keyword => ScalarPayloadKind::Keyword,
                        FieldType::Number => ScalarPayloadKind::Number,
                        FieldType::Set => ScalarPayloadKind::Set,
                        _ => bail!("scalar checkpoint projection field mismatch"),
                    };
                    if reader.scalar_payload_kind() != Some(expected) {
                        bail!("staged scalar payload kind does not match field");
                    }
                    let key = Arc::as_ptr(reader) as usize;
                    let source = match reader_by_ptr.get(&key) {
                        Some(source) => *source,
                        None => {
                            let source = readers.len();
                            readers.push(ReaderSource {
                                reader: reader.clone(),
                                selected: HashMap::new(),
                            });
                            reader_by_ptr.insert(key, source);
                            source
                        }
                    };
                    if readers[source].selected.insert(*row, local).is_some() {
                        bail!("staged scalar row selected more than once");
                    }
                }
                (FieldType::Keyword, _) => bail!("keyword delta value mismatch"),
                (FieldType::Number, _) => bail!("number delta value mismatch"),
                (FieldType::Set, _) => bail!("set delta value mismatch"),
                _ => bail!("scalar checkpoint projection field mismatch"),
            }
        }
        Ok(Self {
            field_type,
            rows,
            readers,
            owned,
            owned_numbers,
        })
    }

    fn staged_keyword<'r>(
        &self,
        row: u32,
        reader: &'r SegmentReader,
    ) -> Result<Option<Cow<'r, str>>> {
        Ok(reader.keyword_at_cow(row))
    }

    fn staged_set(
        &self,
        row: u32,
        reader: &SegmentReader,
        emit: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<bool> {
        let (present, count) = reader
            .set_row_member_count(row)
            .ok_or_else(|| anyhow!("staged Set row is corrupt"))?;
        if !present {
            return Ok(false);
        }
        for ordinal in 0..count {
            let value = reader
                .set_member_at_cow(row, ordinal)
                .ok_or_else(|| anyhow!("staged Set member is corrupt"))?;
            emit(&value)?;
        }
        Ok(true)
    }

    fn staged_posting(&self, source: &ReaderSource, term: &str) -> Result<Vec<u32>> {
        let posting = match self.field_type {
            FieldType::Keyword => source.reader.keyword_postings(term),
            FieldType::Set => source.reader.set_postings(term),
            _ => None,
        };
        // A term from one reader is normally absent from every other reader.
        // `None` is therefore an empty source contribution here.
        let Some(posting) = posting else {
            return Ok(Vec::new());
        };
        // The bitmap and `out` are scoped to this one term and one reader.
        let mut out = Vec::new();
        for staged_row in posting.iter() {
            if let Some(local) = source.selected.get(&staged_row) {
                out.push(*local);
            }
        }
        Ok(out)
    }

    fn posting_for(&self, term: &str) -> Result<Vec<u32>> {
        let mut out = self.owned.get(term).cloned().unwrap_or_default();
        for source in &self.readers {
            out.extend(self.staged_posting(source, term)?);
        }
        out.sort_unstable();
        if out.windows(2).any(|pair| pair[0] == pair[1]) {
            bail!("scalar checkpoint term has duplicate local row");
        }
        Ok(out)
    }

    fn number_key(value: f64) -> Result<u64> {
        Ok(SortableF64::new(value)?.bits())
    }

    fn number_posting_for(&self, key: u64) -> Result<Vec<u32>> {
        let mut out = self.owned_numbers.get(&key).cloned().unwrap_or_default();
        for source in &self.readers {
            let Some(posting) = source.reader.number_value_postings(key) else {
                continue;
            };
            for staged_row in posting.iter() {
                if let Some(local) = source.selected.get(&staged_row) {
                    out.push(*local);
                }
            }
        }
        out.sort_unstable();
        if out.windows(2).any(|pair| pair[0] == pair[1]) {
            bail!("scalar checkpoint number has duplicate local row");
        }
        Ok(out)
    }

    fn number_keys_impl(&self, emit: &mut dyn FnMut(u64) -> Result<()>) -> Result<()> {
        let mut cursors: Vec<Box<dyn Iterator<Item = Result<u64>> + '_>> = Vec::new();
        cursors.push(Box::new(self.owned_numbers.keys().copied().map(Ok)));
        for source in &self.readers {
            cursors.push(Box::new(NumberReaderKeys::new(&source.reader)?));
        }
        let mut heap = BinaryHeap::new();
        for (source, cursor) in cursors.iter_mut().enumerate() {
            if let Some(key) = cursor.next().transpose()? {
                heap.push(NumberHead { key, source });
            }
        }
        while let Some(first) = heap.pop() {
            let key = first.key;
            let mut advance = vec![first.source];
            while heap.peek().is_some_and(|head: &NumberHead| head.key == key) {
                advance.push(heap.pop().expect("heap head checked").source);
            }
            for source in advance {
                if let Some(next) = cursors[source].next().transpose()? {
                    heap.push(NumberHead { key: next, source });
                }
            }
            if !self.number_posting_for(key)?.is_empty() {
                emit(key)?;
            }
        }
        Ok(())
    }

    fn number_posting_impl(
        &self,
        key: u64,
        emit: &mut dyn FnMut(u32) -> Result<()>,
    ) -> Result<bool> {
        let posting = self.number_posting_for(key)?;
        for row in &posting {
            emit(*row)?;
        }
        Ok(!posting.is_empty())
    }

    fn emit_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        let mut cursors: Vec<Box<dyn Iterator<Item = Result<Cow<'_, str>>> + '_>> = Vec::new();
        cursors.push(Box::new(
            self.owned.keys().map(|term| Ok(Cow::Borrowed(*term))),
        ));
        for source in &self.readers {
            cursors.push(Box::new(ReaderTerms::new(&source.reader)?));
        }
        let mut heap = BinaryHeap::new();
        for (source, cursor) in cursors.iter_mut().enumerate() {
            if let Some(term) = cursor.next().transpose()? {
                heap.push(TermHead { term, source });
            }
        }
        while let Some(first) = heap.pop() {
            let term = first.term;
            let mut advance = vec![first.source];
            while heap.peek().is_some_and(|head: &TermHead| head.term == term) {
                advance.push(heap.pop().expect("heap head checked").source);
            }
            for source in advance {
                if let Some(next) = cursors[source].next().transpose()? {
                    heap.push(TermHead { term: next, source });
                }
            }
            let posting = self.posting_for(&term)?;
            if !posting.is_empty() {
                emit(&term)?;
            }
        }
        Ok(())
    }
}

struct ReaderTerms<'a> {
    reader: &'a SegmentReader,
    next: u32,
    count: u32,
}
impl<'a> ReaderTerms<'a> {
    fn new(reader: &'a SegmentReader) -> Result<Self> {
        Ok(Self {
            reader,
            next: 0,
            count: reader
                .keyword_ordinal_count()
                .ok_or_else(|| anyhow!("staged scalar dictionary is missing"))?,
        })
    }
}
impl<'a> Iterator for ReaderTerms<'a> {
    type Item = Result<Cow<'a, str>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.count {
            return None;
        }
        let ordinal = self.next;
        self.next += 1;
        Some(
            self.reader
                .keyword_term_at_ordinal_cow(ordinal)
                .ok_or_else(|| anyhow!("staged scalar dictionary entry is corrupt")),
        )
    }
}
struct TermHead<'a> {
    term: Cow<'a, str>,
    source: usize,
}
impl PartialEq for TermHead<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term && self.source == other.source
    }
}
impl Eq for TermHead<'_> {}
impl PartialOrd for TermHead<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TermHead<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .term
            .cmp(&self.term)
            .then_with(|| other.source.cmp(&self.source))
    }
}

struct NumberReaderKeys<'a> {
    reader: &'a SegmentReader,
    next: u32,
    count: u64,
}
impl<'a> NumberReaderKeys<'a> {
    fn new(reader: &'a SegmentReader) -> Result<Self> {
        Ok(Self {
            reader,
            next: 0,
            count: reader.number_distinct_count(),
        })
    }
}
impl Iterator for NumberReaderKeys<'_> {
    type Item = Result<u64>;
    fn next(&mut self) -> Option<Self::Item> {
        if u64::from(self.next) == self.count {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(
            self.reader
                .number_sorted_bits_at(index)
                .ok_or_else(|| anyhow!("staged scalar number key is corrupt")),
        )
    }
}
struct NumberHead {
    key: u64,
    source: usize,
}
impl PartialEq for NumberHead {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.source == other.source
    }
}
impl Eq for NumberHead {}
impl PartialOrd for NumberHead {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NumberHead {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .key
            .cmp(&self.key)
            .then_with(|| other.source.cmp(&self.source))
    }
}

impl KeywordStreamProjection for RowsProjection<'_> {
    fn n_docs(&self) -> u32 {
        self.rows.len() as u32
    }
    fn keyword_row(
        &self,
        row: u32,
        emit: &mut dyn FnMut(Option<&str>) -> Result<()>,
    ) -> Result<()> {
        match self.rows[row as usize].1.as_deref() {
            Some(CheckpointValue::Keyword(value)) => emit(Some(value)),
            Some(CheckpointValue::StagedScalar { reader, row }) => {
                let value = self.staged_keyword(*row, reader)?;
                emit(value.as_deref())
            }
            None => emit(None),
            _ => bail!("keyword delta value mismatch"),
        }
    }
    fn keyword_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        self.emit_terms(emit)
    }
    fn keyword_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        let posting = self.posting_for(term)?;
        for row in &posting {
            emit(*row)?;
        }
        Ok(!posting.is_empty())
    }
}
impl SetStreamProjection for RowsProjection<'_> {
    fn n_docs(&self) -> u32 {
        self.rows.len() as u32
    }
    fn set_row(&self, row: u32, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<bool> {
        match self.rows[row as usize].1.as_deref() {
            Some(CheckpointValue::Set(values)) => {
                for value in values {
                    emit(value)?;
                }
                Ok(true)
            }
            Some(CheckpointValue::StagedScalar { reader, row }) => {
                self.staged_set(*row, reader, emit)
            }
            None => Ok(false),
            _ => bail!("set delta value mismatch"),
        }
    }
    fn set_terms(&self, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
        self.emit_terms(emit)
    }
    fn set_posting(&self, term: &str, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        let posting = self.posting_for(term)?;
        for row in &posting {
            emit(*row)?;
        }
        Ok(!posting.is_empty())
    }
}
impl NumberStreamProjection for RowsProjection<'_> {
    fn n_docs(&self) -> u32 {
        self.rows.len() as u32
    }
    fn number_row(&self, row: u32, emit: &mut dyn FnMut(Option<f64>) -> Result<()>) -> Result<()> {
        match self.rows[row as usize].1.as_deref() {
            Some(CheckpointValue::Number(value)) => emit(Some(*value)),
            Some(CheckpointValue::StagedScalar { reader, row }) => emit(reader.number_at(*row)),
            None => emit(None),
            _ => bail!("number delta value mismatch"),
        }
    }
    fn number_keys(&self, emit: &mut dyn FnMut(u64) -> Result<()>) -> Result<()> {
        self.number_keys_impl(emit)
    }
    fn number_posting(&self, key: u64, emit: &mut dyn FnMut(u32) -> Result<()>) -> Result<bool> {
        self.number_posting_impl(key, emit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_journal::SharedValue;
    use crate::segment::{write_keyword_segment, write_number_segment, write_set_segment};

    fn owned(value: CheckpointValue) -> SharedValue<CheckpointValue> {
        SharedValue::new(Arc::new(value), None)
    }
    fn staged(reader: Arc<SegmentReader>, row: u32) -> SharedValue<CheckpointValue> {
        owned(CheckpointValue::StagedScalar { reader, row })
    }
    fn open(path: &std::path::Path) -> Arc<SegmentReader> {
        Arc::new(SegmentReader::open(path).unwrap())
    }

    #[test]
    fn keyword_mixes_staged_replacement_and_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.lseg");
        write_keyword_segment(
            &source,
            1,
            &[Some("old"), Some("keep"), Some("gone")],
            &[
                ("gone".into(), [2].into_iter().collect()),
                ("keep".into(), [1].into_iter().collect()),
                ("old".into(), [0].into_iter().collect()),
            ]
            .into_iter()
            .collect(),
        )
        .unwrap();
        let reader = open(&source);
        let out = dir.path().join("out.lseg");
        write_checkpoint_rows(
            &out,
            2,
            FieldType::Keyword,
            &[
                ("a".into(), Some(staged(reader.clone(), 0))),
                (
                    "b".into(),
                    Some(owned(CheckpointValue::Keyword("new".into()))),
                ),
                ("c".into(), None),
                ("d".into(), Some(staged(reader, 1))),
            ],
        )
        .unwrap();
        let got = open(&out);
        assert_eq!(got.keyword_at(0).as_deref(), Some("old"));
        assert_eq!(got.keyword_at(1).as_deref(), Some("new"));
        assert_eq!(got.keyword_at(2), None);
        assert_eq!(got.keyword_at(3).as_deref(), Some("keep"));
        assert_eq!(got.keyword_postings("gone"), None);
    }

    #[test]
    fn set_keeps_empty_and_removes_duplicates_and_deleted_rows() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.lseg");
        let values = vec![
            vec!["a".into(), "b".into()],
            vec![],
            vec!["a".into(), "z".into()],
        ];
        let postings = [
            ("a".into(), [0, 2].into_iter().collect()),
            ("b".into(), [0].into_iter().collect()),
            ("z".into(), [2].into_iter().collect()),
        ]
        .into_iter()
        .collect();
        write_set_segment(
            &source,
            1,
            &[
                Some(values[0].as_slice()),
                Some(values[1].as_slice()),
                Some(values[2].as_slice()),
            ],
            &postings,
        )
        .unwrap();
        let out = dir.path().join("out.lseg");
        write_checkpoint_rows(
            &out,
            2,
            FieldType::Set,
            &[
                ("a".into(), Some(staged(open(&source), 0))),
                ("b".into(), Some(staged(open(&source), 1))),
                ("c".into(), None),
                ("d".into(), Some(staged(open(&source), 2))),
            ],
        )
        .unwrap();
        let got = open(&out);
        assert_eq!(got.set_at(0), Some(vec!["a".into(), "b".into()]));
        assert_eq!(got.set_at(1), Some(vec![]));
        assert_eq!(got.set_at(2), None);
        assert_eq!(got.set_at(3), Some(vec!["a".into(), "z".into()]));
        assert_eq!(
            got.set_postings("a").unwrap().iter().collect::<Vec<_>>(),
            vec![0, 3]
        );
    }

    #[test]
    fn number_sparse_keys_preserve_exact_numeric_order() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.lseg");
        write_number_segment(&source, 1, &[Some(-3.0), None, Some(2.5)]).unwrap();
        let out = dir.path().join("out.lseg");
        write_checkpoint_rows(
            &out,
            2,
            FieldType::Number,
            &[
                ("a".into(), Some(staged(open(&source), 2))),
                ("b".into(), None),
                ("c".into(), Some(owned(CheckpointValue::Number(-4.0)))),
                ("d".into(), Some(staged(open(&source), 0))),
            ],
        )
        .unwrap();
        let got = open(&out);
        assert_eq!(got.number_at(0), Some(2.5));
        assert_eq!(got.number_at(1), None);
        assert_eq!(got.number_at(2), Some(-4.0));
        assert_eq!(got.number_at(3), Some(-3.0));
        assert_eq!(
            got.number_sorted_bits_at(0)
                .map(SortableF64::from_bits)
                .map(SortableF64::to_f64),
            Some(-4.0)
        );
    }

    #[test]
    fn large_staged_keyword_is_read_one_dictionary_term_at_a_time() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.lseg");
        let a = format!("a{}", "x".repeat(64 * 1024));
        let b = format!("b{}", "y".repeat(64 * 1024));
        let c = format!("c{}", "z".repeat(64 * 1024));
        let unused = format!("u{}", "q".repeat(64 * 1024));
        let postings = [
            (a.clone(), [0].into_iter().collect()),
            (b.clone(), [2].into_iter().collect()),
            (c.clone(), [3].into_iter().collect()),
            (unused.clone(), [1].into_iter().collect()),
        ]
        .into_iter()
        .collect();
        write_keyword_segment(
            &source,
            1,
            &[
                Some(a.as_str()),
                Some(unused.as_str()),
                Some(b.as_str()),
                Some(c.as_str()),
            ],
            &postings,
        )
        .unwrap();
        let out = dir.path().join("out.lseg");
        write_checkpoint_rows(
            &out,
            2,
            FieldType::Keyword,
            &[
                ("a".into(), Some(staged(open(&source), 0))),
                ("deleted".into(), None),
                ("b".into(), Some(staged(open(&source), 2))),
                ("c".into(), Some(staged(open(&source), 3))),
            ],
        )
        .unwrap();
        let got = open(&out);
        assert_eq!(got.keyword_at(0).as_deref(), Some(a.as_str()));
        assert_eq!(got.keyword_at(1), None);
        assert_eq!(got.keyword_at(2).as_deref(), Some(b.as_str()));
        assert_eq!(got.keyword_at(3).as_deref(), Some(c.as_str()));
        assert_eq!(
            got.keyword_postings(a.as_str())
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![0]
        );
        assert_eq!(
            got.keyword_postings(b.as_str())
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![2]
        );
        assert_eq!(
            got.keyword_postings(c.as_str())
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec![3]
        );
        assert_eq!(got.keyword_postings(unused.as_str()), None);
    }

    #[test]
    fn wrong_kind_staged_reader_refuses_before_output_exists() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("number.lseg");
        let out = dir.path().join("out.lseg");
        write_number_segment(&source, 1, &[Some(7.0)]).unwrap();
        let error = write_checkpoint_rows(
            &out,
            2,
            FieldType::Keyword,
            &[("a".into(), Some(staged(open(&source), 0)))],
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("staged scalar payload kind does not match field"));
        assert!(!out.exists());
    }

    #[test]
    fn noncanonical_owned_set_refuses_before_output_exists() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.lseg");
        let error = write_checkpoint_rows(
            &out,
            2,
            FieldType::Set,
            &[(
                "a".into(),
                Some(owned(CheckpointValue::Set(vec![
                    "dup".into(),
                    "dup".into(),
                ]))),
            )],
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("owned Set delta values must be sorted and unique"));
        assert!(!out.exists());
    }
}
