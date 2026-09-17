//! Bounded staging for long `WhitespaceLower` tokens.
//!
//! A caller supplies the final row path inside its private stage directory.
//! This helper owns a child directory and removes all of its temporary files
//! before returning. The caller owns the final row file.

use std::borrow::Cow;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(test)]
use std::cell::Cell;

use anyhow::{anyhow, bail, Context, Result};
use memmap2::{Mmap, MmapOptions};

use super::unicode_lower_stream::{lowercase_stream_workspace_bytes, write_streaming_lowercase};
use crate::segment::stream::{write_text_projection, TextStreamView};
use crate::segment::text_row_stage::{stage_text_row, TextRowStageOptions};
use crate::segment::SegmentReader;

pub(crate) const LARGE_TOKEN_SOURCE_BYTES: usize = 64 * 1024;
const MAPPED_TOKEN_BYTES: usize = 256;
const IO_BUFFER_BYTES: usize = 8 * 1024;
const RAW_OFFSET_COPY_BYTES: usize = 64 * 1024;
const VAR_BLOCK_BYTES: usize = 64 * 1024;
static NONCE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum TokenFailure {
    Normalize,
    Map,
}
#[cfg(test)]
thread_local! { static TOKEN_FAILURE: Cell<Option<TokenFailure>> = const { Cell::new(None) }; }
#[cfg(test)]
pub(super) fn set_token_failure_for_test(failure: Option<TokenFailure>) {
    TOKEN_FAILURE.with(|current| current.set(failure));
}
fn token_failure(_stage: &'static str) -> Result<()> {
    #[cfg(test)]
    if TOKEN_FAILURE
        .with(|current| current.get())
        .is_some_and(|failure| {
            matches!(
                (_stage, failure),
                ("normalize", TokenFailure::Normalize) | ("map", TokenFailure::Map)
            )
        })
    {
        bail!("injected large Text token {_stage} failure");
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) struct LargeTextRowReceipt {
    pub(crate) doc_len: u32,
    pub(crate) final_reader_metadata_bytes: usize,
}

/// Stage one exact `for_whitespace_lower_cow` row.  `reserve` is called before
/// every helper-owned allocation. It is additive: `options.scratch_bytes` and
/// the caller's `3 * threshold + 64` short-lower workspace were already
/// reserved by the enclosing staged-row owner and are not charged again.
pub(crate) fn stage_large_whitespace_row(
    input: &str,
    final_path: &Path,
    scratch_dir: &Path,
    options: TextRowStageOptions,
    threshold: usize,
    mut reserve: impl FnMut(usize) -> Result<()>,
) -> Result<LargeTextRowReceipt> {
    if threshold == 0 {
        bail!("large Text token threshold must be nonzero")
    }
    reserve(4096)?; // Private directory, transient paths, and row-view metadata.
    let cleanup = PrivateDirectory::create(scratch_dir)?;
    let mut long = Vec::new();
    let mut doc_len = 0u32;
    let mut small_len = 0u32;
    visit_tokens(input, |token| {
        doc_len = doc_len
            .checked_add(1)
            .ok_or_else(|| anyhow!("Text document length exceeds u32"))?;
        if token.len() > threshold {
            if long.is_empty() {
                reserve(lowercase_stream_workspace_bytes())?;
            }
            reserve_vector_growth(&mut long, &mut reserve)?;
            reserve(MAPPED_TOKEN_BYTES)?;
            long.push(MappedToken::write(token, cleanup.path())?);
        } else {
            small_len = small_len
                .checked_add(1)
                .ok_or_else(|| anyhow!("Text document length exceeds u32"))?;
        }
        Ok(())
    })?;

    let small_path = cleanup.path().join("small-row.lseg");
    stage_text_row(
        &small_path,
        0,
        small_len,
        cleanup.path(),
        options,
        Box::new(|emit| {
            visit_tokens(input, |token| {
                if token.len() <= threshold {
                    emit_small_lower(token, emit)?;
                }
                Ok(())
            })
        }),
    )?;
    reserve(SegmentReader::staged_metadata_bound(&small_path)?)?;
    let small = Arc::new(SegmentReader::open(&small_path)?);

    long.sort_unstable_by(|left, right| left.as_str().cmp(right.as_str()));
    let mut groups = Vec::new();
    let mut first = 0usize;
    while first < long.len() {
        reserve_vector_growth(&mut groups, &mut reserve)?;
        let term = long[first].as_str();
        let mut end = first + 1;
        let mut tf = 1u32;
        while end < long.len() && long[end].as_str() == term {
            tf = tf
                .checked_add(1)
                .ok_or_else(|| anyhow!("Text term frequency exceeds u32"))?;
            end += 1;
        }
        groups.push(Group { first, tf });
        first = end;
    }

    reserve(raw_text_projection_workspace_bytes()?)?;
    let view = OneRow {
        small: &small,
        long: &long,
        groups: &groups,
        doc_len,
    };
    write_text_projection(final_path, 0, &view)?;
    let final_reader_metadata_bytes = SegmentReader::staged_metadata_bound(final_path)?;
    Ok(LargeTextRowReceipt {
        doc_len,
        final_reader_metadata_bytes,
    })
}

/// The exact shared scanner: Unicode whitespace separates, then leading and
/// trailing non-alphanumeric scalars are removed before threshold selection.
fn visit_tokens(input: &str, mut emit: impl FnMut(&str) -> Result<()>) -> Result<()> {
    let mut rest = input;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let raw = &rest[..end];
        rest = &rest[end..];
        let token = raw.trim_matches(|c: char| !c.is_alphanumeric());
        if !token.is_empty() {
            emit(token)?;
        }
    }
    Ok(())
}

fn emit_small_lower(token: &str, emit: &mut dyn FnMut(&str) -> Result<()>) -> Result<()> {
    if token
        .as_bytes()
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return emit(token);
    }
    // Token source bytes are bounded by `threshold`; the existing row writer
    // immediately copies this short temporary into its priced run storage.
    emit(&token.to_lowercase())
}

struct MappedToken {
    mmap: Mmap,
}
impl MappedToken {
    fn write(token: &str, dir: &Path) -> Result<Self> {
        let path = fresh(dir, "large-token", "utf8");
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .with_context(|| format!("create normalized Text token {}", path.display()))?;
        // The enclosing call reserved both handle and fixed lowercase workspace
        // before creating this file; the lower primitive's confirming callback
        // therefore performs no new allocation admission.
        write_streaming_lowercase(token, &mut file, |_| Ok(()))?;
        token_failure("normalize")?;
        file.sync_all()?;
        token_failure("map")?;
        let mmap =
            unsafe { MmapOptions::new().map(&file) }.context("mmap normalized Text token")?;
        if mmap.is_empty() {
            bail!("trimmed Text token normalized to empty")
        }
        // SAFETY: the only writer is the lowercase primitive, which emits UTF-8;
        // sync completed before mapping and this private file is never reopened.
        let _ = unsafe { std::str::from_utf8_unchecked(&mmap) };
        Ok(Self { mmap })
    }
    fn as_str(&self) -> &str {
        // SAFETY: established in `write`; immutable mmap lifetime is `self`.
        unsafe { std::str::from_utf8_unchecked(&self.mmap) }
    }
}

#[derive(Clone, Copy)]
struct Group {
    first: usize,
    tf: u32,
}

struct OneRow<'a> {
    small: &'a SegmentReader,
    long: &'a [MappedToken],
    groups: &'a [Group],
    doc_len: u32,
}
impl OneRow<'_> {
    fn long_tf(&self, term: &str) -> Option<u32> {
        self.groups
            .binary_search_by(|group| self.long[group.first].as_str().cmp(term))
            .ok()
            .map(|index| self.groups[index].tf)
    }
}
impl TextStreamView for OneRow<'_> {
    fn n_docs(&self) -> u32 {
        1
    }
    fn text_is_present(&self, id: u32) -> bool {
        id == 0
    }
    fn text_doc_len(&self, id: u32) -> u32 {
        if id == 0 {
            self.doc_len
        } else {
            0
        }
    }
    fn terms<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Result<Cow<'a, str>>> + 'a>> {
        Ok(Box::new(OneTerms::new(self)?))
    }
    fn text_postings(&self, term: &str) -> Result<Option<Arc<(Vec<u32>, Vec<u32>)>>> {
        let long = self.long_tf(term).unwrap_or(0);
        let small = self
            .small
            .text_postings_arc(term)
            .map(|posting| {
                if posting.0.as_slice() != [0] || posting.1.len() != 1 {
                    return Err(anyhow!("small Text posting is not local"));
                }
                Ok(posting.1[0])
            })
            .transpose()?
            .unwrap_or(0);
        let tf = long
            .checked_add(small)
            .ok_or_else(|| anyhow!("Text term frequency exceeds u32"))?;
        Ok((tf != 0).then(|| Arc::new((vec![0], vec![tf]))))
    }
}

struct OneTerms<'a> {
    view: &'a OneRow<'a>,
    next_small: u32,
    small_count: u32,
    small: Option<Cow<'a, str>>,
    group: usize,
}
impl<'a> OneTerms<'a> {
    fn new(view: &'a OneRow<'a>) -> Result<Self> {
        let small_count = view
            .small
            .keyword_ordinal_count()
            .ok_or_else(|| anyhow!("small Text dictionary is missing"))?;
        let mut result = Self {
            view,
            next_small: 0,
            small_count,
            small: None,
            group: 0,
        };
        result.advance_small()?;
        Ok(result)
    }
    fn advance_small(&mut self) -> Result<()> {
        self.small = if self.next_small == self.small_count {
            None
        } else {
            let ordinal = self.next_small;
            self.next_small += 1;
            Some(
                self.view
                    .small
                    .keyword_term_at_ordinal_cow(ordinal)
                    .ok_or_else(|| anyhow!("small Text dictionary is corrupt"))?,
            )
        };
        Ok(())
    }
}
impl<'a> Iterator for OneTerms<'a> {
    type Item = Result<Cow<'a, str>>;
    fn next(&mut self) -> Option<Self::Item> {
        let long = self
            .view
            .groups
            .get(self.group)
            .map(|group| self.view.long[group.first].as_str());
        match (self.small.as_ref(), long) {
            (None, None) => None,
            (None, Some(term)) => {
                self.group += 1;
                Some(Ok(Cow::Borrowed(term)))
            }
            (Some(_), None) => {
                let term = self.small.take().unwrap();
                match self.advance_small() {
                    Ok(()) => Some(Ok(term)),
                    Err(error) => Some(Err(error)),
                }
            }
            (Some(small), Some(term)) if small.as_ref() < term => {
                let term = self.small.take().unwrap();
                match self.advance_small() {
                    Ok(()) => Some(Ok(term)),
                    Err(error) => Some(Err(error)),
                }
            }
            (Some(small), Some(term)) if small.as_ref() > term => {
                self.group += 1;
                Some(Ok(Cow::Borrowed(term)))
            }
            (Some(_), Some(term)) => {
                self.group += 1;
                match self.advance_small() {
                    Ok(()) => Some(Ok(Cow::Borrowed(term))),
                    Err(error) => Some(Err(error)),
                }
            }
        }
    }
}

fn reserve_vector_growth<T>(
    values: &mut Vec<T>,
    reserve: &mut impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    if values.len() != values.capacity() {
        return Ok(());
    }
    let next = values
        .capacity()
        .checked_mul(2)
        .map(|n| n.max(4))
        .ok_or_else(|| anyhow!("large Text metadata capacity overflow"))?;
    // Price the full new allocation while the previous allocation is still
    // live. Explicit reserve avoids Vec::push's implicit minimum capacity.
    reserve(
        next.checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| anyhow!("large Text metadata capacity overflow"))?,
    )?;
    values
        .try_reserve_exact(next - values.len())
        .map_err(|error| anyhow!("reserve large Text metadata: {error}"))?;
    Ok(())
}

/// The raw dictionary writer has an outer and offsets `BufWriter` (8KiB each),
/// a 64KiB offsets-copy buffer, and one one-row posting block. One posting is
/// only count/docid/TF, so three 64KiB blocks cover suffixes, CBOR, and LZ4.
fn raw_text_projection_workspace_bytes() -> Result<usize> {
    IO_BUFFER_BYTES
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(RAW_OFFSET_COPY_BYTES))
        .and_then(|bytes| bytes.checked_add(VAR_BLOCK_BYTES.checked_mul(3)?))
        .ok_or_else(|| anyhow!("raw Text projection workspace overflow"))
}

struct PrivateDirectory {
    path: PathBuf,
}
impl PrivateDirectory {
    fn create(parent: &Path) -> Result<Self> {
        fs::create_dir_all(parent)?;
        for _ in 0..128 {
            let path = parent.join(format!(
                ".large-text-row-{}-{}",
                std::process::id(),
                NONCE.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        }
        bail!("could not create private large Text staging directory")
    }
    fn path(&self) -> &Path {
        &self.path
    }
}
impl Drop for PrivateDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn fresh(dir: &Path, kind: &str, extension: &str) -> PathBuf {
    dir.join(format!(
        ".{kind}-{}-{}.{}",
        std::process::id(),
        NONCE.fetch_add(1, Ordering::Relaxed),
        extension
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn oracle(input: &str) -> BTreeMap<String, u32> {
        let mut terms = BTreeMap::new();
        crate::tokenize::for_whitespace_lower_cow(input, |term| {
            *terms.entry(term.into_owned()).or_insert(0) += 1;
        });
        terms
    }

    #[test]
    fn small_threshold_merges_two_long_casefolds_with_a_short_collision() {
        let dir = tempfile::tempdir().unwrap();
        // U+212A lowers to one-byte `k`: both first terms are long at four
        // bytes, while KKK is short, and all three must become `kkk`.
        let input = "!!!KKK!!! KKk KKK AΣ !!!";
        let final_path = dir.path().join("row.lseg");
        let row = stage_large_whitespace_row(
            input,
            &final_path,
            dir.path(),
            TextRowStageOptions {
                scratch_bytes: TextRowStageOptions::minimum_scratch_bytes() + 4096,
            },
            4,
            |_| Ok(()),
        )
        .unwrap();

        assert!(row.final_reader_metadata_bytes > 0);
        let reader = SegmentReader::open(&final_path).unwrap();
        let expected = oracle(input);
        assert_eq!(row.doc_len, 4);
        assert_eq!(reader.text_doc_len(0), 4);
        assert_eq!(reader.text_doc_count(), 1);
        for (term, tf) in expected {
            assert_eq!(reader.text_postings(&term), Some((vec![0], vec![tf])));
        }
        assert!(matches!(
            (0..reader.keyword_ordinal_count().unwrap()).find_map(|ordinal| {
                reader
                    .keyword_term_at_ordinal_cow(ordinal)
                    .and_then(|term| (term == "kkk").then_some(term))
            }),
            Some(std::borrow::Cow::Borrowed("kkk"))
        ));
        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(names, vec!["row.lseg"]);
    }

    #[test]
    fn empty_trimmed_input_is_present_with_zero_document_length() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("empty.lseg");
        let row = stage_large_whitespace_row(
            " !!\u{2003}... ",
            &final_path,
            dir.path(),
            TextRowStageOptions {
                scratch_bytes: TextRowStageOptions::minimum_scratch_bytes() + 4096,
            },
            4,
            |_| Ok(()),
        )
        .unwrap();
        let reader = SegmentReader::open(&final_path).unwrap();
        assert_eq!(row.doc_len, 0);
        assert!(reader.text_is_present(0));
        assert_eq!(reader.text_doc_len(0), 0);
        assert_eq!(reader.keyword_ordinal_count(), Some(0));
    }

    #[test]
    fn refusing_a_long_handle_reservation_creates_no_private_files() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("never.lseg");
        let error = stage_large_whitespace_row(
            "KKK",
            &final_path,
            dir.path(),
            TextRowStageOptions {
                scratch_bytes: TextRowStageOptions::minimum_scratch_bytes() + 4096,
            },
            4,
            |_| anyhow::bail!("refuse"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("refuse"));
        assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
    }

    #[test]
    fn normalization_and_map_failure_remove_the_whole_private_child() {
        for failure in [TokenFailure::Normalize, TokenFailure::Map] {
            let dir = tempfile::tempdir().unwrap();
            let final_path = dir.path().join("never.lseg");
            set_token_failure_for_test(Some(failure));
            let error = stage_large_whitespace_row(
                "KKK",
                &final_path,
                dir.path(),
                TextRowStageOptions {
                    scratch_bytes: TextRowStageOptions::minimum_scratch_bytes() + 4096,
                },
                4,
                |_| Ok(()),
            )
            .unwrap_err();
            set_token_failure_for_test(None);
            assert!(error.to_string().contains("injected"));
            assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
        }
    }
}
