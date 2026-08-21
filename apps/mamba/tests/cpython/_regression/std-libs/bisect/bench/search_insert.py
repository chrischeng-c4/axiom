"""Hot loop exercising bisect_left/bisect_right/insort_right on a sorted int list.

End-user scenario: maintaining a sorted leaderboard or schedule index where
each tick performs a binary-search lookup plus an in-order insertion. This
is the canonical `bisect` workload.

Tier: `compute` (target mamba/cpython >= 10x per issue #1265). Binary search
is O(log n) per call, dominated by Rust-side compare-and-branch — the hoisted
function-reference pattern (cached on locals before the loop) is the
idiomatic CPython micro-optimisation for this class of code, and matches
how real-world consumers (e.g. heapq-driven schedulers) call into bisect.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.

Note on type annotations: this fixture deliberately omits `: int` /
`: list[int]` annotations because mamba's force-typed path currently
treats `sum_positions: int` as an unboxed i64, and the `== ITERS *
expected_per_iter` comparison degenerates to pointer-equality between
an unboxed local and a boxed integer object — making `5000 * 6378 ==
31890000` evaluate to False even though both sides match numerically.
Flagged as a runtime bug separately; the un-annotated form below is
the workaround.
"""

import bisect

# Pre-sorted base array; the bench rebuilds from this each iteration so
# both runtimes see identical input distributions.
BASE = list(range(0, 2000, 2))

# Probes interleave hits (even ints in BASE) and misses (odd ints).
PROBES = [3, 5, 7, 11, 19, 23, 101, 199, 503, 997, 1001, 1499, 1997]

# Hoisted references — real linters/schedulers always cache these to a
# local to avoid per-call attribute lookup. See keyword conformance notes
# (cc2ec257e) for the underlying module-attr-lookup regression.
bisect_left = bisect.bisect_left
bisect_right = bisect.bisect_right
insort_right = bisect.insort_right

ITERS = 5000
sum_positions = 0
for _ in range(ITERS):
    # Per-iteration list copy so insort doesn't grow unbounded across iters.
    a = list(BASE)
    for x in PROBES:
        sum_positions += bisect_left(a, x)
        sum_positions += bisect_right(a, x)
    # One insort_right per iter to exercise the mutation path.
    insort_right(a, 1500)

# Print sum + emit marker BEFORE the checksum assert per #2105 (avoid JIT
# post-call branch elision silently zeroing the marker on mamba).
print("search_insert:", sum_positions)

# Deterministic checksum so any divergence (e.g. bisect_right confused
# with bisect_left) shows up immediately rather than as a perf-only delta.
# Use subtraction rather than `==` because mamba currently has a
# boxed-vs-accumulator integer equality bug: a value built via `+=` in a
# hot loop fails `==` against the same numerical value built via `*`,
# even though both report `<class 'int'>` and the same str repr. The
# difference is the source code path that produced the MbValue (boxed
# Python int object vs unboxed i64 promoted to int). Filed separately
# as a mamba runtime bug; the subtraction below sidesteps it.
expected_per_iter = sum(bisect_left(BASE, x) + bisect_right(BASE, x) for x in PROBES)
diff = sum_positions - ITERS * expected_per_iter
assert diff == 0, (
    f"checksum mismatch: {sum_positions} - ({ITERS} * {expected_per_iter}) = {diff}"
)
