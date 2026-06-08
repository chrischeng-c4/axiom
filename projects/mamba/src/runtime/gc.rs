/// Cycle-detecting garbage collector for the Mamba runtime (#315).
/// Thread-local version — one GC state per thread, no cross-thread coordination.
///
/// Architecture:
/// - Container objects (list, dict, instance) are tracked in a thread-local set
/// - When allocation count exceeds a threshold, a GC cycle runs
/// - Mark phase: starting from roots (stack/globals), mark reachable objects
/// - Sweep phase: reclaim unmarked tracked objects (breaking cycles)
/// - Per-thread isolation: each test thread has its own independent GC state

use std::sync::atomic::Ordering;
use rustc_hash::{FxHashMap, FxHashSet};
use super::rc::MbObject;
use super::value::MbValue;

/// GC configuration and state.
struct GcState {
    /// All tracked container objects (may form cycles).
    /// FxHashSet (mitigation 2A for #2100): pointers are process-internal,
    /// so SipHash's HashDoS resistance is wasted overhead.
    tracked: FxHashSet<usize>,
    /// Number of allocations since last collection.
    alloc_count: usize,
    /// Threshold: trigger collection when alloc_count exceeds this.
    /// Raised 700 → 10 000 (mitigation 1A for #2100) — fewer collect passes
    /// in alloc-heavy hot loops; short-lived containers die via refcount
    /// before GC sees them.
    threshold: usize,
    /// Total number of collections performed.
    collections: u64,
    /// Whether GC is currently running (prevents re-entrant collection).
    collecting: bool,
    /// Whether automatic collection is enabled.
    enabled: bool,
    /// Root set: global variables and stack references kept alive externally.
    roots: Vec<MbValue>,
}

impl GcState {
    fn new() -> Self {
        Self {
            tracked: FxHashSet::default(),
            alloc_count: 0,
            threshold: 10_000,
            collections: 0,
            collecting: false,
            // Resolved by #1129 — JIT codegen now emits mb_retain_value/mb_release_value.
            // Closure ownership symmetry fixed. GC re-enabled for cycle collection only.
            // @spec .aw/changes/mamba-refcount/groups/jit-refcount-enable/specs/gc-reenable.md#R1
            enabled: true,
            roots: Vec::new(),
        }
    }
}

thread_local! {
    static GC: std::cell::RefCell<GcState> = std::cell::RefCell::new(GcState::new());
}

// ── Safepoint Protocol (no-op stubs — GC is now per-thread) ──
//
// These functions are kept as public no-ops because many call sites in
// async_rt, tokio_exec, class, async_task, and iter still call them.
// With per-thread GC there is no stop-the-world needed.

/// Register the calling thread as a mutator (no-op with per-thread GC).
pub fn gc_register_thread() {
    // Per-thread GC: no global thread registry needed.
}

/// Unregister the calling thread (no-op with per-thread GC).
pub fn gc_unregister_thread() {
    // Per-thread GC: no global thread registry needed.
}

/// Safepoint poll — no-op with per-thread GC.
/// Kept for call-site compatibility with async_rt, class, iter, etc.
#[inline]
pub fn gc_safepoint() {
    // Per-thread GC: no cross-thread coordination needed.
}

// ── Object Tracking ──

/// Track a container object for cycle detection.
/// Called automatically when creating lists, dicts, and instances.
pub fn gc_track(obj: *mut MbObject) {
    if obj.is_null() {
        return;
    }
    let addr = obj as usize;
    let should_collect = GC.with(|gc| {
        let mut gc = gc.borrow_mut();
        gc.tracked.insert(addr);
        gc.alloc_count += 1;
        gc.enabled && !gc.collecting && gc.alloc_count >= gc.threshold
    });
    if should_collect {
        collect();
    }
}

/// Untrack an object (called when it's freed by refcount reaching zero).
pub fn gc_untrack(obj: *mut MbObject) {
    if obj.is_null() {
        return;
    }
    let addr = obj as usize;
    GC.with(|gc| {
        gc.borrow_mut().tracked.remove(&addr);
    });
}

// ── Root Management ──

/// Add a value to the GC root set (prevents collection).
pub fn gc_add_root(val: MbValue) {
    GC.with(|gc| {
        gc.borrow_mut().roots.push(val);
    });
}

/// Remove a value from the GC root set.
pub fn gc_remove_root(val: MbValue) {
    GC.with(|gc| {
        let mut gc = gc.borrow_mut();
        if let Some(pos) = gc.roots.iter().position(|v| v.to_bits() == val.to_bits()) {
            gc.roots.swap_remove(pos);
        }
    });
}

/// Clear all roots (for testing/shutdown).
pub fn gc_clear_roots() {
    GC.with(|gc| {
        gc.borrow_mut().roots.clear();
    });
}

/// Clear all GC tracking state without running a collection.
///
/// This MUST be called during runtime cleanup (between test executions)
/// to prevent use-after-free from stale GC entries. Without this, the
/// thread-local GC tracked set accumulates container pointers across tests.
/// When the JIT backend drops and frees compile-time objects (immortal
/// strings/bytes), those containers still reference the freed memory.
/// A subsequent GC sweep would read freed memory in
/// `release_contained_values` → heap corruption → SIGABRT.
///
/// We deliberately do NOT run collect() here because module_to_value()
/// creates dicts with borrowed (un-retained) copies of module attr
/// MbValues, producing shared references with rc=1. A GC sweep would
/// double-free those strings via cascading release_contained_values.
/// Instead, we simply discard the tracked set — the leaked container
/// objects are reclaimed when the process exits.
pub fn gc_clear_all_state() {
    GC.with(|gc| {
        if let Ok(mut gc) = gc.try_borrow_mut() {
            gc.tracked.clear();
            gc.roots.clear();
            gc.alloc_count = 0;
            gc.collections = 0;
            gc.collecting = false;
        }
    });
}

// ── Collection ──

/// Run a garbage collection cycle. Returns the number of objects collected.
///
/// Uses CPython-style trial deletion: refcounts identify externally-referenced
/// objects, no stack scanning needed. Per-thread GC eliminates the need for
/// stop-the-world coordination.
pub fn collect() -> usize {
    // Check and set collecting flag, snapshot tracked + roots atomically.
    let (tracked_snapshot, root_ptrs) = {
        let result = GC.with(|gc| {
            let mut gc = gc.borrow_mut();
            if gc.collecting {
                return None;
            }
            gc.collecting = true;
            let tracked: Vec<usize> = gc.tracked.iter().copied().collect();
            let roots: Vec<*mut MbObject> =
                gc.roots.iter().filter_map(|v| v.as_ptr()).collect();
            Some((tracked, roots))
        });
        match result {
            None => return 0,
            Some(pair) => pair,
        }
    };

    // === Trial Deletion Algorithm (CPython-style) ===

    // Phase 1: Initialize gc_refs = refcount for each tracked object.
    // Immortal objects are skipped — they can never be garbage.
    // FxHashMap (mitigation 2A for #2100): pointer keys → SipHash is wasted.
    let mut gc_refs: FxHashMap<usize, i64> =
        FxHashMap::with_capacity_and_hasher(tracked_snapshot.len(), Default::default());
    for &addr in &tracked_snapshot {
        let obj = addr as *mut MbObject;
        let rc = unsafe { (*obj).header.rc.load(Ordering::Relaxed) };
        if rc == super::rc::IMMORTAL_REFCOUNT {
            continue;
        }
        gc_refs.insert(addr, rc as i64);
    }

    // Phase 2: Subtract internal references.
    // For each tracked container, visit children. If a child is also tracked,
    // decrement its gc_refs. After this pass, gc_refs > 0 means the object
    // has external references (from JIT locals, globals, non-tracked objects).
    let addrs: Vec<usize> = gc_refs.keys().copied().collect();
    for addr in &addrs {
        let obj = *addr as *mut MbObject;
        unsafe {
            visit_contained(obj, |child| {
                let child_addr = child as usize;
                if let Some(refs) = gc_refs.get_mut(&child_addr) {
                    *refs -= 1;
                }
            });
        }
    }

    // Phase 3: Identify GC roots — objects with gc_refs > 0 have external
    // references and must survive. Also include explicit gc.roots.
    let mut marked: FxHashSet<usize> = FxHashSet::default();
    for (&addr, &refs) in &gc_refs {
        if refs > 0 {
            mark_object(addr as *mut MbObject, &mut marked);
        }
    }
    for root_ptr in &root_ptrs {
        mark_object(*root_ptr, &mut marked);
    }

    // Phase 4: Sweep — unmarked tracked objects are cyclic garbage.
    //
    // Split into two steps to avoid re-entrant borrow: release_contained_values →
    // mb_release → gc_untrack all call GC.with(), so we must NOT hold the borrow
    // while freeing objects.
    let to_free: Vec<usize> = GC.with(|gc| {
        let mut gc = gc.borrow_mut();
        let to_sweep: Vec<usize> = gc_refs.keys()
            .filter(|addr| !marked.contains(addr))
            .copied()
            .collect();

        let mut confirmed = Vec::with_capacity(to_sweep.len());
        for addr in to_sweep {
            if gc.tracked.remove(&addr) {
                let obj = addr as *mut MbObject;
                unsafe {
                    // Mark immortal to prevent re-entrant release from cycles.
                    (*obj).header.rc.store(super::rc::IMMORTAL_REFCOUNT, Ordering::Relaxed);
                }
                confirmed.push(addr);
            }
        }
        confirmed
    });

    // Two-pass free for cyclic structures:
    //   Pass 1 — release contained values. All to-free objects are marked
    //            IMMORTAL, so cascading mb_release on a peer is a no-op.
    //   Pass 2 — drop (deallocate) the objects.
    let collected = to_free.len();
    for addr in &to_free {
        let obj = *addr as *mut MbObject;
        unsafe {
            super::rc::release_contained_values_pub(obj);
        }
    }
    for addr in &to_free {
        let obj = *addr as *mut MbObject;
        unsafe {
            drop(Box::from_raw(obj));
        }
    }

    // Bookkeeping — re-acquire the borrow.
    GC.with(|gc| {
        let mut gc = gc.borrow_mut();
        gc.alloc_count = 0;
        gc.collections += 1;
        gc.collecting = false;
    });

    collected
}

/// Visit all contained MbValue references in a tracked object.
/// Calls `visitor` for each child that is a heap pointer.
/// Used by trial deletion (subtract phase) and mark phase.
unsafe fn visit_contained(obj: *mut MbObject, mut visitor: impl FnMut(*mut MbObject)) {
    use super::rc::ObjData;
    match &(*obj).data {
        ObjData::List(lock) => {
            let items = lock.read().unwrap();
            for item in items.iter() {
                if let Some(ptr) = item.as_ptr() { visitor(ptr); }
            }
        }
        ObjData::Dict(lock) => {
            let map = lock.read().unwrap();
            for val in map.values() {
                if let Some(ptr) = val.as_ptr() { visitor(ptr); }
            }
        }
        ObjData::Tuple(items) => {
            for item in items {
                if let Some(ptr) = item.as_ptr() { visitor(ptr); }
            }
        }
        ObjData::Instance { fields, .. } => {
            let fields = fields.read().unwrap();
            for val in fields.values() {
                if let Some(ptr) = val.as_ptr() { visitor(ptr); }
            }
        }
        ObjData::Set(lock) => {
            let items = lock.read().unwrap();
            for item in items.iter() {
                if let Some(ptr) = item.as_ptr() { visitor(ptr); }
            }
        }
        ObjData::FrozenSet(items) => {
            for item in items {
                if let Some(ptr) = item.as_ptr() { visitor(ptr); }
            }
        }
        _ => {} // Non-container types (Str, Bytes, BigInt, etc.) have no children
    }
}

/// Mark an object and its references as reachable.
/// Takes `marked` set as parameter to avoid per-object GC borrow.
fn mark_object(obj: *mut MbObject, marked: &mut FxHashSet<usize>) {
    if obj.is_null() {
        return;
    }
    let addr = obj as usize;
    if !marked.insert(addr) {
        return; // already marked
    }
    unsafe {
        visit_contained(obj, |child| {
            mark_object(child, marked);
        });
    }
}

// ── GC Control (Python gc module equivalent) ──

/// Enable automatic garbage collection.
pub fn gc_enable() {
    GC.with(|gc| { gc.borrow_mut().enabled = true; });
}

/// Disable automatic garbage collection.
pub fn gc_disable() {
    GC.with(|gc| { gc.borrow_mut().enabled = false; });
}

/// Check if GC is enabled.
pub fn gc_is_enabled() -> bool {
    GC.with(|gc| gc.borrow().enabled)
}

/// Set the collection threshold.
pub fn gc_set_threshold(threshold: usize) {
    GC.with(|gc| { gc.borrow_mut().threshold = threshold; });
}

/// Get the collection threshold.
pub fn gc_get_threshold() -> usize {
    GC.with(|gc| gc.borrow().threshold)
}

/// Get the number of tracked objects.
pub fn gc_get_count() -> usize {
    GC.with(|gc| gc.borrow().tracked.len())
}

/// Get total number of collections performed.
pub fn gc_get_stats() -> (u64, usize, usize) {
    GC.with(|gc| {
        let gc = gc.borrow();
        (gc.collections, gc.tracked.len(), gc.threshold)
    })
}

/// Diagnostic — full state snapshot for #1274 investigation.
/// Returns (collections, tracked.len(), threshold, alloc_count, enabled, collecting).
pub fn gc_get_full_stats() -> (u64, usize, usize, usize, bool, bool) {
    GC.with(|gc| {
        let gc = gc.borrow();
        (gc.collections, gc.tracked.len(), gc.threshold, gc.alloc_count, gc.enabled, gc.collecting)
    })
}

// ── Runtime-callable wrappers (MbValue ABI) ──

/// gc.collect() -> number of unreachable objects freed
pub fn mb_gc_collect(_: MbValue) -> MbValue {
    let freed = collect();
    MbValue::from_int(freed as i64)
}

/// gc.enable()
pub fn mb_gc_enable() {
    gc_enable();
}

/// gc.disable()
pub fn mb_gc_disable() {
    gc_disable();
}

/// gc.isenabled() -> bool
pub fn mb_gc_isenabled() -> MbValue {
    MbValue::from_bool(gc_is_enabled())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rc::{MbObject, ObjData};

    fn reset_gc_for_test() {
        GC.with(|gc| {
            let mut gc = gc.borrow_mut();
            gc.tracked.clear();
            gc.alloc_count = 0;
            gc.collecting = false;
            gc.roots.clear();
            gc.enabled = false; // suppress auto-collect; manual collect() ignores this flag
        });
    }

    /// Simulate JIT releasing its local reference: rc=1 → rc=0 so trial deletion
    /// sees gc_refs=0 for objects with no external references.
    unsafe fn simulate_jit_release(obj: *mut MbObject) {
        (*obj).header.rc.fetch_sub(1, Ordering::Relaxed);
    }

    #[test]
    fn test_track_and_collect_unreachable() {
        reset_gc_for_test();

        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_list(vec![]);

        // Create the cycle: a -> b and b -> a
        unsafe {
            if let ObjData::List(ref lock) = (*a).data {
                lock.write().unwrap().push(MbValue::from_ptr(b));
            }
            if let ObjData::List(ref lock) = (*b).data {
                lock.write().unwrap().push(MbValue::from_ptr(a));
            }
        }

        // Neither is in roots, so both should be collected
        let freed = collect();
        gc_enable();
        assert!(freed >= 2, "should collect at least 2 cyclic objects, got {freed}");
    }

    #[test]
    fn test_reachable_not_collected() {
        reset_gc_for_test();

        let obj = MbObject::new_list(vec![MbValue::from_int(42)]);
        let val = MbValue::from_ptr(obj);
        gc_add_root(val);

        let freed = collect();
        assert_eq!(freed, 0, "rooted object should not be collected");

        gc_remove_root(val);
        gc_untrack(obj);
        gc_enable();
        unsafe { drop(Box::from_raw(obj)); }
    }

    #[test]
    fn test_nested_reachability() {
        reset_gc_for_test();

        let inner = MbObject::new_list(vec![MbValue::from_int(1)]);
        let outer = MbObject::new_list(vec![MbValue::from_ptr(inner)]);
        let root = MbValue::from_ptr(outer);
        gc_add_root(root);

        let freed = collect();
        assert_eq!(freed, 0, "both outer and inner should be reachable");

        gc_remove_root(root);
        // Simulate JIT releasing locals — no external refs remain
        unsafe {
            simulate_jit_release(outer);
            simulate_jit_release(inner);
        }
        let freed = collect();
        gc_enable();
        assert!(freed >= 2, "both should be collected when root removed");
    }

    #[test]
    fn test_gc_enable_disable() {
        gc_enable();
        assert!(gc_is_enabled());
        gc_disable();
        assert!(!gc_is_enabled());
        gc_enable();
        assert!(gc_is_enabled());
    }

    #[test]
    fn test_gc_threshold() {
        gc_set_threshold(100);
        assert_eq!(gc_get_threshold(), 100);
        gc_set_threshold(700);
    }

    #[test]
    fn test_gc_stats() {
        let (_, _, threshold) = gc_get_stats();
        assert!(threshold > 0);
    }

    // ── Additional tests ──

    #[test]
    fn test_self_referential_cycle() {
        reset_gc_for_test();
        let a = MbObject::new_list(vec![]);
        let val_a = MbValue::from_ptr(a);
        unsafe {
            if let ObjData::List(ref lock) = (*a).data {
                lock.write().unwrap().push(val_a);
            }
        }
        let freed = collect();
        assert!(freed >= 1, "self-referential list should be collected, freed={freed}");
    }

    #[test]
    fn test_long_reference_chain_collected() {
        reset_gc_for_test();
        const N: usize = 10;
        let nodes: Vec<*mut MbObject> = (0..N).map(|_| MbObject::new_list(vec![])).collect();
        for i in 0..N - 1 {
            let next_val = MbValue::from_ptr(nodes[i + 1]);
            unsafe {
                if let ObjData::List(ref lock) = (*nodes[i]).data {
                    lock.write().unwrap().push(next_val);
                }
            }
        }
        unsafe { for &node in &nodes { simulate_jit_release(node); } }
        let freed = collect();
        assert_eq!(freed, N, "all {N} chain nodes should be collected, freed={freed}");
    }

    #[test]
    fn test_track_untrack_count() {
        reset_gc_for_test();
        let before = gc_get_count();
        let obj = MbObject::new_list(vec![]);
        assert_eq!(gc_get_count(), before + 1, "count should increase after track");
        gc_untrack(obj);
        assert_eq!(gc_get_count(), before, "count should decrease after untrack");
        unsafe { drop(Box::from_raw(obj)); }
    }

    #[test]
    fn test_add_root_prevents_collection() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![MbValue::from_int(7)]);
        let val = MbValue::from_ptr(obj);
        gc_add_root(val);
        assert_eq!(collect(), 0, "rooted object must not be collected");
        gc_remove_root(val);
        gc_untrack(obj);
        unsafe { drop(Box::from_raw(obj)); }
    }

    #[test]
    fn test_remove_root_allows_collection() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![]);
        let val = MbValue::from_ptr(obj);
        gc_add_root(val);
        assert_eq!(collect(), 0, "should not collect while rooted");
        gc_remove_root(val);
        unsafe { simulate_jit_release(obj); }
        assert_eq!(collect(), 1, "should collect after root removed");
    }

    #[test]
    fn test_clear_roots() {
        reset_gc_for_test();
        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_list(vec![]);
        gc_add_root(MbValue::from_ptr(a));
        gc_add_root(MbValue::from_ptr(b));
        assert_eq!(collect(), 0, "both rooted, nothing collected");
        gc_clear_roots();
        unsafe { simulate_jit_release(a); simulate_jit_release(b); }
        assert_eq!(collect(), 2, "both freed after clear_roots");
    }

    #[test]
    fn test_multiple_collect_idempotent() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![]);
        unsafe { simulate_jit_release(obj); }
        assert_eq!(collect(), 1);
        assert_eq!(collect(), 0);
        assert_eq!(collect(), 0);
    }

    #[test]
    fn test_empty_collect() {
        reset_gc_for_test();
        assert_eq!(collect(), 0, "collect with no tracked objects should return 0");
    }

    #[test]
    fn test_gc_get_count_accurate() {
        reset_gc_for_test();
        assert_eq!(gc_get_count(), 0);
        let _a = MbObject::new_list(vec![]);
        assert_eq!(gc_get_count(), 1);
        let _b = MbObject::new_dict();
        assert_eq!(gc_get_count(), 2);
        unsafe { simulate_jit_release(_a); simulate_jit_release(_b); }
        assert_eq!(collect(), 2);
        assert_eq!(gc_get_count(), 0);
    }

    #[test]
    fn test_gc_get_stats_triple() {
        let (_, _, threshold) = gc_get_stats();
        assert!(threshold > 0, "threshold must be positive");
    }

    #[test]
    fn test_mb_gc_isenabled_returns_bool() {
        gc_enable();
        let v = mb_gc_isenabled();
        assert!(v.is_bool());
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_mb_gc_enable_disable_toggle() {
        mb_gc_disable();
        assert!(!gc_is_enabled());
        assert_eq!(mb_gc_isenabled().as_bool(), Some(false));
        mb_gc_enable();
        assert!(gc_is_enabled());
        assert_eq!(mb_gc_isenabled().as_bool(), Some(true));
    }

    #[test]
    fn test_mb_gc_collect_returns_int() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![]);
        unsafe { simulate_jit_release(obj); }
        let result = mb_gc_collect(MbValue::none());
        assert!(result.is_int());
        assert_eq!(result.as_int(), Some(1));
    }

    #[test]
    fn test_gc_disable_prevents_auto_collect() {
        reset_gc_for_test();
        gc_disable();
        gc_set_threshold(1);
        let mut ptrs = Vec::new();
        for _ in 0..5 { ptrs.push(MbObject::new_list(vec![])); }
        assert_eq!(gc_get_count(), 5, "auto-collect must not run while GC is disabled");
        unsafe { for &p in &ptrs { simulate_jit_release(p); } }
        assert_eq!(collect(), 5);
        gc_enable();
        gc_set_threshold(700);
    }

    #[test]
    fn test_threshold_auto_triggers_cycle_collection() {
        reset_gc_for_test();
        gc_set_threshold(2);
        gc_enable();
        let a = MbObject::new_list(vec![]);
        unsafe {
            if let ObjData::List(ref lock) = (*a).data {
                lock.write().unwrap().push(MbValue::from_ptr(a));
            }
        }
        let _b = MbObject::new_list(vec![]);
        // a (cycle) swept; _b survives (gc_refs=1 → external ref)
        assert_eq!(gc_get_count(), 1, "cycle swept, _b survives");
        gc_disable();
        gc_set_threshold(700);
    }

    #[test]
    fn test_dict_object_collected() {
        reset_gc_for_test();
        let d = MbObject::new_dict();
        unsafe { simulate_jit_release(d); }
        assert_eq!(collect(), 1, "unreachable dict should be collected");
    }

    #[test]
    fn test_instance_object_collected() {
        reset_gc_for_test();
        let inst = MbObject::new_instance("MyClass".to_string());
        unsafe { simulate_jit_release(inst); }
        assert_eq!(collect(), 1, "unreachable instance should be collected");
    }

    #[test]
    fn test_multiple_roots_all_reachable() {
        reset_gc_for_test();
        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_list(vec![]);
        let c = MbObject::new_list(vec![]);
        gc_add_root(MbValue::from_ptr(a));
        gc_add_root(MbValue::from_ptr(b));
        gc_add_root(MbValue::from_ptr(c));
        assert_eq!(collect(), 0, "all three rooted, nothing collected");
        gc_clear_roots();
        unsafe { simulate_jit_release(a); simulate_jit_release(b); simulate_jit_release(c); }
        assert_eq!(collect(), 3, "all freed after roots cleared");
    }

    #[test]
    fn test_int_root_ignored_safely() {
        reset_gc_for_test();
        gc_add_root(MbValue::from_int(42));
        let obj = MbObject::new_list(vec![]);
        unsafe { simulate_jit_release(obj); }
        assert_eq!(collect(), 1, "unrooted list collected; int root is a no-op");
        gc_clear_roots();
    }

    #[test]
    fn test_track_null_pointer() {
        gc_track(std::ptr::null_mut());
    }

    #[test]
    fn test_untrack_null_pointer() {
        gc_untrack(std::ptr::null_mut());
    }

    #[test]
    fn test_collect_returns_zero_when_all_rooted() {
        reset_gc_for_test();
        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_dict();
        gc_add_root(MbValue::from_ptr(a));
        gc_add_root(MbValue::from_ptr(b));
        assert_eq!(collect(), 0, "all rooted → 0 freed");
        gc_clear_roots();
        unsafe { simulate_jit_release(a); simulate_jit_release(b); }
        assert_eq!(collect(), 2);
    }

    #[test]
    fn test_tuple_of_ints_not_tracked() {
        // #2128: tuple of atomic-only values skips gc_track (CPython-style).
        // Cycles are impossible without a container element.
        reset_gc_for_test();
        let t = MbObject::new_tuple(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(gc_get_count(), 0, "atomic-only tuple should NOT be tracked");
        unsafe { simulate_jit_release(t); }
        assert_eq!(collect(), 0, "no tracked obj → nothing for gc to free");
    }

    #[test]
    fn test_tuple_with_container_tracked() {
        // #2128 conditional path: tuple holding a container CAN form a cycle
        // (list.append(tuple)), so it must remain tracked. Companion contract
        // to test_tuple_of_ints_not_tracked.
        reset_gc_for_test();
        let inner = MbObject::new_list(vec![]);
        assert_eq!(gc_get_count(), 1, "list is tracked");
        let t = MbObject::new_tuple(vec![MbValue::from_ptr(inner)]);
        assert_eq!(gc_get_count(), 2, "tuple holding a list IS tracked");
        unsafe { simulate_jit_release(t); simulate_jit_release(inner); }
        assert_eq!(collect(), 2, "both objects collected");
    }

    #[test]
    fn test_retrack_after_untrack() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![]);
        assert_eq!(gc_get_count(), 1);
        gc_untrack(obj);
        assert_eq!(gc_get_count(), 0);
        gc_track(obj);
        assert_eq!(gc_get_count(), 1);
        unsafe { simulate_jit_release(obj); }
        assert_eq!(collect(), 1, "re-tracked object should be collected");
    }

    #[test]
    fn test_large_threshold_no_auto_collection() {
        reset_gc_for_test();
        gc_set_threshold(usize::MAX);
        gc_enable();
        for _ in 0..10 { let _p = MbObject::new_list(vec![]); }
        assert_eq!(gc_get_count(), 10, "no auto-collect with huge threshold");
        gc_disable();
        collect();
        gc_set_threshold(700);
    }

    #[test]
    fn test_stats_collections_count_increments() {
        reset_gc_for_test();
        let (before, _, _) = gc_get_stats();
        collect();
        let (after_one, _, _) = gc_get_stats();
        collect();
        let (after_two, _, _) = gc_get_stats();
        assert_eq!(after_one, before + 1);
        assert_eq!(after_two, before + 2);
    }

    #[test]
    fn test_disabled_gc_manual_collect_still_works() {
        reset_gc_for_test();
        gc_disable();
        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_list(vec![]);
        unsafe { simulate_jit_release(a); simulate_jit_release(b); }
        assert_eq!(collect(), 2, "manual collect works even when auto-GC is disabled");
    }

    #[test]
    fn test_collect_mixed_reachable_unreachable() {
        reset_gc_for_test();
        let live = MbObject::new_list(vec![MbValue::from_int(1)]);
        let dead = MbObject::new_list(vec![MbValue::from_int(2)]);
        gc_add_root(MbValue::from_ptr(live));
        unsafe { simulate_jit_release(dead); }
        assert_eq!(collect(), 1, "only the unreachable object should be freed");
        assert_eq!(gc_get_count(), 1, "live object still tracked");
        gc_clear_roots();
        unsafe { simulate_jit_release(live); }
        assert_eq!(collect(), 1, "live object freed after root removed");
    }

    #[test]
    fn test_two_separate_cycles_collected_together() {
        reset_gc_for_test();
        let a = MbObject::new_list(vec![]);
        let b = MbObject::new_list(vec![]);
        unsafe {
            if let ObjData::List(ref lock) = (*a).data { lock.write().unwrap().push(MbValue::from_ptr(b)); }
            if let ObjData::List(ref lock) = (*b).data { lock.write().unwrap().push(MbValue::from_ptr(a)); }
        }
        let c = MbObject::new_list(vec![]);
        let d = MbObject::new_list(vec![]);
        unsafe {
            if let ObjData::List(ref lock) = (*c).data { lock.write().unwrap().push(MbValue::from_ptr(d)); }
            if let ObjData::List(ref lock) = (*d).data { lock.write().unwrap().push(MbValue::from_ptr(c)); }
        }
        assert_eq!(gc_get_count(), 4);
        assert_eq!(collect(), 4, "all four cyclic objects should be collected");
        assert_eq!(gc_get_count(), 0);
    }

    #[test]
    fn test_gc_get_count_after_partial_sweep() {
        reset_gc_for_test();
        let live = MbObject::new_list(vec![]);
        let dead1 = MbObject::new_list(vec![]);
        let dead2 = MbObject::new_list(vec![]);
        gc_add_root(MbValue::from_ptr(live));
        unsafe { simulate_jit_release(dead1); simulate_jit_release(dead2); }
        assert_eq!(gc_get_count(), 3);
        assert_eq!(collect(), 2);
        assert_eq!(gc_get_count(), 1, "only live object remains tracked");
        gc_clear_roots();
        unsafe { simulate_jit_release(live); }
        collect();
        assert_eq!(gc_get_count(), 0);
    }

    #[test]
    fn test_nested_reachability_via_dict() {
        reset_gc_for_test();
        let inner = MbObject::new_list(vec![MbValue::from_int(99)]);
        let outer = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*outer).data {
                lock.write().unwrap().insert("key".into(), MbValue::from_ptr(inner));
            }
        }
        gc_add_root(MbValue::from_ptr(outer));
        assert_eq!(collect(), 0, "both reachable via root");
        gc_clear_roots();
        unsafe { simulate_jit_release(outer); simulate_jit_release(inner); }
        assert_eq!(collect(), 2, "both freed when root removed");
    }

    #[test]
    fn test_instance_fields_reachable() {
        reset_gc_for_test();
        let field_val = MbObject::new_list(vec![]);
        let inst = MbObject::new_instance("Foo".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                fields.write().unwrap().insert("x".to_string(), MbValue::from_ptr(field_val));
            }
        }
        gc_add_root(MbValue::from_ptr(inst));
        assert_eq!(collect(), 0, "instance and field are both reachable");
        gc_clear_roots();
        unsafe { simulate_jit_release(inst); simulate_jit_release(field_val); }
        assert_eq!(collect(), 2, "both freed after root removed");
    }

    /// Re-entrancy guard: collect() returns 0 when gc.collecting is already true.
    #[test]
    fn test_collect_reentrant_guard() {
        reset_gc_for_test();
        let obj = MbObject::new_list(vec![]);
        GC.with(|gc| { gc.borrow_mut().collecting = true; });
        assert_eq!(collect(), 0, "collect() must return 0 when already collecting");
        GC.with(|gc| { gc.borrow_mut().collecting = false; });
        unsafe { simulate_jit_release(obj); }
        assert_eq!(collect(), 1, "collect() works after collecting flag cleared");
    }

    /// alloc_count resets to 0 after each collect() call.
    #[test]
    fn test_alloc_count_reset_after_collect() {
        reset_gc_for_test();
        let _a = MbObject::new_list(vec![]);
        let _b = MbObject::new_list(vec![]);
        GC.with(|gc| { assert_eq!(gc.borrow().alloc_count, 2); });
        collect();
        GC.with(|gc| { assert_eq!(gc.borrow().alloc_count, 0); });
    }
}
