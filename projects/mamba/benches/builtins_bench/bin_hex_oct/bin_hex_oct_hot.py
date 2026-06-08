"""bin / hex / oct — int-to-radix-string builtin perf bench.

End-user scenario: `f"0x{flags:x}"`-style debug-dump for register-style
ints inside a tight loop, the canonical int-to-radix primitive that
backs every memory-address pretty-printer / bitmask dumper / file-mode
formatter / hex-color string builder. CPython routes through
long_to_decimal_string (and its base-2/8/16 cousins) — C-level
divmod-by-radix into a new PyUnicode; mamba's int-radix conversion
should hit a native impl through its typed bridge.

This bench exercises all three radix builtins per iter to capture
their relative parity (cpython has them share an inner loop with a
base-parameterized divmod; mamba's coverage may differ).

Bounded context (DDD): builtins_bench/bin_hex_oct.

Tier: compute (with per-call new-str alloc).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `bin`/`hex`/`oct` are top-level builtins, NOT module-attr
lookups; safe to use directly (they're never bound methods).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

VALS = (0, 1, 255, 1024, 65535, 1048576, 16777215, 2147483647,
        7, 42, 1000000, 999999999, 8, 16, 64, 4096)
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for v in VALS:
        s = s + len(bin(v)) + len(hex(v)) + len(oct(v))
    acc = acc + s
_t1 = time.perf_counter()

print("bin_hex_oct_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for v in VALS:
    per_iter = per_iter + len(bin(v)) + len(hex(v)) + len(oct(v))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
