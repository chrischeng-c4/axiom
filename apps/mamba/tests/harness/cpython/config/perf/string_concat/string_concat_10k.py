"""Bulk-work bench for `"".join()` on a 4-element list of short strings,
10000 iterations — the canonical workload from GitHub issue #1382.

End-user scenario: any string-templating hot path that concatenates a
small fixed number of short string fragments per call. Mirrors the
mamba-internal `string_concat` benchmark (see `src/bench/mod.rs:379`),
scaled from 100 iterations (workload smoke) to 10000 (perf gate
discriminator) so per-iter alloc cost dominates over Python startup.

CPython reference: ~140 ns per iter on Apple Silicon — interned literals,
list freelist, pymalloc small-block arena. Mamba currently ~520 ns per
iter (0.27x ratio) because every iter goes through the general-purpose
heap path for all four short strings + the list + the join result.

The fix that unblocks this floor (per #1382 body):
  1. Small-string interning of literal constants in codegen output
     (`mb_str_intern` returning a refcount-immortal handle, so
     `"hello"` etc. become compile-time pointers, not per-call allocs).
  2. Per-thread pymalloc-equivalent freelist for objects <=512 bytes
     (tiny lists, short strings, small tuples recycle into a freelist
     instead of returning to the global allocator).

DoD: exits 0 under both CPython and mamba; emits an
`INTERNAL_TIME_NS=<u64>` marker the regression-pin test parses to
compute the mamba/cpython ratio.
"""

import sys
import time

ITERS = 10000

# Pre-declare `out` at module scope so mamba's type-checker sees it
# after the loop body (mamba scoping does not currently hoist names
# first-bound inside a `for` body — guard against that gap explicitly).
out = ""

# Internal-time marker — unbiased per-call cost. Wall-clock would
# include Python startup overhead; the marker isolates the hot loop.
_t0 = time.perf_counter()
for _ in range(ITERS):
    out = "".join(["hello", " ", "world", "!"])
_t1 = time.perf_counter()
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Sanity-check the join result — guards against a future codegen bug
# that silently drops the body of the loop (which would also tank the
# bench in the "good" direction and falsely close the pin).
assert out == "hello world!", f"unexpected join result: {out!r}"
print("string_concat:", out)
