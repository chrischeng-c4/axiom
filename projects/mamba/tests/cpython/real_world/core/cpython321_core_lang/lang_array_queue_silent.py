# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_array_queue_silent"
# subject = "cpython321.lang_array_queue_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_array_queue_silent.py"
# status = "filled"
# ///
"""cpython321.lang_array_queue_silent: execute CPython 3.12 seed lang_array_queue_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `type(array.array('i', [...]))
# .__name__` (the documented "array.array is the array class — type
# reports 'array'" — mamba returns 'int'), `len(array.array('i',
# [...]))` (the documented "len of an array is its element count" —
# mamba returns 0), `array.array('i', [...])[0]` (the documented
# "indexing an array returns the stored element" — mamba returns
# None), `list(array.array('i', [...]))` (the documented "array is
# iterable; list() materializes the stored elements" — mamba raises
# TypeError 'object is not iterable'), `hasattr(array_instance,
# 'tolist')` (the documented "array exposes a tolist() method" —
# mamba returns False), `hasattr(array_instance, 'count')` (the
# documented "array exposes a count() method" — mamba returns False),
# `queue.Queue().get_nowait()` (the documented "get_nowait on an
# empty queue raises queue.Empty" — mamba silently returns 0),
# `queue.Queue(maxsize=1).put_nowait(...)` after a full put (the
# documented "put_nowait on a full queue raises queue.Full" — mamba
# silently accepts), `queue.Queue().maxsize` (the documented "the
# default maxsize is 0 — unbounded" — mamba returns None), and
# `queue.Queue(maxsize=2).full()` after two puts (the documented
# "full() returns True when qsize equals maxsize" — mamba returns
# False — maxsize is not enforced).
# Ten-pack pinned to atomic 259.
#
# Behavioral edges that CONFORM on mamba (bisect — hasattr bisect/
# bisect_left/bisect_right/insort/insort_left/insort_right + bisect
# on sorted list, bisect_left vs bisect_right on duplicate, bisect of
# empty list, all-equal list, insort and insort_left mutate in-place.
# heapq — hasattr heappush/heappop/heapify/heappushpop/heapreplace/
# nlargest/nsmallest/merge + heappush root index, heap-pop three (via
# list-eq), sort-via-heap, nlargest/nsmallest, heapq.merge. array —
# hasattr array/typecodes + typecodes string 'bBuhHiIlLqQfd', array
# 'i' typecode and itemsize 4. queue — hasattr Queue/LifoQueue/
# PriorityQueue/Empty/Full + Queue FIFO/LIFO/Priority orderings,
# Queue().empty() True on fresh queue, Queue.qsize via list-eq) are
# covered in the matching pass fixture
# `test_bisect_heapq_array_queue_value_ops`.
import array
import queue
from typing import Any


_ledger: list[int] = []

# 1) type(array.array('i', [...])).__name__ == 'array'
#    (mamba: returns 'int')
assert type(array.array("i", [1, 2, 3])).__name__ == "array"; _ledger.append(1)

# 2) len(array.array('i', [...])) == element count
#    (mamba: returns 0)
assert len(array.array("i", [1, 2, 3])) == 3; _ledger.append(1)

# 3) array[0] returns the first stored element
#    (mamba: returns None)
def _arr_idx() -> Any:
    return array.array("i", [1, 2, 3])[0]
assert _arr_idx() == 1; _ledger.append(1)

# 4) list(array.array(...)) materializes stored elements
#    (mamba: raises TypeError 'object is not iterable')
def _arr_to_list() -> list:
    try:
        return list(array.array("i", [1, 2, 3]))
    except TypeError:
        return []
assert _arr_to_list() == [1, 2, 3]; _ledger.append(1)

# 5) hasattr(array_instance, 'tolist') — array has tolist()
#    (mamba: returns False)
assert hasattr(array.array("i", [1, 2, 3]), "tolist") == True; _ledger.append(1)

# 6) hasattr(array_instance, 'count') — array has count()
#    (mamba: returns False)
assert hasattr(array.array("i", [1, 2, 3]), "count") == True; _ledger.append(1)

# 7) queue.Queue().get_nowait() raises queue.Empty on empty queue
#    (mamba: silently returns 0 — no exception)
def _get_raises_empty() -> str:
    q = queue.Queue()
    try:
        q.get_nowait()
        return "silent"
    except queue.Empty:
        return "raised"
assert _get_raises_empty() == "raised"; _ledger.append(1)

# 8) queue.Queue(maxsize=1).put_nowait raises queue.Full when full
#    (mamba: silently accepts — no exception)
def _put_raises_full() -> str:
    q = queue.Queue(maxsize=1)
    q.put(1)
    try:
        q.put_nowait(2)
        return "silent"
    except queue.Full:
        return "raised"
assert _put_raises_full() == "raised"; _ledger.append(1)

# 9) queue.Queue().maxsize == 0 — default unbounded
#    (mamba: returns None)
assert queue.Queue().maxsize == 0; _ledger.append(1)

# 10) queue.Queue(maxsize=2).full() after two puts returns True
#     (mamba: returns False — maxsize not enforced)
def _full_at_max() -> Any:
    q = queue.Queue(maxsize=2)
    q.put(1); q.put(2)
    return q.full()
assert _full_at_max() == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_array_queue_silent {sum(_ledger)} asserts")
