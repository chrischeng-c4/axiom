"""Bulk pickle.dumps + pickle.loads roundtrip (Task #55, Wave-4 ship #3).

Predicted regime per scout: compute (allocation-bound at scale).
Mamba's pickle shim emits a non-CPython text format (S5:hello, I42,
L3;...) so output bytes are NOT byte-equivalent to CPython's protocol.
G1 cannot assert byte-equality between runtimes; instead each runtime
roundtrips dumps -> loads and asserts the reconstructed list-of-int
preserves the sum, then prints the sum as the conformance token.

Workload: 100-element list[int], 1000 iters = 100k pickle/unpickle
of small values. Same realistic-CSV regime — small payload per call,
many calls. Per scout, expect mem 0.3-0.5x FAIL by-design (subset B
per-call serialization buffer + per-call loaded-list Instance
allocation).

No __reduce__ / __reduce_ex__ exercised (#2100 carve-out — built-in
int+list only). No PickleBuffer / out-of-band buffers. No
persistent_id. Protocol 4 only.

Hoist convention (#2097): bind `pickle.dumps` and `pickle.loads`
locally before the loop.

# tier: compute
"""

import pickle

_dumps = pickle.dumps
_loads = pickle.loads

DATA = list(range(100))
ITERS = 1_000

acc = 0
for _ in range(ITERS):
    blob = _dumps(DATA)
    restored = _loads(blob)
    acc += len(restored)
print("dumps_loads_roundtrip:", acc)
