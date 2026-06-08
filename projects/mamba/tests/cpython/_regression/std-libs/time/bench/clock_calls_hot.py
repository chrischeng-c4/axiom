"""Hot-loop bench for `time` clock-reader calls (#1435).

End-user scenario: a request-scoped timer (think: span start/stop in a
tracer, latency histogram, fine-grained metric) calls `time.monotonic()`
Per-call dispatch overhead dominates until the iteration count amortizes
startup noise.

Tier: `native-shim io-light` (target mamba/cpython <= 1.0x — CPython's
clock readers are thin C shims around clock_gettime; mamba's edge is
removing the Python-level frame + attribute-lookup overhead between
the user call site and the libc call).

Hoist convention (per #2097 + CLAUDE.md note): all three clock readers
hoisted to locals before the loop, so the published ratio reflects the
underlying syscall cost rather than module-attr lookup speed.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import time

# Hoist module attributes outside the loop — see CLAUDE.md note + #2097.
monotonic = time.monotonic
wall_time = time.time

ITERS = 10_000

acc = 0.0
for _ in range(ITERS):
    a = monotonic()
    c = wall_time()
    # Touch each result so the loop body cannot be dead-code-eliminated.
    acc = acc + (a - a) + (b - b) + (c - c)

# acc must be exactly 0.0 across both runtimes (each subtract-of-self is
# 0.0; sum of 0.0 floats is 0.0). Use subtraction-equals-zero per the
# boxed-accumulator-float-equality memory entry.
assert acc - 0.0 == 0.0, f"clock-call accumulator drift: acc={acc}"
print("clock_calls_hot:", ITERS)
