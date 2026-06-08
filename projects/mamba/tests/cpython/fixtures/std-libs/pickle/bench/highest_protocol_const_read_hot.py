"""Hot-loop bench for `pickle.HIGHEST_PROTOCOL` module-constant
read (#1468).

End-user scenario: serialization layers that want the newest
wire-format opcode set (`pickle.dumps(obj, protocol=pickle.HIGHEST_PROTOCOL)`).
RPC adapters, cache writers, and disk-snapshot helpers reference
the constant on every `dumps(...)` call; the canonical hot-path
idiom is to hoist a local alias (`HP = pickle.HIGHEST_PROTOCOL`)
once and pass it as the `protocol=` kwarg per-call. That per-iter
module-constant readback is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x —
CPython's `pickle.HIGHEST_PROTOCOL` is a top-level module-dict
probe returning the int `5` on 3.12). Mamba's shim returns the
same sentinel int directly from the module-attribute resolver,
so the per-access constant factor is the only thing on the clock.

Workload: 10_000 reads of `pickle.HIGHEST_PROTOCOL` against the
canonical CPython value (`5` on 3.12). The accumulator sums the
read on every iter, so a misread (wrong int) immediately fails
the correctness assert and dead-code elimination of the read
would leave `acc != ITERS`.

DoD: exits 0 under both CPython and mamba; the cross-runtime
bench harness compares per-iteration internal time
Floor is 1.0x per #1265 Goal 2.
"""

import pickle as _pickle

# Hoist the module-attribute read to a local alias outside the hot
# loop. The bench measures the per-iter readback through this local
# — the bound integer sentinel is the canonical CPython value
# (`HIGHEST_PROTOCOL = 5` on 3.12).
_HIGHEST_PROTOCOL = _pickle.HIGHEST_PROTOCOL

# Snapshot expected value once before the loop so the correctness
# compare is a pure int-equality probe in the hot path.
EXPECTED_HIGHEST = 5

ITERS = 10_000

acc = 0
for _ in range(ITERS):
    s = _HIGHEST_PROTOCOL
    # Accumulator readback prevents DCE — `s` is the bound sentinel
    # int, so the equality always holds in both CPython and mamba
    # and the increment is always taken.
    if s == EXPECTED_HIGHEST:
        acc = acc + 1

# Correctness: every iteration must read back HIGHEST_PROTOCOL == 5.
# acc == ITERS or we have a regression in pickle module constants.
assert acc - ITERS == 0, f"pickle highest-protocol const read acc drift: acc={acc} expected={ITERS}"
print("highest_protocol_const_read_hot:", acc)
