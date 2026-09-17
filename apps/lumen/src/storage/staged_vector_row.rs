//! One borrowed fast-WAL vector staged as aligned native-endian f32 mmap data.
//!
//! Borrowed little-endian wire bytes are decoded to a private same-process
//! native mmap file. The backend and checkpoint journal can lend an aligned
//! slice without retaining a full decoded input or checkpoint buffer.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[cfg(test)]
use std::cell::RefCell;

use anyhow::{anyhow, bail, Context, Result};
use memmap2::{Mmap, MmapOptions};

const DECODE_CHUNK_BYTES: usize = 64 * 1024;
const READER_METADATA_BYTES: usize = 4096;
static NONCE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
thread_local! { static LAST_STAGE_DIRECTORY: RefCell<Option<PathBuf>> = const { RefCell::new(None) }; }

#[cfg(test)]
pub(super) fn last_stage_directory_for_test() -> Option<PathBuf> {
    LAST_STAGE_DIRECTORY.with(|last| last.borrow().clone())
}

/// A private file-backed, native-aligned vector row.
#[derive(Clone, Debug)]
pub(crate) struct StagedVectorRow {
    mmap: Arc<Mmap>,
    dim: usize,
    _directory: Arc<OwnedDirectory>,
}

impl StagedVectorRow {
    /// Decode a wire `f32` span in bounded chunks. `payload` is untrusted LE
    /// bytes and may begin at any address. The callback must accept both the
    /// bounded decode buffer and reader metadata before any allocation or file
    /// creation happens.
    pub(crate) fn stage(
        payload: &[u8],
        dim: u32,
        mut reserve: impl FnMut(usize) -> Result<()>,
    ) -> Result<Self> {
        let dim = usize::try_from(dim).map_err(|_| anyhow!("vector dim exceeds usize"))?;
        if dim == 0 {
            bail!("vector dim must be > 0");
        }
        let expected = dim
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or_else(|| anyhow!("vector byte length overflow"))?;
        if payload.len() != expected {
            bail!(
                "vector wire length {} does not match dim {dim}",
                payload.len()
            );
        }
        let scratch = DECODE_CHUNK_BYTES.min(expected);
        let required = scratch
            .checked_add(READER_METADATA_BYTES)
            .ok_or_else(|| anyhow!("vector staging reservation overflow"))?;
        reserve(required)?;
        let directory = Arc::new(OwnedDirectory::create()?);
        let path = directory.path.join("row.f32");
        let mut output =
            File::create_new(&path).with_context(|| format!("create {}", path.display()))?;
        // This is the only decoded heap buffer. File::write_all does not add a
        // caller-owned buffering allocation, so `scratch` prices every
        // simultaneous decoded workspace byte.
        let mut native = Vec::with_capacity(scratch);
        for chunk in payload.chunks(DECODE_CHUNK_BYTES) {
            native.clear();
            for word in chunk.chunks_exact(4) {
                native.extend_from_slice(
                    &f32::from_le_bytes(word.try_into().expect("four bytes")).to_ne_bytes(),
                );
            }
            output.write_all(&native)?;
        }
        output.flush()?;
        output.sync_all()?;
        // SAFETY: the private file is complete and never written after this
        // point; OwnedDirectory keeps its pathname alive through every clone.
        let mmap = unsafe { MmapOptions::new().map(&output) }.context("mmap staged vector")?;
        if mmap.len() != expected {
            bail!("staged vector length changed before mmap");
        }
        bytemuck::try_cast_slice::<u8, f32>(&mmap[..])
            .map_err(|error| anyhow!("staged vector mmap alignment: {error:?}"))?;
        Ok(Self {
            mmap: Arc::new(mmap),
            dim,
            _directory: directory,
        })
    }

    /// Create the exact current scalar-quantized decoded checkpoint view.
    /// The codebook is widened once over the complete row, then each value is
    /// encoded and decoded with the same arithmetic as vector_index::encode_sq
    /// and decode_sq. Only one bounded native-byte chunk is allocated.
    pub(crate) fn stage_sq_canonical(
        &self,
        mut codebook: crate::vector_index::ScalarCodebook,
        mut reserve: impl FnMut(usize) -> Result<()>,
    ) -> Result<(Self, crate::vector_index::ScalarCodebook)> {
        codebook.widen(self.as_f32_slice());
        let scratch = DECODE_CHUNK_BYTES.min(
            self.dim
                .checked_mul(4)
                .ok_or_else(|| anyhow!("vector byte length overflow"))?,
        );
        reserve(
            scratch
                .checked_add(READER_METADATA_BYTES)
                .ok_or_else(|| anyhow!("vector staging reservation overflow"))?,
        )?;
        let directory = Arc::new(OwnedDirectory::create()?);
        let path = directory.path.join("canonical.f32");
        let mut output = File::create_new(&path)?;
        let mut native = Vec::with_capacity(scratch);
        let span = (codebook.max - codebook.min).max(f32::MIN_POSITIVE);
        for values in self.as_f32_slice().chunks((DECODE_CHUNK_BYTES / 4).max(1)) {
            native.clear();
            for &value in values {
                let encoded =
                    (((value - codebook.min) / span).clamp(0.0, 1.0) * 255.0).round() as u8;
                let canonical = codebook.min + (encoded as f32 / 255.0) * span;
                native.extend_from_slice(&canonical.to_ne_bytes());
            }
            output.write_all(&native)?;
        }
        output.sync_all()?;
        let mmap =
            unsafe { MmapOptions::new().map(&output) }.context("mmap canonical staged vector")?;
        bytemuck::try_cast_slice::<u8, f32>(&mmap[..])
            .map_err(|error| anyhow!("canonical staged vector mmap alignment: {error:?}"))?;
        Ok((
            Self {
                mmap: Arc::new(mmap),
                dim: self.dim,
                _directory: directory,
            },
            codebook,
        ))
    }

    pub(crate) fn dim(&self) -> usize {
        self.dim
    }
    pub(crate) const fn retained_metadata_bound() -> usize {
        READER_METADATA_BYTES
    }
    pub(crate) fn as_f32_slice(&self) -> &[f32] {
        // Checked in stage after mmap creation. The mmap is immutable and Arc
        // keeps it alive for the returned borrow.
        bytemuck::try_cast_slice(&self.mmap[..]).expect("validated staged vector mmap")
    }
}

#[derive(Debug)]
struct OwnedDirectory {
    path: PathBuf,
}
impl OwnedDirectory {
    fn create() -> Result<Self> {
        let root = std::env::temp_dir();
        let process = std::process::id();
        for _ in 0..128 {
            let path = root.join(format!(
                "lumen-staged-vector-row-{process}-{}",
                NONCE.fetch_add(1, Ordering::Relaxed)
            ));
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
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| format!("create {}", path.display()))
                }
            }
        }
        bail!("could not allocate staged vector directory")
    }
}
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_index::{decode_sq, encode_sq, ScalarCodebook};

    fn le_words(bits: &[u32]) -> Vec<u8> {
        bits.iter().flat_map(|bits| bits.to_le_bytes()).collect()
    }
    fn unaligned(bits: &[u32]) -> Vec<u8> {
        let mut bytes = vec![0xff];
        bytes.extend(le_words(bits));
        bytes
    }
    fn bits(values: &[f32]) -> Vec<u32> {
        values.iter().map(|value| value.to_bits()).collect()
    }

    #[test]
    fn unaligned_le_input_round_trips_exact_floating_bits() {
        let words = [0x8000_0000, 0x7f80_0000, 0x7fc0_0042, 0x3f80_0000];
        let bytes = unaligned(&words);
        let row = StagedVectorRow::stage(&bytes[1..], words.len() as u32, |_| Ok(())).unwrap();
        assert_eq!(row.dim(), words.len());
        assert_eq!(bits(row.as_f32_slice()), words);
    }
    #[test]
    fn wrong_length_refuses_before_stage_creation() {
        let before = last_stage_directory_for_test();
        assert!(StagedVectorRow::stage(&[0; 7], 2, |_| Ok(()))
            .unwrap_err()
            .to_string()
            .contains("wire length"));
        assert_eq!(last_stage_directory_for_test(), before);
    }
    #[test]
    fn reservation_refusal_creates_no_private_file() {
        let before = last_stage_directory_for_test();
        assert!(
            StagedVectorRow::stage(&le_words(&[0]), 1, |_| anyhow::bail!("refuse"))
                .unwrap_err()
                .to_string()
                .contains("refuse")
        );
        assert_eq!(last_stage_directory_for_test(), before);
    }
    #[test]
    fn clone_keeps_private_file_until_final_owner_drops() {
        let row = StagedVectorRow::stage(&le_words(&[0x3f80_0000]), 1, |_| Ok(())).unwrap();
        let directory = last_stage_directory_for_test().unwrap();
        let clone = row.clone();
        drop(row);
        assert!(directory.is_dir());
        assert_eq!(clone.as_f32_slice()[0].to_bits(), 0x3f80_0000);
        drop(clone);
        assert!(!directory.exists());
    }
    #[test]
    fn reservation_prices_the_only_simultaneous_decode_buffer_and_metadata() {
        let mut required = None;
        let row = StagedVectorRow::stage(&le_words(&[0, 1]), 2, |bytes| {
            required = Some(bytes);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            required,
            Some(DECODE_CHUNK_BYTES.min(8) + READER_METADATA_BYTES)
        );
        assert_eq!(row.as_f32_slice().len(), 2);
    }
    #[cfg(unix)]
    #[test]
    fn private_stage_directory_is_created_with_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let row = StagedVectorRow::stage(&le_words(&[0]), 1, |_| Ok(())).unwrap();
        let directory = last_stage_directory_for_test().unwrap();
        assert_eq!(
            std::fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        drop(row);
    }
    #[test]
    fn sq_canonical_matches_shared_codec_and_preserves_raw_bits() {
        for words in [
            vec![0x8000_0000, 0x7f80_0000, 0xff80_0000, 0x7fc0_0042],
            vec![5f32.to_bits(), 5f32.to_bits(), 5f32.to_bits()],
            vec![(-100f32).to_bits(), 0f32.to_bits(), 100f32.to_bits()],
        ] {
            let input = unaligned(&words);
            let raw = StagedVectorRow::stage(&input[1..], words.len() as u32, |_| Ok(())).unwrap();
            let mut expected_cb = ScalarCodebook::empty(words.len());
            expected_cb.widen(raw.as_f32_slice());
            let expected = decode_sq(&encode_sq(raw.as_f32_slice(), &expected_cb), &expected_cb);
            let (canonical, widened) = raw
                .stage_sq_canonical(ScalarCodebook::empty(words.len()), |_| Ok(()))
                .unwrap();
            assert_eq!(
                (widened.min.to_bits(), widened.max.to_bits(), widened.dim),
                (
                    expected_cb.min.to_bits(),
                    expected_cb.max.to_bits(),
                    expected_cb.dim
                )
            );
            assert_eq!(bits(canonical.as_f32_slice()), bits(&expected));
            assert_eq!(bits(raw.as_f32_slice()), words);
        }
    }
    #[test]
    fn sq_canonical_reserves_bounded_multi_chunk_workspace_before_new_directory() {
        let words = vec![1f32.to_bits(); DECODE_CHUNK_BYTES / 4 + 1];
        let raw =
            StagedVectorRow::stage(&le_words(&words), words.len() as u32, |_| Ok(())).unwrap();
        let mut required = None;
        let before = last_stage_directory_for_test();
        let refusal = raw
            .stage_sq_canonical(ScalarCodebook::empty(words.len()), |bytes| {
                required = Some(bytes);
                anyhow::bail!("refuse")
            })
            .unwrap_err();
        assert!(refusal.to_string().contains("refuse"));
        assert_eq!(required, Some(DECODE_CHUNK_BYTES + READER_METADATA_BYTES));
        assert_eq!(last_stage_directory_for_test(), before);
    }
    #[test]
    fn sq_canonical_widens_the_complete_row_before_chunked_decode() {
        let mut words = vec![5.0f32.to_bits(); DECODE_CHUNK_BYTES / 4];
        words.push(5.5f32.to_bits());
        let raw =
            StagedVectorRow::stage(&le_words(&words), words.len() as u32, |_| Ok(())).unwrap();

        let mut expected_codebook = ScalarCodebook::empty(words.len());
        expected_codebook.widen(raw.as_f32_slice());
        let expected = decode_sq(
            &encode_sq(raw.as_f32_slice(), &expected_codebook),
            &expected_codebook,
        );

        let (canonical, widened) = raw
            .stage_sq_canonical(ScalarCodebook::empty(words.len()), |_| Ok(()))
            .unwrap();

        assert_eq!(widened.min.to_bits(), 5.0f32.to_bits());
        assert_eq!(widened.max.to_bits(), 5.5f32.to_bits());
        assert_eq!(bits(canonical.as_f32_slice()), bits(&expected));
    }
}
