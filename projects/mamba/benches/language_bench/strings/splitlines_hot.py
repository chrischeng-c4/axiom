"""str.splitlines — multi-newline split perf bench.

End-user scenario: `content.splitlines()` inside a tight loop, the
canonical text-to-lines primitive that backs every log-file line
iterator / multi-line config parser / paste-buffer line splitter /
markdown paragraph extractor. CPython routes through
unicode_splitlines (C-level scan across \\n, \\r, \\r\\n + new-list
build of new-str slices); mamba's str should hit a native impl
through its typed bridge.

Distinct from `split_hot.py` (single-delimiter split — needs an
explicit `sep` arg). splitlines is universal-newline-aware and
handles a fixed set of recognized line-end sequences in one pass.

Bounded context (DDD): language_bench/strings.

Tier: compute (with new-list + per-line new-str alloc per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `splitlines` is a str method; DO NOT hoist `_sl = TEXT.splitlines`
— bound-method hoist returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

TEXT = ("2026-05-27T10:14:33Z INFO request received\n"
        "2026-05-27T10:14:33Z INFO handler dispatched\n"
        "2026-05-27T10:14:34Z WARN downstream slow\n"
        "2026-05-27T10:14:34Z INFO response sent\n"
        "2026-05-27T10:14:35Z DEBUG metric recorded\n"
        "2026-05-27T10:14:35Z INFO request received\n"
        "2026-05-27T10:14:36Z ERROR retry exhausted\n"
        "2026-05-27T10:14:36Z INFO fallback triggered")
ITERS = 20000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    lines = TEXT.splitlines()
    acc = acc + len(lines)
_t1 = time.perf_counter()

print("splitlines_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = len(TEXT.splitlines())
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
