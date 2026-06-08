"""Recursive factorial — perf-pin fixture for GitHub issue #2515.

Mirrors the `mamba bench` builtin `factorial_recursive` workload
(`fact(15)` recursive multiply chain), wrapped in a 100_000-iter hot
`factorial_recursive_perf_pin_2515.rs` median-of-9 gate can read an
unbiased per-call cost.

Constants the design relies on:

- `fact((i & 7) + 8)` cycles through `fact(8)` ... `fact(15)`. Varying
  the argument across iters prevents any future mamba MIR optimiser
  from constant-folding a single `fact(15)` call out of the hot loop
  (we are measuring call-frame cost, not a folded constant).
- `& 0xFF` reduces the accumulated product to a 1-byte value before
  the `+`, so `acc` stays well inside the JIT's native i64 fast path
  for the full 100_000-iter run and the accumulator is not itself the
  bottleneck.
- The first `print` of `acc` is emitted BEFORE the
  `project_mamba_jit_drops_branches_after_stdlib_call` we never
  interleave an `assert` between the call and the marker.

# tier: compute
"""



def fact(n: int) -> int:
    if n <= 1:
        return 1
    return n * fact(n - 1)


ITERS = 100_000

acc: int = 0
for i in range(ITERS):
    acc = acc + (fact((i & 7) + 8) & 0xFF)

print("factorial_recursive:", acc)
