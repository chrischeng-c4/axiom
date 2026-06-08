#![cfg(test)]

//! Perf-gate regression pin for GitHub issue #2518 —
//! container-lock uncontended fast path.
//!
//! Status (2026-05-19): `runtime::rc::MbRwLock` now wraps
//! `parking_lot::RwLock` instead of `std::sync::RwLock`. On the
//! canonical uncontended write+read micro-loop (10M iters of
//! `lock.write().push(); lock.read().len()`), parking_lot's lock
//! takes ~16 ns/pair vs std::sync's ~22 ns/pair on aarch64-darwin —
//! a ~30% reduction, clearing the issue's ≥20% acceptance gate.
//!
//! This pin is the binding regression contract: any future
//! `MbRwLock<T>` implementation MUST beat `std::sync::RwLock<T>` by
//! at least 20% on this exact pattern, otherwise #2518's
//! single-threaded container R/W floor regresses.
//!
//! Methodology: best-of-N (N=5) per implementation to defang
//! per-run jitter, then assert
//! `best(MbRwLock) / best(std::sync::RwLock) <= 0.80`.
//!
//! Run with:
//!
//!   cargo test -p mamba --release --test container_lock_perf_2518 \
//!       -- --ignored test_container_lock_uncontended_rw_gate_2518

use std::sync::RwLock as StdRwLock;
use std::time::Instant;

use crate::runtime::rc::MbRwLock;

const ITERS: usize = 10_000_000;
const SAMPLES: usize = 5;

fn bench_std() -> u128 {
    let lock = StdRwLock::new(Vec::<u64>::with_capacity(16));
    let t0 = Instant::now();
    for i in 0..ITERS {
        {
            let mut w = lock.write().unwrap();
            w.push(i as u64);
            if w.len() >= 8 {
                w.clear();
            }
        }
        {
            let r = lock.read().unwrap();
            std::hint::black_box(r.len());
        }
    }
    t0.elapsed().as_nanos()
}

fn bench_mb() -> u128 {
    let lock = MbRwLock::new(Vec::<u64>::with_capacity(16));
    let t0 = Instant::now();
    for i in 0..ITERS {
        {
            let mut w = lock.write().unwrap();
            w.push(i as u64);
            if w.len() >= 8 {
                w.clear();
            }
        }
        {
            let r = lock.read().unwrap();
            std::hint::black_box(r.len());
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
#[ignore = "#2518 perf gate (best-of-5) - 10M-iter write+read micro-loop; opt in via --ignored. \
            Asserts best(MbRwLock) / best(std::sync::RwLock) <= 0.80 per acceptance criterion \
            (≥20% reduction in uncontended container R/W cost)."]
fn test_container_lock_uncontended_rw_gate_2518() {
    let _ = bench_std();
    let _ = bench_mb();

    let std_ns = best_of(bench_std, SAMPLES);
    let mb_ns = best_of(bench_mb, SAMPLES);
    let ratio = mb_ns as f64 / std_ns as f64;
    let savings_pct = 100.0 * (1.0 - ratio);
    eprintln!(
        "#2518 best-of-{} uncontended rw micro-loop: \
         std::sync::RwLock {} ns ({} ns/iter), \
         MbRwLock {} ns ({} ns/iter), \
         ratio = {:.3}× ({:.1}% savings)",
        SAMPLES,
        std_ns,
        std_ns / ITERS as u128,
        mb_ns,
        mb_ns / ITERS as u128,
        ratio,
        savings_pct
    );
    assert!(
        ratio <= 0.80,
        "#2518 perf gate FAIL: best-of-{} ratio = {:.3}× ({:.1}% savings) \
         exceeds 0.80 floor (≥20% reduction required per acceptance). \
         MbRwLock {} ns vs std::sync::RwLock {} ns. The container-lock \
         uncontended fast path regressed; re-investigate before re-closing.",
        SAMPLES,
        ratio,
        savings_pct,
        mb_ns,
        std_ns
    );
}
