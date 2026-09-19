//! Exact HNSW fallback must not copy rejected vectors or every candidate ID.
//! Count allocations without imposing a machine-specific wall-clock threshold.
use lumen::types::{VectorBackend, VectorMetric, VectorQuantize, VectorSpec};
use lumen::vector_index::{FlatCpuIndex, HnswCpuIndex, VectorIndex};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    static ALLOCATIONS: Cell<Option<usize>> = const { Cell::new(None) };
}
struct CountingAllocator;
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn record_allocation(pointer: *mut u8) {
    if !pointer.is_null() {
        let _ = ALLOCATIONS.try_with(|state| {
            if let Some(count) = state.get() {
                state.set(Some(count + 1));
            }
        });
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(pointer);
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(pointer);
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let pointer = unsafe { System.realloc(pointer, layout, size) };
        record_allocation(pointer);
        pointer
    }
}

fn measure<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Restore(Option<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ALLOCATIONS.with(|state| state.set(self.0));
        }
    }
    let restore = Restore(ALLOCATIONS.with(|state| state.replace(Some(0))));
    let result = f();
    let count = ALLOCATIONS.with(|state| state.get().unwrap());
    drop(restore);
    (result, count)
}

fn check_filtered_scan(quantize: Option<VectorQuantize>) {
    const N: usize = 1024;
    const K: usize = 10;
    // Room for graph traversal, score storage, and returned hits, but not a
    // corpus-sized sequence of ID/vector copies. The old path exceeds 2*N.
    const MAX_ALLOCATIONS: usize = 512;
    for metric in [VectorMetric::L2, VectorMetric::Cosine, VectorMetric::Dot] {
        // DistDot's graph backend requires dot products no greater than one.
        // Keep this allocation fixture inside that existing precondition.
        let scale = if metric == VectorMetric::Dot {
            0.25
        } else {
            1.0
        };
        let spec = VectorSpec {
            dim: 8,
            metric,
            backend: VectorBackend::HnswCpu,
            quantize,
        };
        let index = HnswCpuIndex::new(spec);
        index.set_ef_search(32);
        for number in 0..N {
            let vector: Vec<_> = (0..8)
                .map(|dim| ((number * (dim + 3)) % 997 + 1) as f32 / 997.0 * scale)
                .collect();
            index.add(&format!("v{number:04}"), &vector).unwrap();
        }
        index.add("v1022", &[0.25 * scale; 8]).unwrap();
        index.remove("v0000").unwrap();
        let query = [0.5 * scale; 8];
        let reference = FlatCpuIndex::new(VectorSpec {
            backend: VectorBackend::FlatCpu,
            quantize: None,
            ..spec
        });
        // SQ snapshots expose the current decoded values. Use those values
        // in an independent exact backend, not the approximate graph scores.
        for (id, vector) in index.dump_for_snapshot().unwrap().0 {
            reference.add(&id, &vector).unwrap();
        }
        for allow_some in [false, true] {
            let allow = |id: &str| allow_some && matches!(id, "v1022" | "v1023");
            let expected = reference.search_knn_filtered(&query, K, &allow).unwrap();
            let before = index.exact_scan_fallbacks();
            let (actual, allocations) =
                measure(|| index.search_knn_filtered(&query, K, &allow).unwrap());
            assert_eq!(index.exact_scan_fallbacks(), before + 1);
            assert_eq!(actual, expected, "metric={metric:?}, quantize={quantize:?}");
            assert!(allocations <= MAX_ALLOCATIONS,
                "exact scan copied rejected or non-result values: {allocations} allocations, metric={metric:?}, quantize={quantize:?}, allow_some={allow_some}");
        }
    }
}

#[test]
fn raw_exact_fallback_borrows_rejected_and_candidate_values() {
    check_filtered_scan(None);
}

#[test]
fn sq_exact_fallback_decodes_only_allowed_values() {
    check_filtered_scan(Some(VectorQuantize::Sq));
}
