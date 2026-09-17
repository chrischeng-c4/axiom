//! Text checkpoint projections keep prepared rows in their immutable mmaps.
//! The dictionary merge owns one current term per input. Posting payloads are
//! decoded one term at a time, with the same live-version selection as queries.

use super::*;
use crate::segment::stream::{write_text_projection, TextStreamView};
use crate::segment::SegmentReader;
use std::borrow::Cow;
#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

type Terms<'a> = Box<dyn Iterator<Item = Result<Cow<'a, str>>> + 'a>;

struct ReaderTerms<'a> {
    reader: &'a SegmentReader,
    next: u32,
    count: u32,
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
                .ok_or_else(|| anyhow!("staged Text dictionary entry is corrupt")),
        )
    }
}
fn reader_terms(reader: &SegmentReader) -> Result<Terms<'_>> {
    let count = reader
        .keyword_ordinal_count()
        .ok_or_else(|| anyhow!("staged Text dictionary is missing"))?;
    Ok(Box::new(ReaderTerms {
        reader,
        next: 0,
        count,
    }))
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
        // `BinaryHeap` is a max heap. Reverse lexical order makes its top the
        // next dictionary term, while the source tie-break makes output stable.
        other
            .term
            .cmp(&self.term)
            .then_with(|| other.source.cmp(&self.source))
    }
}

struct UnionTerms<'a> {
    sources: Vec<Terms<'a>>,
    heads: BinaryHeap<TermHead<'a>>,
    failed: bool,
}

impl<'a> UnionTerms<'a> {
    fn new(mut sources: Vec<Terms<'a>>) -> Result<Self> {
        let mut heads = BinaryHeap::new();
        for (source, terms) in sources.iter_mut().enumerate() {
            if let Some(term) = terms.next().transpose()? {
                heads.push(TermHead { term, source });
            }
        }
        Ok(Self {
            sources,
            heads,
            failed: false,
        })
    }
}
impl<'a> Iterator for UnionTerms<'a> {
    type Item = Result<Cow<'a, str>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let first = self.heads.pop()?;
        let minimum = first.term;
        let mut matches = vec![first.source];
        while self.heads.peek().is_some_and(|head| head.term == minimum) {
            matches.push(self.heads.pop().expect("heap head checked").source);
        }
        for source in matches {
            match self.sources[source].next().transpose() {
                Ok(Some(term)) => self.heads.push(TermHead { term, source }),
                Ok(None) => {}
                Err(error) => {
                    self.failed = true;
                    return Some(Err(error));
                }
            }
        }
        Some(Ok(minimum))
    }
}

fn live_terms(index: &TextIndex) -> Result<Terms<'_>> {
    let mut sources: Vec<Terms<'_>> = Vec::new();
    if let Some(segment) = &index.segment {
        sources.push(TextStreamView::terms(segment.as_ref())?);
    }
    sources.push(Box::new(
        index
            .tokens
            .keys()
            .map(|term| Ok(Cow::Borrowed(term.as_str()))),
    ));
    for row in index.staged_rows.values() {
        sources.push(reader_terms(row.reader())?);
    }
    Ok(Box::new(UnionTerms::new(sources)?))
}

pub(super) fn live_term_count(index: &TextIndex) -> Result<u64> {
    let mut count = 0;
    for term in live_terms(index)? {
        if index.tok_postings(term?.as_ref()).is_some() {
            count += 1;
        }
    }
    Ok(count)
}

struct LiveProjection<'a> {
    index: &'a TextIndex,
    n_docs: u32,
    live: &'a dyn Fn(u32) -> bool,
}
impl TextStreamView for LiveProjection<'_> {
    fn n_docs(&self) -> u32 {
        self.n_docs
    }
    fn text_is_present(&self, id: u32) -> bool {
        (self.live)(id)
            && (self.index.staged_rows.contains_key(&id)
                || self.index.distinct_at(id).is_some()
                || self.index.segment.as_ref().is_some_and(|segment| {
                    !self.index.tombstones.contains(id) && segment.text_is_present(id)
                }))
    }
    fn text_doc_len(&self, id: u32) -> u32 {
        if self.text_is_present(id) {
            self.index.doc_len(id)
        } else {
            0
        }
    }
    fn terms<'a>(&'a self) -> Result<Terms<'a>> {
        live_terms(self.index)
    }
    fn text_postings(&self, term: &str) -> Result<Option<Arc<(Vec<u32>, Vec<u32>)>>> {
        let Some(posting) = self.index.tok_postings(term) else {
            return Ok(None);
        };
        let (mut ids, mut tfs) = match posting {
            TokPostings::Live(posting) => (posting.docids.clone(), posting.tfs.clone()),
            TokPostings::Segment(posting) => {
                Arc::try_unwrap(posting).unwrap_or_else(|posting| (*posting).clone())
            }
            TokPostings::Combined { docids, tfs } => (docids, tfs),
            // Only `tok_postings_at` builds a candidate projection (#4246);
            // `tok_postings` always yields the full active posting.
            TokPostings::Sparse(_) => {
                unreachable!("tok_postings never yields a candidate projection")
            }
        };
        let mut write = 0;
        for read in 0..ids.len() {
            if self.text_is_present(ids[read]) {
                ids[write] = ids[read];
                tfs[write] = tfs[read];
                write += 1;
            }
        }
        ids.truncate(write);
        tfs.truncate(write);
        if ids.is_empty() {
            return Ok(None);
        }
        Ok(Some(Arc::new((ids, tfs))))
    }
}

pub(super) fn write_live(
    path: &std::path::Path,
    seq: u64,
    index: &TextIndex,
    n_docs: u32,
    live: &dyn Fn(u32) -> bool,
) -> Result<()> {
    write_text_projection(
        path,
        seq,
        &LiveProjection {
            index,
            n_docs,
            live,
        },
    )
}

pub(crate) fn write_checkpoint_rows(
    path: &std::path::Path,
    seq: u64,
    rows: &[(
        String,
        Option<crate::change_journal::SharedValue<CheckpointValue>>,
    )],
) -> Result<()> {
    u32::try_from(rows.len()).map_err(|_| anyhow!("Text delta rows exceed u32"))?;
    for (_, value) in rows {
        if !matches!(
            value.as_deref(),
            None | Some(CheckpointValue::Text { .. } | CheckpointValue::StagedText(_))
        ) {
            bail!("text delta value mismatch");
        }
    }
    write_text_projection(
        path,
        seq,
        &RowsProjection {
            rows,
            current: RefCell::new(None),
            #[cfg(test)]
            rows_examined: Cell::new(0),
        },
    )
}
struct RowsProjection<'a> {
    rows: &'a [(
        String,
        Option<crate::change_journal::SharedValue<CheckpointValue>>,
    )],
    current: RefCell<Option<CurrentPosting<'a>>>,
    #[cfg(test)]
    rows_examined: Cell<usize>,
}

struct CurrentPosting<'a> {
    term: Cow<'a, str>,
    posting: Arc<(Vec<u32>, Vec<u32>)>,
}

enum RowTermCursor<'a> {
    Ordinary {
        source: usize,
        row: u32,
        terms: std::collections::btree_map::Iter<'a, String, u32>,
    },
    Staged {
        source: usize,
        row: u32,
        reader: &'a SegmentReader,
        next: u32,
        count: u32,
    },
}

impl<'a> RowTermCursor<'a> {
    fn next(&mut self) -> Result<Option<RowTermHead<'a>>> {
        match self {
            Self::Ordinary { source, row, terms } => {
                Ok(terms.next().map(|(term, tf)| RowTermHead {
                    term: Cow::Borrowed(term.as_str()),
                    source: *source,
                    row: *row,
                    tf: *tf,
                }))
            }
            Self::Staged {
                source,
                row,
                reader,
                next,
                count,
            } => {
                if *next == *count {
                    return Ok(None);
                }
                let ordinal = *next;
                *next += 1;
                let term = reader
                    .keyword_term_at_ordinal_cow(ordinal)
                    .ok_or_else(|| anyhow!("staged Text dictionary entry is corrupt"))?;
                let posting = reader
                    .text_postings_arc(term.as_ref())
                    .ok_or_else(|| anyhow!("staged Text posting is corrupt"))?;
                if posting.0.len() != 1 || posting.1.len() != 1 || posting.0[0] != 0 {
                    bail!("staged Text row posting is not a single local row");
                }
                Ok(Some(RowTermHead {
                    term,
                    source: *source,
                    row: *row,
                    tf: posting.1[0],
                }))
            }
        }
    }
}

struct RowTermHead<'a> {
    term: Cow<'a, str>,
    source: usize,
    row: u32,
    tf: u32,
}

impl PartialEq for RowTermHead<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term && self.source == other.source
    }
}

impl Eq for RowTermHead<'_> {}

impl PartialOrd for RowTermHead<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RowTermHead<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .term
            .cmp(&self.term)
            .then_with(|| other.source.cmp(&self.source))
    }
}

/// One dictionary pass. It retains one `(term, tf)` head for each present row
/// and produces the posting for the returned term before advancing that row.
struct RowsTerms<'projection, 'rows> {
    cursors: Vec<RowTermCursor<'rows>>,
    heads: BinaryHeap<RowTermHead<'rows>>,
    current: &'projection RefCell<Option<CurrentPosting<'rows>>>,
    failed: bool,
}

impl<'projection, 'rows> RowsTerms<'projection, 'rows> {
    fn new(projection: &'projection RowsProjection<'rows>) -> Result<Self> {
        projection.current.replace(None);
        let mut cursors = Vec::new();
        for (id, (_, value)) in projection.rows.iter().enumerate() {
            let row = u32::try_from(id).map_err(|_| anyhow!("Text row id exceeds u32"))?;
            let source = cursors.len();
            match value.as_deref() {
                Some(CheckpointValue::Text { tokens, .. }) => {
                    cursors.push(RowTermCursor::Ordinary {
                        source,
                        row,
                        terms: tokens.iter(),
                    })
                }
                Some(CheckpointValue::StagedText(staged)) => {
                    let reader = staged.reader();
                    let count = reader
                        .keyword_ordinal_count()
                        .ok_or_else(|| anyhow!("staged Text dictionary is missing"))?;
                    cursors.push(RowTermCursor::Staged {
                        source,
                        row,
                        reader,
                        next: 0,
                        count,
                    });
                }
                None => {}
                Some(_) => bail!("text checkpoint value mismatch"),
            }
        }
        let mut heads = BinaryHeap::new();
        for cursor in &mut cursors {
            if let Some(head) = cursor.next()? {
                heads.push(head);
            }
        }
        Ok(Self {
            cursors,
            heads,
            current: &projection.current,
            failed: false,
        })
    }
}

impl<'projection, 'rows> Iterator for RowsTerms<'projection, 'rows> {
    type Item = Result<Cow<'rows, str>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        // The writer has consumed the preceding posting before it asks for the
        // next term. Drop that payload before allocating this term's vectors.
        self.current.replace(None);
        let first = self.heads.pop()?;
        let term = first.term;
        let mut sources = Vec::new();
        let mut ids = Vec::new();
        let mut tfs = Vec::new();
        sources.push(first.source);
        ids.push(first.row);
        tfs.push(first.tf);
        while self.heads.peek().is_some_and(|head| head.term == term) {
            let head = self.heads.pop().expect("heap head checked");
            sources.push(head.source);
            ids.push(head.row);
            tfs.push(head.tf);
        }
        for source in sources {
            let cursor = &mut self.cursors[source];
            match cursor.next() {
                Ok(Some(next)) => self.heads.push(next),
                Ok(None) => {}
                Err(error) => {
                    self.failed = true;
                    return Some(Err(error));
                }
            }
        }
        self.current.replace(Some(CurrentPosting {
            term: term.clone(),
            posting: Arc::new((ids, tfs)),
        }));
        Some(Ok(term))
    }
}

#[cfg(test)]
impl RowsProjection<'_> {
    fn rows_examined(&self) -> usize {
        self.rows_examined.get()
    }
}
impl TextStreamView for RowsProjection<'_> {
    fn n_docs(&self) -> u32 {
        self.rows.len() as u32
    }
    fn text_is_present(&self, id: u32) -> bool {
        self.rows[id as usize].1.is_some()
    }
    fn text_doc_len(&self, id: u32) -> u32 {
        match self.rows[id as usize].1.as_deref() {
            Some(CheckpointValue::Text { doc_len, .. }) => *doc_len,
            Some(CheckpointValue::StagedText(row)) => row.doc_len(),
            _ => 0,
        }
    }
    fn terms<'a>(&'a self) -> Result<Terms<'a>> {
        // The cache keeps the row lifetime. Narrow only each yielded Cow to
        // this iterator borrow; RefCell itself is invariant in that lifetime.
        Ok(Box::new(
            RowsTerms::new(self)?.map(|term| term.map(|value| -> Cow<'a, str> { value })),
        ))
    }
    fn text_postings(&self, term: &str) -> Result<Option<Arc<(Vec<u32>, Vec<u32>)>>> {
        if let Some(posting) = self
            .current
            .borrow()
            .as_ref()
            .filter(|posting| posting.term.as_ref() == term)
            .map(|posting| posting.posting.clone())
        {
            return Ok(Some(posting));
        }
        let mut out = Vec::new();
        for (id, (_, value)) in self.rows.iter().enumerate() {
            #[cfg(test)]
            self.rows_examined.set(self.rows_examined.get() + 1);
            let tf = match value.as_deref() {
                Some(CheckpointValue::Text { tokens, .. }) => tokens.get(term).copied(),
                Some(CheckpointValue::StagedText(row)) => {
                    match row.reader().text_postings_arc(term) {
                        Some(posting)
                            if posting.0.len() == 1
                                && posting.1.len() == 1
                                && posting.0[0] == 0 =>
                        {
                            Some(posting.1[0])
                        }
                        Some(_) => bail!("staged Text row posting is not a single local row"),
                        None => None,
                    }
                }
                _ => None,
            };
            if let Some(tf) = tf {
                out.push((id as u32, tf));
            }
        }
        Ok((!out.is_empty()).then(|| Arc::new(out.into_iter().unzip())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::text_row_stage::TextRowStageOptions;
    use crate::types::Analyzer;
    use std::collections::BTreeMap;

    fn ordinary(
        doc_len: u32,
        terms: &[(&str, u32)],
    ) -> crate::change_journal::SharedValue<CheckpointValue> {
        crate::change_journal::SharedValue::new(
            Arc::new(CheckpointValue::Text {
                doc_len,
                tokens: terms
                    .iter()
                    .map(|(term, tf)| ((*term).to_owned(), *tf))
                    .collect::<BTreeMap<_, _>>(),
            }),
            None,
        )
    }

    fn staged(input: &str) -> crate::change_journal::SharedValue<CheckpointValue> {
        let row = super::super::staged_text_row::StagedTextRow::stage(
            input,
            Analyzer::WhitespaceLower,
            TextRowStageOptions::minimum_scratch_bytes() + 4096,
            |_| Ok(()),
        )
        .unwrap();
        crate::change_journal::SharedValue::new(
            Arc::new(CheckpointValue::StagedText(Arc::new(row))),
            None,
        )
    }

    #[test]
    fn rows_projection_merges_current_term_postings_without_rescanning_rows() {
        let rows = vec![
            (
                "zero".to_owned(),
                Some(ordinary(3, &[("alpha", 2), ("beta", 1)])),
            ),
            ("deleted".to_owned(), None),
            ("empty".to_owned(), Some(ordinary(0, &[]))),
            (
                "three".to_owned(),
                Some(ordinary(4, &[("alpha", 1), ("gamma", 3)])),
            ),
            ("four".to_owned(), Some(staged("beta beta delta"))),
        ];
        let view = RowsProjection {
            rows: &rows,
            current: RefCell::new(None),
            rows_examined: Cell::new(0),
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rows.lseg");

        write_text_projection(&path, 41, &view).unwrap();

        let reader = SegmentReader::open(&path).unwrap();
        assert_eq!(
            reader.text_postings("alpha"),
            Some((vec![0, 3], vec![2, 1]))
        );
        assert_eq!(reader.text_postings("beta"), Some((vec![0, 4], vec![1, 2])));
        assert_eq!(reader.text_postings("delta"), Some((vec![4], vec![1])));
        assert_eq!(reader.text_postings("gamma"), Some((vec![3], vec![3])));
        assert!(reader.text_is_present(0));
        assert!(!reader.text_is_present(1));
        assert!(reader.text_is_present(2));
        assert_eq!(reader.text_doc_len(2), 0);
        assert_eq!(
            view.rows_examined(),
            0,
            "streaming writer must use the current-term cache"
        );
    }

    #[test]
    fn rows_projection_keeps_point_lookup_when_the_term_is_not_current() {
        let rows = vec![
            (
                "zero".to_owned(),
                Some(ordinary(2, &[("alpha", 1), ("zeta", 1)])),
            ),
            ("one".to_owned(), Some(ordinary(1, &[("beta", 3)]))),
        ];
        let view = RowsProjection {
            rows: &rows,
            current: RefCell::new(None),
            rows_examined: Cell::new(0),
        };

        let mut terms = view.terms().unwrap();
        assert_eq!(terms.next().unwrap().unwrap(), "alpha");
        assert_eq!(
            view.text_postings("alpha").unwrap().as_deref(),
            Some(&(vec![0], vec![1]))
        );
        assert_eq!(view.rows_examined(), 0);
        assert_eq!(
            view.text_postings("zeta").unwrap().as_deref(),
            Some(&(vec![0], vec![1]))
        );
        assert_eq!(view.rows_examined(), rows.len());
    }

    #[test]
    fn checkpoint_rows_reject_non_text_values_before_streaming() {
        let rows = vec![(
            "wrong".to_owned(),
            Some(crate::change_journal::SharedValue::new(
                Arc::new(CheckpointValue::Keyword("not text".to_owned())),
                None,
            )),
        )];
        let dir = tempfile::tempdir().unwrap();
        assert!(write_checkpoint_rows(&dir.path().join("wrong.lseg"), 1, &rows).is_err());
    }
}
