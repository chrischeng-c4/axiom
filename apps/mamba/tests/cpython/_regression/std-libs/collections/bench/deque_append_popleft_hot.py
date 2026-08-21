"""Hot-loop bench for `collections.deque` append/popleft (#1449).

End-user scenario: a sliding-window aggregator (rolling sum, recent-events
buffer, ring queue) does N append + N popleft cycles per tick at high
frequency. The interesting cost is the per-call dispatcher overhead and
the underlying VecDeque push/pop primitives.

Tier: `heap container-light` (target mamba/cpython <= 1.0x — CPython's
deque is a doubly-linked block list in C; mamba's edge is a thin
dispatcher over Rust's VecDeque, which gives O(1) on both ends without
the per-block bookkeeping).

Note on hoisting: mamba's current method-binding path returns None when
`deque.popleft` is hoisted to a local (`pl = d.popleft; pl()` → None).
Direct attribute access (`d.popleft()`) returns the correct value. We
deliberately call `d.append(...)` / `d.popleft()` per-iter — this
matches the typical user pattern (no manual hoist), and the per-call
attribute-lookup overhead is the same on both runtimes so the published
ratio still reflects the underlying primitive cost.

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
and reports the ratio. Floor is 1.0x per #1265 Goal 2.
"""

import collections

ITERS = 10_000

d = collections.deque()

# Touched-sum readback so the loop cannot be DCE'd.
acc = 0
for i in range(ITERS):
    d.append(i)
    acc = acc + d.popleft()

expected = ITERS * (ITERS - 1) // 2
assert acc - expected == 0, f"deque append/popleft drift: acc={acc} expected={expected}"
print("deque_append_popleft_hot:", acc)
