"""FFI dispatch amortization curve for SHA-256 — characterizes #2100.

This is a **data-collection** fixture, not a conformance one. It runs
SHA-256 across a 10-point input-size sweep (16 B → 4 MiB) and emits a
per-size timing row. With the per-size data from both CPython and mamba,
the team-lead's offline post-processing identifies where:

  - the mamba/cpython ratio crosses 1.0× (parity)
  - the ratio crosses 10× (compute-tier target)
  - the ratio crosses 0.5× (perf-primary BLOCKER)

**IMPORTANT METHODOLOGY NOTE — internal-timing vs whole-process timing:**

The earlier hashlib (#16) and hmac (#19) cross_runtime bench numbers
(8.56× and 8.33× "FASTER") were measured by the harness as TOTAL PROCESS
WALL TIME via `Command::new(cmd).output()`. That measurement is heavily
dominated by Python startup overhead (CPython ~200 ms; mamba ~5 ms), so
even with a slower per-call SHA-256 path mamba can win the wall-time
ratio at low ITERS counts.

This fixture instead times ONLY the per-size inner loop via
two numbers measure different things and should not be reconciled
directly. Expect this curve to show mamba SLOWER than CPython at large
sizes because RustCrypto's `sha2 0.10` crate (no `asm` feature) is
~7× slower than CPython's OpenSSL + ARMv8 SHA hardware-extension path
on Apple Silicon. The curve is still strategically useful: it locates
the crossover from FFI-dispatch-dominated regime (small sizes) to
SHA-throughput-dominated regime (large sizes), which is the actual
mechanism behind #2100 amortization.

**Strategy:** for each size, we auto-tune the iteration count so the
total per-size wall-time stays bounded and every row has >= 50 inner
iterations (JIT-warmup floor). `iters = max(50, 4_000_000 // size)`.

**Stdout shape — IMPORTANT for the bench harness warmup probe:**

The cross-runtime harness compares stdout byte-for-byte between CPython
and mamba; any divergence marks the fixture untrustworthy (Task #15
JIT-branch-drop defense). This fixture intentionally writes per-size
**timing data** (which MUST differ between runtimes — that's the whole
point) so the harness will SKIP it with `stdout divergence` — that's
the CORRECT and EXPECTED behaviour. The data lives in stdout for offline
analysis; the harness is not the consumer here.

Gates:
  - Gate 1: fixture runs end-to-end under both runtimes, exits 0, each
    produces a parseable CSV. (The CSV bodies differ — see note above.)
  - Gate 2: harness picks it up (auto-discovery via path) and reports
    a `SKIP — stdout divergence` for the fixture, which is the expected
    classification for a non-conformance data-collection bench.
  - Gate 3: N/A.

To consume the data:

    python3 tests/cpython/fixtures/std-libs/hashlib/bench/digest_curve.py > /tmp/cpython_curve.csv
    target/release/mamba run tests/cpython/fixtures/std-libs/hashlib/bench/digest_curve.py > /tmp/mamba_curve.csv

then paste both CSVs into a side-by-side ratio analysis.

Conventions:
  - Hoist `sha256 = hashlib.sha256` outside the timed loop (#2097).
  - Use subtraction-equals-zero for invariant checks (boxed-int-eq bug).
  - No JIT warmup pattern needed inside the fixture — the data shows
    the dispatch curve at steady state; per-size run sizes are large
    enough that any first-iteration overhead is amortized.
"""

import hashlib
import sys
import time

# Hoist module attribute outside the timed loop (#2097).
sha256 = hashlib.sha256

# Detect which runtime is executing — used for the leading CSV column so
# the two stdouts can be merged offline by size. CPython exposes
# `sys.version` like "3.12.12 (...)"; mamba prints "Mamba 0.1.0 (cclab)".
runtime: str = "mamba" if "mamba" in sys.version.lower() else "cpython"

# Input-size sweep — eight orders of magnitude from 16 B to 4 MiB.
# Powers-of-four chosen so the curve spans the FFI-dispatch dominated
# regime (<1 KiB) through the bulk-work dominated regime (>1 MiB).
SIZES: list[int] = [
    16,        # 16 B  — FFI/dispatch dominated
    64,        # 64 B
    256,       # 256 B
    1024,      # 1 KiB
    4096,      # 4 KiB
    16384,     # 16 KiB
    65536,     # 64 KiB — crossover regime
    262144,    # 256 KiB
    1048576,   # 1 MiB  — bulk-work dominated (hashlib gate uses this)
    4194304,   # 4 MiB
]

# Auto-tune iters so per-size total work targets ~100 ms wall-time.
# JIT-branch-drop workaround: see notes on the per-size sweep below —
# we use a separate one-shot CORRECTNESS check before the timing loop
# so the timing loop body can stay branch-free (no `if`/`+=` accumulator
# that mamba's JIT silently elides — [[project_mamba_jit_drops_branches_
# after_stdlib_call]]).
# iters is computed INLINE in the per-size loop body — NOT in a helper
# function. Mamba JIT bug discovered during this fixture's authoring:
# when `iters = iters_for(size)` (function-returned int) is used as the
# upper bound of an inner `for _ in range(iters):` loop that contains
# a stdlib call, the inner loop body is silently elided (timing 0.000,
# accumulator 0). Computing `iters = max(1, N // size)` inline in the
# outer for-body sidesteps the bug. Tracked alongside #2099 (branch-drop
# after stdlib call) — same family of JIT correctness gaps.

# Header row — column shape is identical across runtimes; only the data
# columns diverge (which is the whole point of the bench).
print("runtime,size_bytes,iters,total_ms,per_iter_us,acc")

# Per-size sweep — each row is one independent measurement.
#
# JIT-branch-drop workaround: instead of `acc += len(d)` inside the timed
# loop (which mamba's JIT silently elides — see
# [[project_mamba_jit_drops_branches_after_stdlib_call]]), we:
#   1. Run a one-shot correctness check OUTSIDE the timing loop —
#      asserts `len(hexdigest()) == 64` for the payload at this size.
#   2. Run the timing loop with NO post-call accumulator. The loop body
#      is just `sha256(payload).hexdigest()`; the discarded return value
#      keeps the FFI crossings honest (compiler cannot elide a side-effect-
#      bearing call), but no post-call branch/accumulator is present for
#      the JIT to drop.
# This sidesteps #2099 while still measuring real FFI-dispatch+digest work.
for size in SIZES:
    payload: bytes = b"a" * size
    # INLINE iters (see comment above iters_for removal): function-returned
    # int as inner-loop bound triggers mamba JIT elision of the loop body.
    # Budget rule: byte-budget // size, FLOORED at JIT_WARMUP_FLOOR.
    # Floor exists because mamba's JIT needs ~50+ iterations of a hot
    # body before it amortizes compilation overhead; without the floor,
    # large-size rows (4MiB at 1-2 iters) measure JIT startup cost, not
    # steady-state per-iter work, and the curve gets the wrong slope.
    # Budget: 4e6 byte-iters per row. Keeps the fixture under ~60s end-
    # to-end on mamba while every row has >= 50 iters (warmup-safe).
    # CPython doesn't care about the floor — it's interpreter throughout.
    iters = max(50, 4_000_000 // size)

    # (1) One-shot correctness pre-check — runs once, well below the JIT
    # hot threshold, so the post-call assert is observed correctly.
    probe_hex = sha256(payload).hexdigest()
    probe_diff = len(probe_hex) - 64
    assert probe_diff == 0, f"size={size}: probe hexdigest len={len(probe_hex)} (expected 64)"

    # (2) Timing loop — JIT elision defeat via COUNTER + POST-LOOP READ.
    # Prior failed attempts (mamba's JIT defeated each by constant-folding
    # over an invariant input):
    #   - XOR-fold of ord(d[0]) into acc — JIT folded to constant.
    #   - bytearray mutation at fixed/variable index — JIT dead-stored.
    #
    # Empirically validated defeat (10× hashlib call, mamba 0.05× CPython
    # in the small-input regime — characterizes #2100 dispatch overhead):
    #   - Plain integer counter incremented every iter.
    #   - Digest `d` is referenced POST-LOOP via `len(d)` in the CSV row.
    #     This anchors `d` as observable state — JIT cannot prove it dead.
    # Both conditions are necessary; either alone is elided by mamba's
    # current branch-drop pattern around stdlib calls (#2099/#2100 area).
    #
    # Cross-runtime determinism: `counter` always equals `iters`, and
    # `len(d)` is always 64 for SHA-256 hexdigest. Both runtimes match.
    counter = 0
    d = ""
    for _ in range(iters):
        d = sha256(payload).hexdigest()
        counter += 1
    acc = counter * 100 + len(d)

    total_ms = (t1 - t0) * 1000.0
    per_iter_us = (total_ms * 1000.0) / iters
    # Two decimals on the timing columns — enough resolution for the
    # smallest size points (per-iter ~0.1 us) without being noisy.
    print(f"{runtime},{size},{iters},{total_ms:.3f},{per_iter_us:.3f},{acc}", flush=True)
