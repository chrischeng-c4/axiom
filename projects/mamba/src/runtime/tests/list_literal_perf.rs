#![cfg(test)]

//! Perf-gate regression pin for GitHub issue #2517 —
//! SmallVec inline-storage for short list literals.
//!
//! Status (2026-05-19): `ObjData::List` now wraps an `MbList`
//! (`SmallVec<[MbValue; 8]>`) instead of `Vec<MbValue>`, and the
//! fixed-arity JIT shims `mb_list_new_1..8` (`runtime::list_ops`) build
//! the storage inline via `smallvec![..]` + `new_list_inline`, skipping
//! the `Vec` heap allocation entirely. For literals with N ≤ 8 — the
//! dominant shape the JIT emits — this collapses the per-literal heap
//! cost from `Box(MbObject) + Vec(elements)` (two allocs) to
//! `Box(MbObject)` (one alloc).
//!
//! This pin is the binding regression contract: any future short-list
//! literal construction MUST beat the `Vec`-backed equivalent by at
//! least 30% on the 4-element micro-loop, otherwise #2517's
//! list-literal allocator floor regresses.
//!
//! Methodology: best-of-N (N=5) per implementation to defang per-run
//! jitter, then assert
//! `best(inline) / best(vec_backed) <= 0.70`.
//!
//! Run with:
//!
//!   cargo test -p mamba --release --test list_literal_perf_2517 \
//!       -- --ignored test_list_literal_inline_gate_2517

use std::time::Instant;

use crate::runtime::rc::{MbList, MbObject};
use crate::runtime::value::MbValue;
use smallvec::smallvec;

const ITERS: usize = 1_000_000;
const SAMPLES: usize = 5;

/// Old path: `Vec`-backed construction. `new_list` will copy the `Vec`
/// into the inline `SmallVec` at the boundary, but the caller's `Vec`
/// allocation still happens — so this is exactly the pre-#2517 cost
/// for short literals built through the generic constructor.
fn bench_vec_backed() -> u128 {
    let t0 = Instant::now();
    for i in 0..ITERS {
        let a = MbValue::from_int(i as i64);
        let b = MbValue::from_int((i + 1) as i64);
        let c = MbValue::from_int((i + 2) as i64);
        let d = MbValue::from_int((i + 3) as i64);
        let ptr = MbObject::new_list(vec![a, b, c, d]);
        std::hint::black_box(ptr);
        // Drop the resulting list so the GC can free it; without this
        // the loop builds 1M live lists and the allocator's free-list
        // pressure dominates the measurement.
        unsafe {
            crate::runtime::rc::mb_release(ptr);
        }
    }
    t0.elapsed().as_nanos()
}

/// New path: inline-storage construction. `smallvec![a,b,c,d]` builds
/// the 4-element buffer entirely in the SmallVec's inline array — no
/// heap touch for the elements — and `new_list_inline` consumes it
/// without any `Vec` boundary copy.
fn bench_inline() -> u128 {
    let t0 = Instant::now();
    for i in 0..ITERS {
        let a = MbValue::from_int(i as i64);
        let b = MbValue::from_int((i + 1) as i64);
        let c = MbValue::from_int((i + 2) as i64);
        let d = MbValue::from_int((i + 3) as i64);
        let buf: MbList = smallvec![a, b, c, d];
        let ptr = MbObject::new_list_inline(buf);
        std::hint::black_box(ptr);
        unsafe {
            crate::runtime::rc::mb_release(ptr);
        }
    }
    t0.elapsed().as_nanos()
}

fn best_of<F: FnMut() -> u128>(mut f: F, n: usize) -> u128 {
    let mut best = u128::MAX;
    for _ in 0..n {
        let v = f();
        if v < best {
            best = v;
        }
    }
    best
}

#[test]
#[ignore = "#2517 perf gate (best-of-5) - 1M-iter 4-element list literal micro-loop; opt in via --ignored. \
            Asserts best(new_list_inline) / best(new_list) <= 0.70 per acceptance criterion \
            (≥30% reduction in short-list literal allocation cost)."]
fn test_list_literal_inline_gate_2517() {
    let _ = bench_vec_backed();
    let _ = bench_inline();

    let vec_ns = best_of(bench_vec_backed, SAMPLES);
    let inline_ns = best_of(bench_inline, SAMPLES);
    let ratio = inline_ns as f64 / vec_ns as f64;
    let savings_pct = 100.0 * (1.0 - ratio);
    eprintln!(
        "#2517 best-of-{} 4-element list literal micro-loop: \
         Vec-backed {} ns ({} ns/iter), \
         inline SmallVec {} ns ({} ns/iter), \
         ratio = {:.3}× ({:.1}% savings)",
        SAMPLES,
        vec_ns,
        vec_ns / ITERS as u128,
        inline_ns,
        inline_ns / ITERS as u128,
        ratio,
        savings_pct
    );
    assert!(
        ratio <= 0.70,
        "#2517 perf gate FAIL: best-of-{} ratio = {:.3}× ({:.1}% savings) \
         exceeds 0.70 floor (≥30% reduction required per acceptance). \
         inline SmallVec {} ns vs Vec-backed {} ns. The short-list literal \
         allocator floor regressed; re-investigate before re-closing.",
        SAMPLES,
        ratio,
        savings_pct,
        inline_ns,
        vec_ns
    );
}
