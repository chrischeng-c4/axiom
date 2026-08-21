"""chr/ord — codepoint conversion builtin perf bench.

End-user scenario: `ord(ch)` over each char of a string + `chr(code)`
build loop, the canonical character-codepoint conversion that backs
every byte-stuffing encoder / Caesar-shift toy / hash-of-string
fingerprint / lex token emit. CPython routes through PyLong_FromLong
+ PyUnicode_FromOrdinal (C-level); mamba's builtins should hit a
native impl through its typed bridge.

Bounded context (DDD): builtins_bench/chr_ord.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: chr and ord are builtins; no module-attr hoisting needed.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEXT = "The quick brown fox jumps over the lazy dog 0123456789"
N = len(TEXT)
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for ch in TEXT:
        s = s + ord(ch)
    for code in range(48, 58):
        s = s + ord(chr(code))
    acc = acc + s
_t1 = time.perf_counter()

print("chr_ord_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for ch in TEXT:
    per_iter = per_iter + ord(ch)
for code in range(48, 58):
    per_iter = per_iter + ord(chr(code))
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
