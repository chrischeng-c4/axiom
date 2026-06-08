# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_bisect_queue_value_ops"
# subject = "cpython321.test_heapq_bisect_queue_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_bisect_queue_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_bisect_queue_value_ops: execute CPython 3.12 seed test_heapq_bisect_queue_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules used by every sort / priority / search
# / FIFO-LIFO path: `heapq` (the documented min-heap primitives —
# heapify / heappush / heappop / heappushpop / heapreplace /
# nlargest / nsmallest / merge), `bisect` (the binary-search +
# in-place insertion primitives — bisect / bisect_left /
# bisect_right / insort / insort_left / insort_right), and `queue`
# (the documented FIFO Queue + LIFO Queue with put / get / qsize /
# empty value ops; PriorityQueue ordering is covered in the
# divergence-spec fixture).
#
# The matching subset between mamba and CPython is the heap-
# ordering layer + binary-search layer + thread-safe FIFO/LIFO
# layer: heapify produces a min-heap; heappush / heappop maintain
# the heap invariant; heappushpop returns the smaller of pushed-
# or-root; heapreplace returns the root and replaces it; nlargest
# / nsmallest return sorted-by-rank K-element lists; merge yields
# a sorted iterator; bisect / bisect_left / bisect_right find
# insertion indices in sorted lists; insort variants insert in-
# place at the right index; queue.Queue is FIFO with put/get/qsize
# /empty value contract; queue.LifoQueue reverses order; struct
# pack/unpack/calcsize round-trips signed-int formats with
# big-endian byte order.
#
# Surface in this fixture:
#   • heapq.heapify, heappush, heappop, heappushpop, heapreplace;
#   • heapq.nlargest, nsmallest, merge;
#   • bisect.bisect, bisect_left, bisect_right;
#   • bisect.insort, insort_left, insort_right;
#   • queue.Queue — put/get/qsize/empty FIFO contract;
#   • queue.LifoQueue — put/get LIFO ordering;
#   • struct.pack / unpack — big-endian signed-int round-trip;
#   • struct.calcsize — format-string size for >i / >d / >h;
#   • struct.error — exception class identity.
#
# Behavioral edges that DIVERGE on mamba (contextlib.suppress not
# actually suppressing exceptions, contextlib class identity,
# array.array class identity + iteration broken,
# queue.PriorityQueue ordering inverted, queue class identity,
# struct.Struct.pack missing) are covered in the matching spec
# fixture `lang_contextlib_array_queue_struct_silent`.
import heapq
import bisect
import queue
import struct


_ledger: list[int] = []

# 1) heapq.heapify — min-heap invariant
_h = [5, 2, 8, 1, 9]
heapq.heapify(_h)
assert _h[0] == 1; _ledger.append(1)

# 2) heapq.heappush — invariant maintained on push
heapq.heappush(_h, 0)
assert _h[0] == 0; _ledger.append(1)

# 3) heapq.heappop — returns + removes root
assert heapq.heappop(_h) == 0; _ledger.append(1)
assert heapq.heappop(_h) == 1; _ledger.append(1)

# 4) heapq.nlargest / nsmallest
assert heapq.nlargest(3, [4, 1, 7, 3, 5, 2]) == [7, 5, 4]; _ledger.append(1)
assert heapq.nsmallest(3, [4, 1, 7, 3, 5, 2]) == [1, 2, 3]; _ledger.append(1)

# 5) heapq.merge — sorted iterator
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# 6) heapq.heappushpop / heapreplace
assert heapq.heappushpop([1, 3, 5], 2) == 1; _ledger.append(1)
assert heapq.heapreplace([1, 3, 5], 4) == 1; _ledger.append(1)

# 7) bisect — search indices on sorted list
_a = [1, 3, 5, 7, 9]
assert bisect.bisect(_a, 4) == 2; _ledger.append(1)
assert bisect.bisect_left(_a, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(_a, 5) == 3; _ledger.append(1)

# 8) bisect.insort — in-place insertion
_b = [1, 3, 5]
bisect.insort(_b, 4)
assert _b == [1, 3, 4, 5]; _ledger.append(1)

_c = [1, 3, 5]
bisect.insort_left(_c, 3)
assert _c == [1, 3, 3, 5]; _ledger.append(1)

_d = [1, 3, 5]
bisect.insort_right(_d, 3)
assert _d == [1, 3, 3, 5]; _ledger.append(1)

# 9) queue.Queue — FIFO put/get/qsize/empty
_q = queue.Queue()
_q.put(1); _q.put(2); _q.put(3)
assert _q.qsize() == 3; _ledger.append(1)
assert _q.get() == 1; _ledger.append(1)
assert _q.get() == 2; _ledger.append(1)
assert _q.empty() == False; _ledger.append(1)
assert _q.get() == 3; _ledger.append(1)
assert _q.empty() == True; _ledger.append(1)

# 10) queue.LifoQueue — LIFO ordering
_lq = queue.LifoQueue()
_lq.put(1); _lq.put(2)
assert _lq.get() == 2; _ledger.append(1)
assert _lq.get() == 1; _ledger.append(1)

# 11) struct.pack / unpack — big-endian int round-trip
_packed = struct.pack(">i", 42)
assert _packed == b"\x00\x00\x00*"; _ledger.append(1)
assert struct.unpack(">i", _packed) == (42,); _ledger.append(1)

# 12) struct.calcsize — format-string sizes
assert struct.calcsize(">i") == 4; _ledger.append(1)
assert struct.calcsize(">d") == 8; _ledger.append(1)
assert struct.calcsize(">h") == 2; _ledger.append(1)

# 13) struct.error — exception class identity
assert struct.error.__name__ == "error"; _ledger.append(1)

# 14) hasattr surface — module-level helpers
assert hasattr(heapq, "heapify"); _ledger.append(1)
assert hasattr(heapq, "heappush"); _ledger.append(1)
assert hasattr(heapq, "heappop"); _ledger.append(1)
assert hasattr(bisect, "bisect"); _ledger.append(1)
assert hasattr(bisect, "insort"); _ledger.append(1)
assert hasattr(queue, "Queue"); _ledger.append(1)
assert hasattr(queue, "LifoQueue"); _ledger.append(1)
assert hasattr(struct, "pack"); _ledger.append(1)
assert hasattr(struct, "unpack"); _ledger.append(1)
assert hasattr(struct, "calcsize"); _ledger.append(1)

# NB: contextlib.suppress not actually suppressing, contextlib
# class identity, array.array class identity + iteration broken,
# queue.PriorityQueue ordering inverted, queue class identity,
# struct.Struct.pack missing all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_heapq_bisect_queue_value_ops {sum(_ledger)} asserts")
