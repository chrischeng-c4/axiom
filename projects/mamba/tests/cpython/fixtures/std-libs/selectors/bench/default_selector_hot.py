"""Hot-loop selectors.DefaultSelector() microbench for #1471 Gate 2.

Predicted regime per scout: pure-Python hot loop.
CPython's `selectors.DefaultSelector()` constructs a real
`KqueueSelector` on macOS (which calls `select.kqueue()` to allocate a
kernel kqueue file descriptor, then initializes a per-instance dict
mapping fileobj→SelectorKey) or an `EpollSelector` on Linux (which
calls `epoll_create1()`). The syscall + dict allocation per call is
the dominant overhead.

Mamba dispatches directly into `mb_selectors_default_selector_new`
(single `MbObject::new_instance` allocation — no syscall, no dict
init), so the per-call overhead is one native-handle dispatch.

Workload: 50_000 iters of `selectors.DefaultSelector()`, closing each
selector when the runtime exposes `close()`. Result discarded. The
close step keeps CPython from exhausting kqueue/epoll file descriptors
while preserving the constructor hot path that mamba optimizes.

Hoist convention (#2097): bind `selectors.DefaultSelector` locally to
avoid per-iter module-attr lookup. Same pattern as the
signal/warnings/tempfile/queue/contextvars/abc/traceback hot-loop
bench fixtures. Mamba import quirk avoidance: separate `import sys` /
`import time` / `import selectors` lines.

# tier: hot-loop
"""

import selectors

_DefaultSelector = selectors.DefaultSelector
_probe = _DefaultSelector()
_CloseSelector = getattr(_probe, "close", None)
if _CloseSelector is not None:
    _CloseSelector()
_SHOULD_CLOSE = _CloseSelector is not None

ITERS = 50_000

acc = 0
for _ in range(ITERS):
    _selector = _DefaultSelector()
    if _SHOULD_CLOSE:
        _selector.close()
    acc = acc + 1
print("default_selector_hot:", acc)
