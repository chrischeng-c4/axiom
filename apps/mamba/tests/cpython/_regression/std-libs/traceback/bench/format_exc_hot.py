"""Hot-loop traceback.format_exc microbench for #1441 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `traceback.format_exc()` is pure Python in `Lib/traceback.py` —
each call walks `sys.exc_info()`, threads through `TracebackException`
construction (which probes `__cause__`, `__context__`, suppresses chain
traversal when no exception is active, and finally falls through to the
hard-coded `"NoneType: None\\n"` line). Even on the no-exception fast
path the per-call overhead is several dozen Python opcodes.

Mamba dispatches directly into `mb_traceback_format_exc` (single allocation
of the static `"NoneType: None\\n"` Str via `MbObject::new_str`), so the
per-call overhead is one heap alloc plus the native handle dispatch.

Workload: 200_000 iters of `traceback.format_exc()`. Result discarded.
The ratio favors mamba because CPython's per-call frame walk + chain
probe dwarfs the native string alloc.

Hoist convention (#2097): bind `traceback.format_exc` locally to avoid
per-iter module-attr lookup. Same pattern as
warnings/tempfile/queue/contextvars/abc hot-loop bench fixtures.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import traceback` lines.

# tier: hot-loop
"""

import sys
import traceback

_format_exc = traceback.format_exc

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    _format_exc()
    acc = acc + 1
print("format_exc_hot:", acc)
