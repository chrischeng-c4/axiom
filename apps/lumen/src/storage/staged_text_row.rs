//! A file-owned, one-row Text reader prepared before apply.
//!
//! The staging writer owns bounded scratch space. This receipt owns only the
//! final staged file through its reader. The caller must reserve raw input,
//! staging workspace, row handles, and reader metadata separately.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};

#[cfg(test)]
use std::cell::RefCell;

use crate::segment::text_row_stage::{stage_text_row, TextRowStageOptions, TextTokenStream};
use crate::segment::SegmentReader;
use crate::types::Analyzer;

static STAGED_TEXT_ROW_NONCE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
thread_local! {
    static LAST_STAGE_DIRECTORY: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Test-only observation of actual stage-directory creation. It has no effect
/// on stage ownership or cleanup.
#[cfg(test)]
pub(super) fn last_stage_directory_for_test() -> Option<PathBuf> {
    LAST_STAGE_DIRECTORY.with(|last| last.borrow().clone())
}

/// One prepared Text value. The reader's private stage guard keeps the exact
/// locally-created directory alive until its final `Arc` is dropped.
#[derive(Debug)]
pub(super) struct StagedTextRow {
    reader: Arc<SegmentReader>,
    input_bytes: usize,
    doc_len: u32,
    unique_term_count: u64,
    total_unique_term_bytes: u64,
}

/// Temporary workspace is released when staging returns; reader bytes follow
/// the final mmap owner through apply and checkpoint.
#[derive(Clone, Copy)]
pub(super) enum StageAllocation {
    Workspace(usize),
    Reader(usize),
}

impl StagedTextRow {
    #[cfg(test)]
    pub(super) fn stage(
        input: &str,
        analyzer: Analyzer,
        scratch_bytes: usize,
        mut reserve_reader: impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        Self::stage_charged(
            input,
            analyzer,
            scratch_bytes,
            |allocation| match allocation {
                StageAllocation::Reader(bytes) => reserve_reader(bytes),
                StageAllocation::Workspace(_) => Ok(()),
            },
        )
    }

    /// Stage before apply. The caller has priced the row writer and the
    /// bounded short-token normalizer before entering this method.
    pub(super) fn stage_charged(
        input: &str,
        analyzer: Analyzer,
        scratch_bytes: usize,
        mut reserve: impl FnMut(StageAllocation) -> Result<()>,
    ) -> Result<Self> {
        let mut stage_dir = StageDirectory::create()?;
        let segment_path = stage_dir.path().join("row.lseg");
        let options = TextRowStageOptions { scratch_bytes };
        let has_large_token = analyzer == Analyzer::WhitespaceLower
            && input.split_whitespace().any(|raw| {
                raw.trim_matches(|c: char| !c.is_alphanumeric()).len()
                    > super::large_text_row::LARGE_TOKEN_SOURCE_BYTES
            });
        let (doc_len, reader_bytes) = if has_large_token {
            let receipt = super::large_text_row::stage_large_whitespace_row(
                input,
                &segment_path,
                stage_dir.path(),
                options,
                super::large_text_row::LARGE_TOKEN_SOURCE_BYTES,
                |bytes| reserve(StageAllocation::Workspace(bytes)),
            )?;
            (receipt.doc_len, receipt.final_reader_metadata_bytes)
        } else {
            #[cfg(feature = "jieba")]
            let mut route = if analyzer == Analyzer::Jieba {
                Some(super::jieba_disk_route::DiskRoute::create(
                    stage_dir.path(),
                )?)
            } else {
                None
            };
            let doc_len = document_len(
                input,
                analyzer,
                #[cfg(feature = "jieba")]
                route.as_mut(),
            )?;
            let stream = token_stream(
                input,
                analyzer,
                #[cfg(feature = "jieba")]
                route,
            )?;
            stage_text_row(&segment_path, 0, doc_len, stage_dir.path(), options, stream)?;
            (
                doc_len,
                SegmentReader::staged_metadata_bound(&segment_path)?,
            )
        };

        // Every temporary helper mapping has dropped. Price the final reader
        // before opening it, then transfer the directory to its exact owner.
        reserve(StageAllocation::Reader(reader_bytes))?;
        let reader = Arc::new(SegmentReader::open_owned_stage(
            &segment_path,
            stage_dir.path().to_owned(),
        )?);
        let (unique_term_count, total_unique_term_bytes) = reader
            .text_dictionary_stats()
            .ok_or_else(|| anyhow!("staged Text dictionary is unreadable"))?;
        stage_dir.transfer();
        Ok(Self {
            reader,
            input_bytes: input.len(),
            doc_len,
            unique_term_count,
            total_unique_term_bytes,
        })
    }

    pub(super) fn reader(&self) -> &Arc<SegmentReader> {
        &self.reader
    }

    pub(super) fn doc_len(&self) -> u32 {
        self.doc_len
    }

    pub(super) fn input_bytes(&self) -> usize {
        self.input_bytes
    }

    /// Match the live Text index accounting: each distinct token stores its
    /// bytes plus this document's external ID once.
    pub(super) fn indexed_bytes(&self, external_id: &str) -> u64 {
        // Match the existing live `TextIndex::bytes` arithmetic exactly. A
        // staged row has at most its checked `u32` document-token count, so
        // these sums are bounded by the admitted record payload in practice.
        self.total_unique_term_bytes + self.unique_term_count * external_id.len() as u64
    }
}

struct StageDirectory {
    path: PathBuf,
    transferred: bool,
}

impl StageDirectory {
    fn create() -> Result<Self> {
        let root = std::env::temp_dir();
        let process = std::process::id();
        for _ in 0..128 {
            let nonce = STAGED_TEXT_ROW_NONCE.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!("lumen-staged-text-row-{process}-{nonce}"));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => {
                    #[cfg(test)]
                    LAST_STAGE_DIRECTORY.with(|last| *last.borrow_mut() = Some(path.clone()));
                    return Ok(Self {
                        path,
                        transferred: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("create staged Text directory {}", path.display())
                    });
                }
            }
        }
        bail!("could not allocate a unique staged Text directory")
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn transfer(&mut self) {
        self.transferred = true;
    }
}

impl Drop for StageDirectory {
    fn drop(&mut self) {
        if !self.transferred {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn document_len(
    input: &str,
    analyzer: Analyzer,
    #[cfg(feature = "jieba")] route: Option<&mut super::jieba_disk_route::DiskRoute>,
) -> Result<u32> {
    match analyzer {
        Analyzer::Ngram => {
            crate::ngram_stream::stream_default_ngrams(input, |_| Ok::<_, anyhow::Error>(()))
                .map_err(|error| anyhow!("stream staged Ngram Text length: {error}"))
        }
        Analyzer::WhitespaceLower => Ok(crate::tokenize::for_whitespace_lower_cow(input, |_| {})),
        #[cfg(not(feature = "jieba"))]
        Analyzer::Jieba => crate::jieba_fallback_stream::stream_fallback_jieba(input, |_| {
            Ok::<_, anyhow::Error>(())
        })
        .map_err(|error| anyhow!("stream staged fallback Jieba Text length: {error}")),
        #[cfg(feature = "jieba")]
        Analyzer::Jieba => Ok(index_text::for_jieba_no_hmm(
            input,
            route.ok_or_else(|| anyhow!("missing dictionary Jieba route"))?,
            |_| Ok(()),
        )?),
    }
}

fn token_stream<'a>(
    input: &'a str,
    analyzer: Analyzer,
    #[cfg(feature = "jieba")] route: Option<super::jieba_disk_route::DiskRoute>,
) -> Result<Box<TextTokenStream<'a>>> {
    match analyzer {
        Analyzer::Ngram => Ok(Box::new(
            move |emit| match crate::ngram_stream::stream_default_ngrams(input, |token| emit(token))
            {
                Ok(_) => Ok(()),
                Err(crate::ngram_stream::NgramStreamError::Callback(error)) => Err(error),
                Err(crate::ngram_stream::NgramStreamError::TokenCountOverflow) => {
                    bail!("staged Ngram Text document length exceeds u32")
                }
            },
        )),
        Analyzer::WhitespaceLower => Ok(Box::new(move |emit| {
            let mut failure = None;
            crate::tokenize::for_whitespace_lower_cow(input, |token| {
                if failure.is_none() {
                    if let Err(error) = emit(token.as_ref()) {
                        failure = Some(error);
                    }
                }
            });
            match failure {
                Some(error) => Err(error),
                None => Ok(()),
            }
        })),
        #[cfg(not(feature = "jieba"))]
        Analyzer::Jieba => {
            Ok(Box::new(
                move |emit| match crate::jieba_fallback_stream::stream_fallback_jieba(
                    input,
                    |token| emit(token),
                ) {
                    Ok(_) => Ok(()),
                    Err(crate::jieba_fallback_stream::JiebaFallbackStreamError::Callback(
                        error,
                    )) => Err(error),
                    Err(
                        crate::jieba_fallback_stream::JiebaFallbackStreamError::TokenCountOverflow,
                    ) => {
                        bail!("staged fallback Jieba Text document length exceeds u32")
                    }
                },
            ))
        }
        #[cfg(feature = "jieba")]
        Analyzer::Jieba => {
            let mut route = route.ok_or_else(|| anyhow!("missing dictionary Jieba route"))?;
            Ok(Box::new(move |emit| {
                // Preserve the callback's concrete workspace error for the
                // caller's bounded retry. I/O errors keep their own cause.
                let mut failure = None;
                let result =
                    index_text::for_jieba_no_hmm(input, &mut route, |token| match emit(token) {
                        Ok(()) => Ok(()),
                        Err(error) => {
                            failure = Some(error);
                            Err(std::io::Error::other("stop staged Jieba callback"))
                        }
                    });
                match failure {
                    Some(error) => Err(error),
                    None => result.map(|_| ()).map_err(Into::into),
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn staged(analyzer: Analyzer, input: &str) -> StagedTextRow {
        StagedTextRow::stage(
            input,
            analyzer,
            TextRowStageOptions::minimum_scratch_bytes() + 4096,
            |_| Ok(()),
        )
        .unwrap()
    }

    #[test]
    fn ngram_receipt_has_exact_term_frequency_document_length_and_empty_row() {
        let row = staged(Analyzer::Ngram, "ababa");
        assert_eq!(row.input_bytes(), 5);
        assert_eq!(row.doc_len(), 7);
        assert_eq!(row.indexed_bytes("doc"), 22);
        let reader = row.reader();
        assert_eq!(reader.text_doc_len(0), 7);
        assert_eq!(reader.text_postings("ab"), Some((vec![0], vec![2])));
        assert_eq!(reader.text_postings("ba"), Some((vec![0], vec![2])));
        assert_eq!(reader.text_postings("aba"), Some((vec![0], vec![2])));
        assert_eq!(reader.text_postings("bab"), Some((vec![0], vec![1])));

        let empty = staged(Analyzer::Ngram, " ");
        assert_eq!(empty.doc_len(), 0);
        assert_eq!(empty.reader().text_doc_len(0), 0);
        assert_eq!(empty.reader().text_postings("missing"), None);
    }

    #[cfg(not(feature = "jieba"))]
    #[test]
    fn fallback_jieba_receipt_matches_shipped_tokens_and_empty_presence() {
        let input = "  lumen 搜尋引擎 ΣΟΣ  ";
        let row = staged(Analyzer::Jieba, input);
        let expected = crate::tokenize::tokenize(input, Analyzer::Jieba);
        let mut frequencies = BTreeMap::<String, u32>::new();
        for token in expected {
            *frequencies.entry(token).or_default() += 1;
        }
        assert_eq!(row.doc_len(), frequencies.values().sum::<u32>());
        for (token, frequency) in frequencies {
            assert_eq!(
                row.reader().text_postings(&token),
                Some((vec![0], vec![frequency]))
            );
        }
        let empty = staged(Analyzer::Jieba, " \u{3000}\t ");
        assert_eq!(empty.doc_len(), 0);
        assert_eq!(empty.reader().text_doc_len(0), 0);
    }

    #[cfg(feature = "jieba")]
    #[test]
    fn dictionary_jieba_receipt_matches_existing_cut_and_removes_route_file() {
        let input = format!(
            "{} abc123+_#&.%-XYZ\n\tΣΟΣ İSTANBUL 👪",
            "南京市长江大桥".repeat(1000)
        );
        let row = staged(Analyzer::Jieba, &input);
        let expected = crate::tokenize::tokenize(&input, Analyzer::Jieba);
        assert_eq!(row.doc_len(), expected.len() as u32);
        let mut frequencies = BTreeMap::<String, u32>::new();
        for token in expected {
            *frequencies.entry(token).or_default() += 1;
        }
        for (token, frequency) in frequencies {
            assert_eq!(
                row.reader().text_postings(&token),
                Some((vec![0], vec![frequency])),
                "{token}"
            );
        }
        let directory = LAST_STAGE_DIRECTORY.with(|last| last.borrow().clone().unwrap());
        assert!(directory.join("row.lseg").is_file());
        assert!(!directory.join("jieba-route.tmp").exists());
        drop(row);
        assert!(!directory.exists());
    }

    #[test]
    fn whitespace_lower_receipt_matches_shared_unicode_tokenizer() {
        let input = "  İSTANBUL\tStraße  İSTANBUL ";
        let row = staged(Analyzer::WhitespaceLower, input);
        let expected = crate::tokenize::tokenize(input, Analyzer::WhitespaceLower);
        let mut frequencies = BTreeMap::<String, u32>::new();
        for token in expected {
            *frequencies.entry(token).or_default() += 1;
        }
        assert_eq!(row.doc_len(), frequencies.values().sum::<u32>());
        for (token, frequency) in frequencies {
            assert_eq!(
                row.reader().text_postings(&token),
                Some((vec![0], vec![frequency]))
            );
        }
    }

    #[test]
    fn denied_reader_reservation_removes_the_sealed_private_directory() {
        SegmentReader::reset_owned_stage_open_calls();
        let result = StagedTextRow::stage(
            "reserve after seal",
            Analyzer::WhitespaceLower,
            TextRowStageOptions::minimum_scratch_bytes() + 4096,
            |_| {
                let path = last_stage_dir();
                assert!(path.join("row.lseg").is_file());
                assert_eq!(SegmentReader::owned_stage_open_calls(), 0);
                bail!("deny reader reservation")
            },
        );
        assert!(result.is_err());
        assert!(!last_stage_dir().exists());
    }

    #[test]
    fn reader_arc_keeps_only_its_private_stage_directory_until_last_drop() {
        let row = staged(Analyzer::WhitespaceLower, "kept reader");
        let reader = row.reader().clone();
        let path = reader.owned_stage_dir().unwrap().to_owned();
        assert!(path.is_dir());
        drop(row);
        assert!(path.is_dir());
        drop(reader);
        assert!(!path.exists());
    }

    #[test]
    fn ngram_stream_preserves_retryable_workspace_callback_error() {
        let stream = token_stream(
            "abcd",
            Analyzer::Ngram,
            #[cfg(feature = "jieba")]
            None,
        )
        .unwrap();
        let error = stream(&mut |_| {
            Err(anyhow::Error::new(
                crate::segment::text_row_stage::RequiredTextRowWorkspace {
                    required_bytes: TextRowStageOptions::minimum_scratch_bytes() + 1,
                },
            ))
        })
        .unwrap_err();
        assert!(
            error
                .downcast_ref::<crate::segment::text_row_stage::RequiredTextRowWorkspace>()
                .is_some(),
            "the caller must be able to retry a bounded Ngram workspace request"
        );
    }

    #[cfg(feature = "jieba")]
    #[test]
    fn dictionary_stream_preserves_retryable_workspace_error_and_cleans_route() {
        let directory = StageDirectory::create().unwrap();
        let route = super::super::jieba_disk_route::DiskRoute::create(directory.path()).unwrap();
        let stream = token_stream("南京市长江大桥", Analyzer::Jieba, Some(route)).unwrap();
        let error = stream(&mut |_| {
            Err(anyhow::Error::new(
                crate::segment::text_row_stage::RequiredTextRowWorkspace {
                    required_bytes: TextRowStageOptions::minimum_scratch_bytes() + 1,
                },
            ))
        })
        .unwrap_err();
        assert!(error
            .downcast_ref::<crate::segment::text_row_stage::RequiredTextRowWorkspace>()
            .is_some());
        assert!(!directory.path().join("jieba-route.tmp").exists());
    }

    #[cfg(unix)]
    #[test]
    fn stage_directory_is_private_before_any_token_is_written() {
        use std::os::unix::fs::PermissionsExt;
        let directory = StageDirectory::create().unwrap();
        assert_eq!(
            fs::metadata(directory.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }

    fn last_stage_dir() -> PathBuf {
        LAST_STAGE_DIRECTORY
            .with(|last| last.borrow().clone())
            .unwrap()
    }
}
