"""Hot-loop tempfile.gettempdir microbench for #1462 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `tempfile.gettempdir()` is wrapped in `_lazy_init` + a
module-level lock + a tempdir cache; the cache hit still pays for a
Python function call + two attribute loads + a `try`/`finally`
acquire-release on the lock. Mamba dispatches directly into
`mb_tempfile_gettempdir` (native `std::env::temp_dir()` -> str), so
per-call overhead is the constant-time native handle dispatch.

Workload: 200_000 iters of `tempfile.gettempdir()`. Each call returns
a fresh path string in mamba (and a cached string in CPython); the
ratio still favors mamba because the per-call Python interpreter
overhead dwarfs the native dispatch.

Hoist convention (#2097): bind `tempfile.gettempdir` locally to avoid
per-iter module-attr lookup. This is the same pattern as the other
hot-loop bench fixtures (queue, glob, mimetypes).
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import tempfile` lines.

# tier: hot-loop
"""

import tempfile

_gettempdir = tempfile.gettempdir

ITERS = 200_000

acc = 0
for _ in range(ITERS):
    s = _gettempdir()
    acc = acc + 1
print("gettempdir_hot:", acc)
