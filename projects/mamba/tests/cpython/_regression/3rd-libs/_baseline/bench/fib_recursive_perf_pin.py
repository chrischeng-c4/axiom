"""Recursive Fibonacci hot loop - perf-pin fixture for GitHub issue #1073.

Pins the recursive call-convention cost targeted by #1073 with a
deterministic pure-Python workload. This slice intentionally stays at the
call-frame layer: it does not reach into hidden tstate ABI work,
recursion-depth inlining, or unrelated hot-path extern optimizations.

Constants the design relies on:

- `fib((i & 3) + 12)` cycles through `fib(12)` ... `fib(15)`. Varying the
  argument across iterations prevents any future optimizer from folding the
  recursive call tree to a constant; we are measuring recursive call
  overhead, not a cached literal.
- `ITERS = 25` keeps the current mamba runtime below the perf runner's
  timeout while still exercising thousands of recursive Python calls per
  sample.
- `acc = (acc + fib(...)) & 0xFFFFFFFF` keeps the printed checksum bounded
  while forcing every recursive result to stay observable.

# tier: compute
"""


def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)


ITERS = 25

acc: int = 0
for i in range(ITERS):
    acc = (acc + fib((i & 3) + 12)) & 0xFFFFFFFF

print("fib_recursive:", acc)
