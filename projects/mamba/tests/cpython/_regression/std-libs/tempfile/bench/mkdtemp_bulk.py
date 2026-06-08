"""Bulk tempfile.mkdtemp + os.rmdir cycle (Task #71, Wave-6 ship #4).

Predicted regime per scout: balanced (syscall-dominated — mkdir +
rmdir + open). Wall target >=1.5x. CPython tempfile.mkdtemp routes
through Python's full tempfile machinery (template + suffix + prefix
+ os.urandom-driven name + retry loop on EEXIST). Mamba uses a
process-pid-prefixed counter so name generation is constant-time;
the actual mkdir/rmdir syscalls dominate but the per-call Python-side
work in CPython adds ~25-40us of pure interpreter overhead per
mkdtemp.

Workload: 100 mkdtemp+rmdir pairs × 10 iters. Each pair creates a
new subdirectory under sys-temp and removes it immediately.

Per scout sequencing: tempfile is extending existing 4-entry surface
to 6-entry (#1462); this fixture pairs with tempfile_mod.rs
registering gettempdir/gettempprefix/mkdtemp/mkstemp/
NamedTemporaryFile/TemporaryDirectory dispatchers at the same
revision. No class.rs changes needed (no integer handles — every
result is a path string or tuple).

Hoist convention (#2097): bind `tempfile.mkdtemp` and `os.rmdir`
locally to avoid per-iter module-attr lookup.
Mamba import quirk avoidance: separate `import sys` / `import time` /
`import tempfile` / `import os` lines.

# tier: balanced
"""

import tempfile
import os

_mkdtemp = tempfile.mkdtemp
_rmdir = os.rmdir

ITERS = 10
N = 100

acc = 0
TOTAL = ITERS * N
for _ in range(TOTAL):
    p = _mkdtemp()
    _rmdir(p)
    acc = acc + 1
print("mkdtemp_bulk:", acc)
