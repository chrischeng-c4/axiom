# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_bisect_array_inspect_value_ops"
# subject = "cpython321.test_heapq_bisect_array_inspect_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_bisect_array_inspect_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_bisect_array_inspect_value_ops: execute CPython 3.12 seed test_heapq_bisect_array_inspect_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib / language surfaces used by every priority-
# queue / sorted-search / typed-buffer / introspection path:
# `heapq` (the documented `heappush` / `heappop` / `heapify` /
# `nlargest` / `nsmallest` / `merge` / `heappushpop` /
# `heapreplace` min-heap helpers — fixed against canonical
# integer-ranking sequences), `bisect` (the documented
# `bisect_left` / `bisect_right` / `insort` / `insort_left` /
# `insort_right` sorted-search and ordered-insert surface),
# `array` (the documented `array.typecode` attribute +
# top-level `array` class identifier), and `inspect` (the
# documented `signature` / `isfunction` / `isclass` /
# `ismethod` attribute surface).
#
# The matching subset between mamba and CPython is the heap
# layer (push / pop / heapify / nlargest / nsmallest / merge /
# heappushpop / heapreplace), the sorted-search layer
# (bisect_left / bisect_right / insort + variants), the
# array-typecode attribute layer (constructor-side divergences
# move to spec), and the inspect attribute hasattr layer
# (predicate-call divergences move to spec).
#
# Surface in this fixture:
#   • heapq — full helper surface against [3,1,2] push order,
#     `heapify([5,1,3,7,2])`, nlargest / nsmallest of integer
#     iterable, two-iterable merge, heappushpop + heapreplace;
#   • bisect — bisect_left/right against existing key,
#     bisect_left/right against absent key, insort + insort_left
#     + insort_right ordered-insert chain;
#   • array — `array.array` class identifier + typecode
#     attribute on int + float arrays;
#   • inspect — signature / isfunction / isclass / ismethod
#     attribute surface.
#
# Behavioral edges that DIVERGE on mamba (array.array(typecode,
# iterable) constructor returns empty array, list(arr) returns
# [], len(arr) returns 0, arr.append / arr.extend / arr.reverse
# / arr.index all broken, inspect.getfullargspec absent,
# inspect.isbuiltin absent, inspect.isfunction predicate
# inverted — returns False on `def` functions and True on
# integers — inspect.isclass(int) returns False,
# inspect.signature(fn) returns empty parameter map + str
# representation "()" — parameters are silently dropped) are
# covered in the matching spec fixture
# `lang_array_inspect_silent`.
import heapq
import bisect
import array
import inspect


_ledger: list[int] = []


def _free_fn(a: int, b: int = 2, *args, **kw) -> int:
    return a + b


# 1) heapq — push / pop / heapify
_h: list[int] = []
heapq.heappush(_h, 3)
heapq.heappush(_h, 1)
heapq.heappush(_h, 2)
assert _h == [1, 3, 2]; _ledger.append(1)
assert heapq.heappop(_h) == 1; _ledger.append(1)
assert _h == [2, 3]; _ledger.append(1)

_arr = [5, 1, 3, 7, 2]
heapq.heapify(_arr)
assert _arr == [1, 2, 3, 7, 5]; _ledger.append(1)

# 2) heapq — nlargest / nsmallest / merge
assert heapq.nlargest(3, [1, 5, 2, 8, 3, 9]) == [9, 8, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [5, 1, 9, 3, 7, 2]) == [1, 2, 3]; _ledger.append(1)
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# 3) heapq — heappushpop / heapreplace
assert heapq.heappushpop([1, 2, 3], 0) == 0; _ledger.append(1)
assert heapq.heapreplace([1, 2, 3], 5) == 1; _ledger.append(1)

# 4) bisect — sorted-search against existing key
_sa = [1, 3, 5, 7, 9]
assert bisect.bisect_left(_sa, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(_sa, 5) == 3; _ledger.append(1)

# 5) bisect — sorted-search against absent key
assert bisect.bisect_left(_sa, 4) == 2; _ledger.append(1)
assert bisect.bisect_right(_sa, 4) == 2; _ledger.append(1)

# 6) bisect — ordered-insert chain
_sb = [1, 3, 5, 7]
bisect.insort(_sb, 4)
assert _sb == [1, 3, 4, 5, 7]; _ledger.append(1)
bisect.insort_left(_sb, 5)
assert _sb == [1, 3, 4, 5, 5, 7]; _ledger.append(1)
bisect.insort_right(_sb, 5)
assert _sb == [1, 3, 4, 5, 5, 5, 7]; _ledger.append(1)

# 7) bisect — module attribute surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 8) array — module attribute + typecode property
assert hasattr(array, "array") == True; _ledger.append(1)
_ai = array.array("i", [1, 2, 3, 4])
assert _ai.typecode == "i"; _ledger.append(1)
_af = array.array("f", [1.0, 2.0, 3.0])
assert _af.typecode == "f"; _ledger.append(1)

# 9) inspect — attribute hasattr surface
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)

# NB: array.array(typecode, iterable) constructor returns empty
# array on mamba, list(arr) returns [], len(arr) returns 0,
# arr.append / arr.extend / arr.reverse / arr.index all broken,
# inspect.getfullargspec absent, inspect.isbuiltin absent,
# inspect.isfunction predicate inverted — returns False on
# `def` functions and True on integers — inspect.isclass(int)
# returns False, inspect.signature(fn) returns empty parameter
# map + str representation "()" all DIVERGE on mamba — moved
# to the divergence-spec fixture.

# Suppress unused-symbol diagnostic — `_free_fn` lives only to
# serve as the inspect-predicate target in the matching spec
# fixture, which imports the same surface but probes it.
_ = _free_fn

print(f"MAMBA_ASSERTION_PASS: test_heapq_bisect_array_inspect_value_ops {sum(_ledger)} asserts")
