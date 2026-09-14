//! Bounded streaming `str::to_lowercase` parity for one whitespace token.
//!
//! `WhitespaceLower` uses `str::to_lowercase` for a non-ASCII token
//! (`libs/index-text/src/lib.rs`). The final-Greek-sigma rule needs both
//! surrounding context, so this writer keeps only a pending sigma file offset.
//! It asks Rust's own lowercase operation about each non-ASCII context scalar;
//! therefore it does not embed Unicode property tables that could drift from
//! the standard library. The writer must begin at byte zero, and it remains at
//! the logical end after a pending-sigma patch.

use std::io::{Seek, SeekFrom, Write};

use anyhow::{anyhow, bail, Result};

pub(crate) const LOWER_OUTPUT_BUFFER_BYTES: usize = 64 * 1024;
const CONTEXT_PROBE_BYTES: usize = 128;
const LOWER_STREAM_METADATA_BYTES: usize = 4096;

/// The bounded reservation needed before the primitive allocates or writes.
pub(crate) const fn lowercase_stream_workspace_bytes() -> usize {
    LOWER_OUTPUT_BUFFER_BYTES + CONTEXT_PROBE_BYTES + LOWER_STREAM_METADATA_BYTES
}

/// Lowercase `input` with the same result as `str::to_lowercase`, without a
/// whole-token output string. `reserve` must accept the fixed workspace before
/// this function allocates its output buffer or writes to `writer`.
pub(crate) fn write_streaming_lowercase<W: Write + Seek>(
    input: &str,
    writer: &mut W,
    mut reserve: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    if writer.stream_position()? != 0 {
        bail!("streaming lowercase writer must start at byte zero");
    }
    reserve(lowercase_stream_workspace_bytes())?;

    let mut output = Output::new(writer);
    let mut preceded_cased = false;
    let mut pending_sigma = None;

    for character in input.chars() {
        if character == 'Σ' {
            resolve_pending_sigma(&mut output, &mut pending_sigma, Context::Cased)?;
            let offset = output.position()?;
            output.push("σ".as_bytes())?;
            pending_sigma = Some(PendingSigma {
                offset,
                preceded_cased,
            });
            preceded_cased = true;
            continue;
        }

        let context = context_of(character);
        resolve_pending_sigma(&mut output, &mut pending_sigma, context)?;
        match context {
            Context::Cased => preceded_cased = true,
            Context::Ignorable => {}
            Context::Break => preceded_cased = false,
        }
        push_lowercase(&mut output, character)?;
    }
    resolve_pending_sigma(&mut output, &mut pending_sigma, Context::Break)?;
    output.finish()
}

fn push_lowercase<W: Write + Seek>(output: &mut Output<'_, W>, character: char) -> Result<()> {
    if character.is_ascii_uppercase() {
        output.push(&[character.to_ascii_lowercase() as u8])
    } else if character.is_ascii_lowercase() {
        output.push(&[character as u8])
    } else {
        for lowered in character.to_lowercase() {
            let mut encoded = [0; 4];
            output.push(lowered.encode_utf8(&mut encoded).as_bytes())?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct PendingSigma {
    offset: u64,
    preceded_cased: bool,
}

fn resolve_pending_sigma<W: Write + Seek>(
    output: &mut Output<'_, W>,
    pending: &mut Option<PendingSigma>,
    following: Context,
) -> Result<()> {
    let Some(sigma) = *pending else {
        return Ok(());
    };
    if following == Context::Ignorable {
        return Ok(());
    }
    *pending = None;
    if sigma.preceded_cased && following == Context::Break {
        output.patch_final_sigma(sigma.offset)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Context {
    Cased,
    Ignorable,
    Break,
}

fn context_of(character: char) -> Context {
    if character.is_ascii_alphabetic() {
        return Context::Cased;
    }
    // Rust determines whether a following scalar leaves sigma final.  The
    // first probe sees the scalar as the entire suffix; the second also has a
    // cased scalar after it. This distinguishes cased, Case_Ignorable, and a
    // non-cased break with the same Unicode data used by `str::to_lowercase`.
    let first_is_final = sigma_is_final_after(character, false);
    if !first_is_final {
        Context::Cased
    } else if !sigma_is_final_after(character, true) {
        Context::Ignorable
    } else {
        Context::Break
    }
}

fn sigma_is_final_after(character: char, trailing_cased: bool) -> bool {
    let mut sample = String::with_capacity(8);
    sample.push('A');
    sample.push('Σ');
    sample.push(character);
    if trailing_cased {
        sample.push('A');
    }
    sample.to_lowercase().chars().nth(1) == Some('ς')
}

struct Output<'a, W> {
    writer: &'a mut W,
    buffer: Vec<u8>,
    flushed: u64,
}

impl<'a, W: Write + Seek> Output<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(LOWER_OUTPUT_BUFFER_BYTES),
            flushed: 0,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() > LOWER_OUTPUT_BUFFER_BYTES {
            return Err(anyhow!("lowercase scalar exceeds output buffer"));
        }
        if self.buffer.len() + bytes.len() > LOWER_OUTPUT_BUFFER_BYTES {
            self.flush()?;
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn position(&self) -> Result<u64> {
        self.flushed
            .checked_add(
                u64::try_from(self.buffer.len())
                    .map_err(|_| anyhow!("lowercase output length exceeds u64"))?,
            )
            .ok_or_else(|| anyhow!("lowercase output length exceeds u64"))
    }

    fn patch_final_sigma(&mut self, offset: u64) -> Result<()> {
        self.flush()?;
        let end = self.flushed;
        if offset.checked_add(2).map_or(true, |finish| finish > end) {
            bail!("pending sigma patch is outside streamed output");
        }
        self.writer.seek(SeekFrom::Start(offset))?;
        self.writer.write_all("ς".as_bytes())?;
        self.writer.seek(SeekFrom::Start(end))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer)?;
            self.flushed = self.position()?;
            self.buffer.clear();
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.flush()?;
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File, OpenOptions};
    use std::io::{Cursor, Read, Seek, SeekFrom};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_NONCE: AtomicU64 = AtomicU64::new(0);

    fn streamed(input: &str) -> String {
        let mut output = Cursor::new(Vec::new());
        write_streaming_lowercase(input, &mut output, |_| Ok(())).unwrap();
        String::from_utf8(output.into_inner()).unwrap()
    }

    #[test]
    fn matches_rust_lowercase_for_contextual_and_expanding_scalars() {
        for input in [
            "UPPER lower İ",
            "Σ",
            "AΣ",
            "AΣA",
            "AΣ'",
            "AΣ9",
            "AΣ\u{200d}A",
            "AΣ\u{0345}A",
            "AΣ\u{0345} ",
            "ΣΟΣ",
            "ΟΣ",
            "İSTANBUL",
        ] {
            assert_eq!(
                streamed(input).as_bytes(),
                input.to_lowercase().as_bytes(),
                "{input:?}"
            );
        }
    }

    #[test]
    fn long_case_ignorable_suffix_crosses_output_buffer_and_patches_final_sigma() {
        let input = format!("AΣ{} ", "\u{0345}".repeat(LOWER_OUTPUT_BUFFER_BYTES));
        assert_eq!(streamed(&input).as_bytes(), input.to_lowercase().as_bytes());
    }

    #[test]
    fn unicode_chunks_match_rust_lowercase_bit_for_bit() {
        let input = ["İ", "Σ", "\u{0345}", "Ａ", "Ж", "\u{200d}", "Σ"]
            .concat()
            .repeat(20_000);
        assert_eq!(streamed(&input).as_bytes(), input.to_lowercase().as_bytes());
    }

    #[test]
    fn reservation_refusal_happens_before_any_write() {
        let mut output = Cursor::new(Vec::new());
        let error =
            write_streaming_lowercase("AΣ", &mut output, |_| anyhow::bail!("refuse")).unwrap_err();
        assert!(error.to_string().contains("refuse"));
        assert!(output.into_inner().is_empty());
    }

    #[test]
    fn rejects_a_writer_that_does_not_start_at_zero() {
        let mut output = Cursor::new(vec![7]);
        output.seek(SeekFrom::End(0)).unwrap();
        assert!(write_streaming_lowercase("A", &mut output, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("byte zero"));
    }

    #[test]
    fn writes_multiple_chunks_to_a_real_file() {
        let path = std::env::temp_dir().join(format!(
            "lumen-unicode-lower-test-{}-{}",
            std::process::id(),
            TEST_NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        let input = format!("AΣ{} ", "\u{0345}".repeat(LOWER_OUTPUT_BUFFER_BYTES));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        write_streaming_lowercase(&input, &mut file, |_| Ok(())).unwrap();
        drop(file);
        let mut output = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut output)
            .unwrap();
        assert_eq!(output.as_bytes(), input.to_lowercase().as_bytes());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reservation_is_fixed_before_the_output_buffer_exists() {
        let mut output = Cursor::new(Vec::new());
        let mut requested = None;
        write_streaming_lowercase("İ", &mut output, |bytes| {
            requested = Some(bytes);
            Ok(())
        })
        .unwrap();
        assert_eq!(requested, Some(lowercase_stream_workspace_bytes()));
    }
}
