"""PEP 654 try / except — no-raise fast path perf bench.

End-user scenario: `try: ... except SomeError: ...` wrapping a
tight body that never actually raises — the canonical defensive
guard around hot code (cache miss probes, conditional decode,
typed-dispatch entry). PEP 654 added except*, but the no-raise
fast path remains the dominant cost in real systems. CPython's
no-raise try block is essentially free (SETUP_FINALLY + POP_BLOCK);
mamba's try lowers to a frame-stack guard that the JIT can
sometimes elide.

Bounded context (DDD): pep_bench/pep654_except_star.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = list(range(N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        try:
            acc = acc + x
        except ValueError:
            acc = acc - 1
_t1 = time.perf_counter()

print("try_except_no_raise_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per inner pass: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
