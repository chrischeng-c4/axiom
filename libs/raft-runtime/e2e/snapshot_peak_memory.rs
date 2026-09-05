use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use raft_core::Index;
use raft_runtime::{ChunkSink, RaftStateMachine, SNAPSHOT_CHUNK_SIZE};

struct CountingAlloc;

static CURRENT_ALLOC: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALLOC: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let current = CURRENT_ALLOC.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
            PEAK_ALLOC.fetch_max(current, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        CURRENT_ALLOC.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            if new_size > layout.size() {
                let diff = new_size - layout.size();
                let current = CURRENT_ALLOC.fetch_add(diff, Ordering::SeqCst) + diff;
                PEAK_ALLOC.fetch_max(current, Ordering::SeqCst);
            } else {
                let diff = layout.size() - new_size;
                CURRENT_ALLOC.fetch_sub(diff, Ordering::SeqCst);
            }
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

const SMALL_ENTRIES: usize = 1000;
const LARGE_ENTRIES: usize = 50000;

struct MemoryTestSm {
    applied: AtomicU64,
    entries: Mutex<Vec<(u64, Vec<u8>)>>,
}

impl MemoryTestSm {
    fn new() -> Self {
        Self {
            applied: AtomicU64::new(0),
            entries: Mutex::new(Vec::new()),
        }
    }

    fn populate(&self, count: usize) {
        let mut entries = self.entries.lock().unwrap();
        entries.reserve(count);
        let sample_payload = vec![0x42u8; 100];
        for i in 1..=count {
            let idx = i as u64;
            entries.push((idx, sample_payload.clone()));
        }
        self.applied.store(count as u64, Ordering::Release);
    }
}

impl RaftStateMachine for MemoryTestSm {
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
        let mut entries = self.entries.lock().unwrap();
        entries.push((index, command.to_vec()));
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        let entries = self.entries.lock().unwrap();
        let total = entries.len() as u64;
        writer.write_all(&total.to_le_bytes())?;
        for (idx, payload) in entries.iter() {
            writer.write_all(&idx.to_le_bytes())?;
            let len = payload.len() as u32;
            writer.write_all(&len.to_le_bytes())?;
            writer.write_all(payload)?;
        }
        writer.flush()?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        let mut count_buf = [0u8; 8];
        reader.read_exact(&mut count_buf)?;
        let count = u64::from_le_bytes(count_buf);
        let mut restored_entries = Vec::with_capacity(count.min(1024) as usize);
        let mut last_idx = 0;
        let mut u32_buf = [0u8; 4];
        for _ in 0..count {
            reader.read_exact(&mut count_buf)?;
            let idx = u64::from_le_bytes(count_buf);
            reader.read_exact(&mut u32_buf)?;
            let len = u32::from_le_bytes(u32_buf) as usize;
            let mut payload = vec![0u8; len];
            reader.read_exact(&mut payload)?;
            last_idx = idx;
            restored_entries.push((idx, payload));
        }
        *self.entries.lock().unwrap() = restored_entries;
        self.applied.store(last_idx, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

fn measure_snapshot_peak(count: usize) -> (usize, usize) {
    let sm = MemoryTestSm::new();
    sm.populate(count);

    let mut sink = ChunkSink::streaming(SNAPSHOT_CHUNK_SIZE, |_chunk| {});

    PEAK_ALLOC.store(CURRENT_ALLOC.load(Ordering::SeqCst), Ordering::SeqCst);
    let start_alloc = CURRENT_ALLOC.load(Ordering::SeqCst);

    sm.snapshot(&mut sink).expect("snapshot must succeed");

    let peak = PEAK_ALLOC.load(Ordering::SeqCst);
    let peak_delta = peak.saturating_sub(start_alloc);
    (peak_delta, sm.applied_index() as usize)
}

#[test]
fn snapshot_peak_memory_grows_sublinearly() {
    let (small_peak, small_applied) = measure_snapshot_peak(SMALL_ENTRIES);
    let (large_peak, large_applied) = measure_snapshot_peak(LARGE_ENTRIES);

    assert_eq!(small_applied, SMALL_ENTRIES);
    assert_eq!(large_applied, LARGE_ENTRIES);

    assert!(
        large_peak < small_peak.max(SNAPSHOT_CHUNK_SIZE) * 4,
        "Snapshot peak memory assertion failed: small peak {} bytes, large peak {} bytes",
        small_peak,
        large_peak
    );
}

#[test]
fn restore_round_trips_exact_index() {
    let sm = MemoryTestSm::new();
    sm.populate(SMALL_ENTRIES);

    let mut captured_bytes = Vec::new();
    sm.snapshot(&mut captured_bytes)
        .expect("snapshot must succeed");

    let restored_sm = MemoryTestSm::new();
    let mut reader = std::io::Cursor::new(&captured_bytes);
    restored_sm
        .restore(&mut reader)
        .expect("restore must succeed");

    assert_eq!(
        restored_sm.applied_index(),
        SMALL_ENTRIES as u64,
        "restored applied_index must match snapshot applied_index"
    );
}
