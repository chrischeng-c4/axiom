# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_contextlib_array_queue_struct_silent"
# subject = "cpython321.lang_contextlib_array_queue_struct_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_contextlib_array_queue_struct_silent.py"
# status = "filled"
# ///
"""cpython321.lang_contextlib_array_queue_struct_silent: execute CPython 3.12 seed lang_contextlib_array_queue_struct_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# context-manager / typed-array / priority-queue / binary-pack
# quartet pinned by atomic 149: `contextlib` (the documented
# suppress / nullcontext / ExitStack / AbstractContextManager
# surface — bare class identity + functional suppression of the
# named exception), `array` (the documented array.array typed-
# sequence — class identity + iteration + index/count contract),
# `queue` (the PriorityQueue lowest-first ordering + Queue /
# LifoQueue / PriorityQueue / Empty / Full class identity), and
# `struct` (the Struct prebuilt-format class identity +
# Struct().pack documented method).
#
# The matching subset (heapq.heapify / heappush / heappop /
# heappushpop / heapreplace / nlargest / nsmallest / merge,
# bisect.bisect / bisect_left / bisect_right / insort / insort_left
# / insort_right, queue.Queue / LifoQueue value ops,
# struct.pack / unpack / calcsize / error class identity) is
# covered by `test_heapq_bisect_queue_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • contextlib.suppress.__name__ == "suppress" — bare class
#     identity (mamba: contextlib.suppress is a `function`, not a
#     class — `<function <lambda>>` with no `__name__` matching);
#   • contextlib.suppress(ValueError) suppresses raised ValueError
#     (mamba: NOT actually suppressed — exception propagates);
#   • contextlib.nullcontext.__name__ == "nullcontext" — class
#     identity (mamba: function);
#   • contextlib.ExitStack.__name__ == "ExitStack" — class
#     identity (mamba: AttributeError);
#   • contextlib.AbstractContextManager.__name__ ==
#     "AbstractContextManager" — abstract-base class identity
#     (mamba: AttributeError);
#   • array.array.__name__ == "array" — typed-array class
#     identity (mamba: returns None);
#   • list(array.array("i", [1, 2, 3])) == [1, 2, 3] —
#     iteration over typed-array elements (mamba: returns [], the
#     array is not iterable);
#   • array.array("i", [1, 2, 3]).index(2) == 1 (mamba: returns
#     None);
#   • array.array("i", [1, 1, 2]).count(1) == 2 (mamba: returns
#     0);
#   • queue.Queue.__name__ == "Queue" — FIFO class identity
#     (mamba: None);
#   • queue.LifoQueue.__name__ == "LifoQueue" (mamba: None);
#   • queue.PriorityQueue.__name__ == "PriorityQueue" (mamba:
#     None);
#   • queue.Empty.__name__ == "Empty" (mamba: None);
#   • queue.Full.__name__ == "Full" (mamba: None);
#   • PriorityQueue lowest-tuple-first ordering — put (2,"b"),
#     (1,"a"), (3,"c"); get returns (1,"a") (mamba: returns
#     (2,"b"), insertion order — heap invariant not enforced);
#   • struct.Struct.__name__ == "Struct" — prebuilt-format class
#     identity (mamba: None);
#   • struct.Struct(">ii").pack(1, 2) == b"\x00\x00\x00\x01..."
#     — prebuilt-format pack method (mamba: AttributeError,
#     'Struct' object has no attribute 'pack').
import contextlib as _contextlib_mod
import array as _array_mod
import queue as _queue_mod
import struct as _struct_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance methods that mamba's bundled type
# stubs do not surface accurately.
contextlib: Any = _contextlib_mod
array: Any = _array_mod
queue: Any = _queue_mod
struct: Any = _struct_mod


_ledger: list[int] = []

# 1) contextlib.suppress — bare class identity
assert contextlib.suppress.__name__ == "suppress"; _ledger.append(1)

# 2) contextlib.suppress — functionally suppresses named exception
_suppressed = True
try:
    with contextlib.suppress(ValueError):
        raise ValueError("x")
except ValueError:
    _suppressed = False
assert _suppressed == True; _ledger.append(1)

# 3) contextlib.nullcontext / ExitStack / AbstractContextManager — class identity
assert contextlib.nullcontext.__name__ == "nullcontext"; _ledger.append(1)
assert contextlib.ExitStack.__name__ == "ExitStack"; _ledger.append(1)
assert contextlib.AbstractContextManager.__name__ == "AbstractContextManager"; _ledger.append(1)

# 4) array.array — class identity
assert array.array.__name__ == "array"; _ledger.append(1)

# 5) array.array — iteration over typed sequence
_arr = array.array("i", [1, 2, 3])
assert list(_arr) == [1, 2, 3]; _ledger.append(1)

# 6) array.array — index / count methods
assert array.array("i", [1, 2, 3]).index(2) == 1; _ledger.append(1)
assert array.array("i", [1, 1, 2]).count(1) == 2; _ledger.append(1)

# 7) queue.* — class identity
assert queue.Queue.__name__ == "Queue"; _ledger.append(1)
assert queue.LifoQueue.__name__ == "LifoQueue"; _ledger.append(1)
assert queue.PriorityQueue.__name__ == "PriorityQueue"; _ledger.append(1)
assert queue.Empty.__name__ == "Empty"; _ledger.append(1)
assert queue.Full.__name__ == "Full"; _ledger.append(1)

# 8) queue.PriorityQueue — lowest-first ordering
_pq = queue.PriorityQueue()
_pq.put((2, "b"))
_pq.put((1, "a"))
_pq.put((3, "c"))
assert _pq.get() == (1, "a"); _ledger.append(1)

# 9) struct.Struct — class identity
assert struct.Struct.__name__ == "Struct"; _ledger.append(1)

# 10) struct.Struct — prebuilt-format pack method
_s = struct.Struct(">ii")
assert _s.pack(1, 2) == b"\x00\x00\x00\x01\x00\x00\x00\x02"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_contextlib_array_queue_struct_silent {sum(_ledger)} asserts")
