"""Integer multiplication hot loop — perf-pin fixture for GitHub issue #2514.

Mirrors the `mamba bench` builtin `int_mul_loop` workload (an inline
`result = result * i` chain), wrapped in a 100_000-iter outer loop with
`int_mul_loop_perf_pin_2514.rs` median-of-9 gate can read an unbiased
per-iter cost.

Constants the design relies on:

- `bound = 10 + (k & 3)` cycles through 10..=13. Cycling the loop bound
  across iters prevents any future mamba MIR optimiser from
  constant-folding the entire inner multiply chain (we are measuring
  the multiply hot path, not a folded constant).
- The full intermediate product is bounded by 13! = 6_227_020_800 which
  fits inside the 48-bit native-int fast path
  (`project_mamba_runtime_correctness_gaps_2026_05_13`); we are
  measuring the boxed-int / native-i64 multiply cost, not bigint
  fallback.
- `acc = (acc + result) & 0xFFFFFFFF` (32-bit wrap-add) keeps the
  running accumulator bounded by 2^32 (well inside the 48-bit
  native-int fast path,
  `project_mamba_runtime_correctness_gaps_2026_05_13`) while still
  forcing every multiply chain to be observable so the JIT cannot
  dead-code-eliminate it. We avoided `& 0xFF` and `^=` because
  factorial products are divisible by large powers of 2 and even-count
  XOR cycles cancel — both produce a zero accumulator that the JIT
  could prove statically.
  marker on the wire — per
  `project_mamba_jit_drops_branches_after_stdlib_call` we never
  interleave an `assert` between the multiply and the marker.

# tier: compute
"""



ITERS = 100_000

acc: int = 0
for k in range(ITERS):
    result: int = 1
    i: int = 1
    bound: int = 10 + (k & 3)
    while i <= bound:
        result = result * i
        i = i + 1
    acc = (acc + result) & 0xFFFFFFFF

print("int_mul_loop:", acc)
