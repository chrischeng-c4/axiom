"""PEP 498 f-string — mixed-type interpolation perf bench.

End-user scenario: `f"[{ts:.3f}] {level:<5} {name}={count:>6d}"` inside a
tight loop, the canonical mixed-type interpolation primitive that backs
every structured log-line emitter / metrics-line formatter / status-row
printer / debug-trace builder. CPython routes through BUILD_STRING +
FORMAT_VALUE with per-conversion format-spec dispatch (str.__format__,
float.__format__, int.__format__); mamba should hit a native impl
through its typed bridge.

Distinct from `fstring_int_hot.py` which interpolates a single int
(only int.__format__ exercised). Mixed types stress the FORMAT_VALUE
opcode dispatch and per-type __format__ lookup.

Bounded context (DDD): pep_bench/pep498_fstrings.

Tier: compute (with per-call new-str alloc + multi-type format-spec).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: f-strings are syntax, not method calls — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

LEVELS = ("INFO", "WARN", "DEBUG", "ERROR", "TRACE")
NAMES = ("rps", "latency_ms", "errors", "queue_depth", "qps")
ITERS = 10000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(5):
        ts = 1716798000.123 + float(i) * 0.001
        line = f"[{ts:.3f}] {LEVELS[i]:<5} {NAMES[i]}={i * 100:>6d}"
        s = s + len(line)
    acc = acc + s
_t1 = time.perf_counter()

print("fstring_mixed_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(5):
    ts = 1716798000.123 + float(i) * 0.001
    per_iter = per_iter + len(f"[{ts:.3f}] {LEVELS[i]:<5} {NAMES[i]}={i * 100:>6d}")
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
