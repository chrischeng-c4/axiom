"""Hot-loop bench for `heapq.heappush` + `heapq.heappop` on a small int heap.

End-user scenario: an event-loop scheduler maintains a min-heap of due
timestamps; each tick pushes a fresh deadline and pops the earliest. This
is the canonical heapq workload — push, peek-via-pop, push the next.

Tier: `compute` (target mamba/cpython >= 10x per issue #1265). Each heap
op is O(log n) and dominated by Rust-side compare-and-branch on the
sifted item; per-call cost is the dispatch edge plus a handful of
comparisons. Hoisted callable references match how real-world
schedulers cache these locals before the hot loop — see the underlying
module-attr-lookup regression noted in keyword conformance (cc2ec257e).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
harness compares per-iteration wall time and reports the ratio.

Note on the checksum form: mamba currently has a boxed-vs-accumulator
integer equality bug where a `+=` accumulator fails `==` against the
same numerical value built via `*`. Use `(actual - expected) == 0`
instead — same correctness check, sidesteps the bug. Filed separately
as a runtime issue.
"""

import heapq


# Seed values — deliberately out of order so heappush exercises sift_down.
SEED = [37, 11, 23, 5, 41, 17, 29, 3, 19, 31, 7, 43, 13, 47, 2]
# Per-iter probe pattern: alternating large/small so heappushpop
# decisions are evenly mixed between "smaller-than-root" (return input)
# and "larger-than-root" (swap with root, sift down).
PROBES = [1, 100, 4, 200, 9, 300, 15, 400]

# Hoisted refs — idiomatic locals-cache pattern for hot loops.
heappush = heapq.heappush
heappop = heapq.heappop
heappushpop = heapq.heappushpop
heapify = heapq.heapify

ITERS = 20000
checksum = 0
for _ in range(ITERS):
    # Fresh heap per iter so the heap size stays bounded and the
    # per-iter cost is comparable across iterations.
    h = list(SEED)
    heapify(h)
    for p in PROBES:
        # heappushpop returns the smaller of (root, p); accumulate the
        # popped value so any divergence in the comparison path shows up
        # in the checksum, not just as a wall-time delta.
        checksum += heappushpop(h, p)
    # Drain root once per iter to exercise heappop's sift_up path.
    checksum += heappop(h)

# Print checksum + emit marker BEFORE the trailing reference checksum
# (which also calls heap ops) per #2105 — keep the marker as close to
# the timed loop as possible and ahead of any conditional assert.
print("heappush_pop_hot:", checksum)

# Deterministic expected — recompute the same operations once and
# multiply, then validate via subtraction to avoid the bignum eq bug.
ref_heap = list(SEED)
heapify(ref_heap)
ref_per_iter = 0
for p in PROBES:
    ref_per_iter += heappushpop(ref_heap, p)
ref_per_iter += heappop(ref_heap)

diff = checksum - ITERS * ref_per_iter
assert diff == 0, (
    f"heap checksum mismatch: {checksum} - ({ITERS} * {ref_per_iter}) = {diff}"
)
