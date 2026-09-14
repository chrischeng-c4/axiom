//! Private file-backed storage for Jieba's no-HMM dynamic-programming route.
//! Two fixed pages hold the most recently used slots. Neither the number of
//! input scalars nor the dictionary edge count changes the resident cache size.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const SLOT_BYTES: usize = 16;
const PAGE_BYTES: usize = 4096;
const SLOTS_PER_PAGE: usize = PAGE_BYTES / SLOT_BYTES;
pub(crate) const ROUTE_CACHE_BYTES: usize = 2 * PAGE_BYTES;

struct Page {
    number: Option<usize>,
    dirty: bool,
    bytes: Box<[u8; PAGE_BYTES]>,
}
impl Page {
    fn new() -> Self {
        Self {
            number: None,
            dirty: false,
            bytes: Box::new([0; PAGE_BYTES]),
        }
    }
}

pub(crate) struct DiskRoute {
    file: File,
    path: PathBuf,
    slots: usize,
    pages: [Page; 2],
    victim: usize,
}

impl DiskRoute {
    /// `directory` is the staging owner's private directory. The route never
    /// accepts a caller-selected file name, and never replaces an existing file.
    pub(crate) fn create(directory: &Path) -> io::Result<Self> {
        let path = directory.join("jieba-route.tmp");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        Ok(Self {
            file,
            path,
            slots: 0,
            pages: [Page::new(), Page::new()],
            victim: 0,
        })
    }

    fn checked_offset(slots: usize) -> io::Result<u64> {
        slots
            .checked_mul(SLOT_BYTES)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Jieba route byte offset overflows",
                )
            })
    }

    fn page_length(&self, number: usize) -> usize {
        (self.slots - number * SLOTS_PER_PAGE).min(SLOTS_PER_PAGE) * SLOT_BYTES
    }

    fn page(&mut self, index: usize) -> io::Result<(usize, usize)> {
        if index >= self.slots {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Jieba route slot is out of bounds",
            ));
        }
        let number = index / SLOTS_PER_PAGE;
        let selected = match self
            .pages
            .iter()
            .position(|page| page.number == Some(number))
        {
            Some(selected) => selected,
            None => {
                let selected = self
                    .pages
                    .iter()
                    .position(|page| page.number.is_none())
                    .unwrap_or(self.victim);
                if let Some(previous) = self.pages[selected]
                    .number
                    .filter(|_| self.pages[selected].dirty)
                {
                    let length = self.page_length(previous);
                    self.file.seek(SeekFrom::Start(Self::checked_offset(
                        previous * SLOTS_PER_PAGE,
                    )?))?;
                    self.file.write_all(&self.pages[selected].bytes[..length])?;
                }
                // A failed read cannot leave a partly filled page addressable.
                self.pages[selected].number = None;
                self.pages[selected].dirty = false;
                let length = self.page_length(number);
                self.file.seek(SeekFrom::Start(Self::checked_offset(
                    number * SLOTS_PER_PAGE,
                )?))?;
                self.file
                    .read_exact(&mut self.pages[selected].bytes[..length])?;
                self.pages[selected].number = Some(number);
                selected
            }
        };
        self.victim = 1 - selected;
        Ok((selected, index % SLOTS_PER_PAGE * SLOT_BYTES))
    }
}

impl jieba_rs::RouteStore for DiskRoute {
    fn reset(&mut self, slots: usize) -> io::Result<()> {
        let bytes = Self::checked_offset(slots)?;
        self.slots = 0;
        for page in &mut self.pages {
            page.number = None;
            page.dirty = false;
        }
        // Truncate before extending even when the next block has the same
        // length. Its terminal score and every unread slot must start at zero.
        self.file.set_len(0)?;
        self.file.set_len(bytes)?;
        self.slots = slots;
        self.victim = 0;
        Ok(())
    }

    fn get(&mut self, index: usize) -> io::Result<(f64, usize)> {
        let (page, offset) = self.page(index)?;
        let bytes = &self.pages[page].bytes[offset..offset + SLOT_BYTES];
        let score = f64::from_bits(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
        let next =
            usize::try_from(u64::from_le_bytes(bytes[8..].try_into().unwrap())).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Jieba route target exceeds usize",
                )
            })?;
        Ok((score, next))
    }

    fn set(&mut self, index: usize, value: (f64, usize)) -> io::Result<()> {
        let next = u64::try_from(value.1).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Jieba route target exceeds u64",
            )
        })?;
        let (page, offset) = self.page(index)?;
        let bytes = &mut self.pages[page].bytes[offset..offset + SLOT_BYTES];
        bytes[..8].copy_from_slice(&value.0.to_bits().to_le_bytes());
        bytes[8..].copy_from_slice(&next.to_le_bytes());
        self.pages[page].dirty = true;
        Ok(())
    }
}

impl Drop for DiskRoute {
    fn drop(&mut self) {
        // It is disposable computation state. Only the final staged Text file
        // becomes durable; cached route writes need no final flush.
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jieba_rs::RouteStore;

    #[test]
    fn reverse_writes_and_distant_reads_preserve_exact_slots_with_two_pages() {
        let directory = tempfile::tempdir().unwrap();
        let mut route = DiskRoute::create(directory.path()).unwrap();
        route.reset(6001).unwrap();
        for index in (0..6001).rev() {
            route
                .set(index, (-(index as f64) * 0.25, index + 1))
                .unwrap();
        }
        for step in 0..6001 {
            let index = step * 29 % 6001;
            let (score, next) = route.get(index).unwrap();
            assert_eq!(score.to_bits(), (-(index as f64) * 0.25).to_bits());
            assert_eq!(next, index + 1);
        }
        assert_eq!(
            route.file.metadata().unwrap().len(),
            6001 * SLOT_BYTES as u64
        );
        assert_eq!(
            route
                .pages
                .iter()
                .map(|page| page.bytes.len())
                .sum::<usize>(),
            ROUTE_CACHE_BYTES
        );
        let path = route.path.clone();
        drop(route);
        assert!(!path.exists());
    }

    #[test]
    fn reset_discards_old_terminal_and_cached_scores_even_at_the_same_size() {
        let directory = tempfile::tempdir().unwrap();
        let mut route = DiskRoute::create(directory.path()).unwrap();
        route.reset(1025).unwrap();
        for index in [0, 255, 256, 1024] {
            route.set(index, (f64::INFINITY, usize::MAX)).unwrap();
        }
        route.reset(1025).unwrap();
        for index in [0, 255, 256, 1024] {
            assert_eq!(route.get(index).unwrap(), (0.0, 0));
        }
        assert!(route.get(1025).is_err());
        assert!(route.set(1025, (0.0, 0)).is_err());
        assert!(route.reset(usize::MAX).is_err());
        route.reset(0).unwrap();
        assert!(route.get(0).is_err());
        assert_eq!(route.file.metadata().unwrap().len(), 0);
    }
}
