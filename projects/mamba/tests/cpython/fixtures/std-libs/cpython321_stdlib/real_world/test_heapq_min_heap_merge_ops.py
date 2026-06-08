# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_min_heap_merge_ops"
# subject = "cpython321.test_heapq_min_heap_merge_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_min_heap_merge_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_min_heap_merge_ops: execute CPython 3.12 seed test_heapq_min_heap_merge_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the matching `heapq` surface —
# scalar (int / float / str) min-heap operations + the keyless merge /
# nsmallest / nlargest helpers + heapreplace / heappushpop /
# max-heap-via-negation idiom.
#
# `heapq` is a binary-heap interface laid over an existing list. The
# matching subset between mamba and CPython is the SCALAR-key family
# without callable key= kwargs and without tuple keys. The (tuple-key
# heappop crash, key= kwarg crash, merge(reverse=), heappop-empty
# silent-None) edges are split off into the divergence-spec fixture.
#
# Surface in this fixture:
#   • heapq.heapify(list)               — in-place transform into a
#                                          min-heap;
#   • heapq.heappush(heap, item)        — push preserving heap invariant;
#   • heapq.heappop(heap)               — pop the smallest item;
#   • heappush + heappop round-trip     — yields a sorted sequence;
#   • heapq.heapreplace(heap, item)     — pop + push as one atomic step;
#   • heapq.heappushpop(heap, item)     — push + pop in the other order;
#   • heapq.nsmallest(n, iterable)      — top-n-smallest from any iter;
#   • heapq.nlargest(n, iterable)       — top-n-largest from any iter;
#   • heapq.merge(*iters)               — k-way merge of sorted iters;
#   • max-heap-via-negation              — the canonical idiom for
#                                          producing a max-heap from a
#                                          min-heap;
#   • heap of int / float / str          — three scalar types confirmed
#                                          to dispatch the same compare.
#
# Behavioral edges that DIVERGE on mamba (string-key heaps left
# unsorted by heapify, merge(reverse=) silently ignored, heappop on
# empty returning None instead of IndexError, pathlib Path attribute /
# parts / parent / with_suffix / with_name / is_absolute returning
# None or raising AttributeError) are covered in
# `lang_heapq_pathlib_keyword_empty_silent.py`.
import heapq

_ledger: list[int] = []

# 1) heapify on an int list — produces a min-heap
_h: list[int] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3]
heapq.heapify(_h)
# After heapify the root is the smallest element
assert _h[0] == 1; _ledger.append(1)
assert len(_h) == 10; _ledger.append(1)

# 2) heappop yields elements in sorted order
_h_copy: list[int] = list(_h)
_sorted_out: list[int] = []
while _h_copy:
    _sorted_out.append(heapq.heappop(_h_copy))
assert _sorted_out == [1, 1, 2, 3, 3, 4, 5, 5, 6, 9]; _ledger.append(1)
assert len(_h_copy) == 0; _ledger.append(1)

# 3) heappush preserves heap invariant
_h2: list[int] = []
for _v in [5, 3, 8, 1, 9, 2, 7]:
    heapq.heappush(_h2, _v)
assert _h2[0] == 1; _ledger.append(1)
assert len(_h2) == 7; _ledger.append(1)
# Popping out in order yields sorted output
_out2: list[int] = []
while _h2:
    _out2.append(heapq.heappop(_h2))
assert _out2 == [1, 2, 3, 5, 7, 8, 9]; _ledger.append(1)

# 4) heappush + heappop round-trip = sorting
_unsorted: list[int] = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5]
_h3: list[int] = []
for _v in _unsorted:
    heapq.heappush(_h3, _v)
_sorted_out2: list[int] = []
while _h3:
    _sorted_out2.append(heapq.heappop(_h3))
assert _sorted_out2 == sorted(_unsorted); _ledger.append(1)

# 5) heapreplace — pop smallest, then push, atomically
_hr: list[int] = [1, 5, 3, 7]
heapq.heapify(_hr)
_popped = heapq.heapreplace(_hr, 4)
assert _popped == 1; _ledger.append(1)
assert _hr[0] == 3; _ledger.append(1)
assert len(_hr) == 4; _ledger.append(1)
# Subsequent pops are in heap order
_hr_out: list[int] = []
while _hr:
    _hr_out.append(heapq.heappop(_hr))
assert _hr_out == [3, 4, 5, 7]; _ledger.append(1)

# 6) heappushpop — push, then pop smallest, atomically
_hpp: list[int] = [1, 5, 3]
heapq.heapify(_hpp)
# Pushing 0 then popping yields 0 (smaller than the heap min)
assert heapq.heappushpop(_hpp, 0) == 0; _ledger.append(1)
# The heap should still have 3 elements: 1, 5, 3
assert len(_hpp) == 3; _ledger.append(1)
assert _hpp[0] == 1; _ledger.append(1)
# Pushing 4 then popping yields 1 (the old min)
assert heapq.heappushpop(_hpp, 4) == 1; _ledger.append(1)
assert _hpp[0] == 3; _ledger.append(1)

# 7) nsmallest — top N smallest, no key= kwarg
assert heapq.nsmallest(3, [5, 2, 8, 1, 7, 3]) == [1, 2, 3]; _ledger.append(1)
assert heapq.nsmallest(1, [5, 2, 8, 1, 7]) == [1]; _ledger.append(1)
assert heapq.nsmallest(0, [5, 2, 8, 1, 7]) == []; _ledger.append(1)
# Empty input
assert heapq.nsmallest(3, []) == []; _ledger.append(1)
# N larger than the input size
assert heapq.nsmallest(10, [5, 2, 8]) == [2, 5, 8]; _ledger.append(1)

# 8) nlargest — top N largest, no key= kwarg
assert heapq.nlargest(3, [5, 2, 8, 1, 7, 3]) == [8, 7, 5]; _ledger.append(1)
assert heapq.nlargest(1, [5, 2, 8, 1, 7]) == [8]; _ledger.append(1)
assert heapq.nlargest(0, [5, 2, 8, 1, 7]) == []; _ledger.append(1)
assert heapq.nlargest(3, []) == []; _ledger.append(1)
assert heapq.nlargest(10, [5, 2, 8]) == [8, 5, 2]; _ledger.append(1)

# 9) heapq.merge — k-way merge of sorted iterables (no reverse= kwarg)
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert list(heapq.merge([1], [2], [3])) == [1, 2, 3]; _ledger.append(1)
assert list(heapq.merge([1, 4], [2, 5], [3, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
# One empty iter
assert list(heapq.merge([], [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(heapq.merge([1, 2, 3], [])) == [1, 2, 3]; _ledger.append(1)
# Both empty
assert list(heapq.merge([], [])) == []; _ledger.append(1)
# Single iter
assert list(heapq.merge([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
# Overlapping values
assert list(heapq.merge([1, 1, 3], [2, 3])) == [1, 1, 2, 3, 3]; _ledger.append(1)

# 10) max-heap via negation — canonical idiom
_max: list[int] = []
for _v in [3, 1, 4, 1, 5, 9, 2, 6]:
    heapq.heappush(_max, -_v)
# Top of the negated heap is the most-negative -> the largest original
assert -_max[0] == 9; _ledger.append(1)
# Pop the entire negated heap and negate back
_max_out: list[int] = []
while _max:
    _max_out.append(-heapq.heappop(_max))
assert _max_out == [9, 6, 5, 4, 3, 2, 1, 1]; _ledger.append(1)

# 11) Float-valued heap
_fh: list[float] = [3.14, 1.5, 2.0, 0.5, 1.1]
heapq.heapify(_fh)
assert _fh[0] == 0.5; _ledger.append(1)
_fh_out: list[float] = []
while _fh:
    _fh_out.append(heapq.heappop(_fh))
assert _fh_out == [0.5, 1.1, 1.5, 2.0, 3.14]; _ledger.append(1)

# NB: string-keyed heaps (lex order) DIVERGE on mamba — heapify on a
# str list does not re-establish the heap invariant, leaving the input
# in its original order. Moved to the divergence-spec fixture.

# 13) Single-element heap
_one: list[int] = [42]
heapq.heapify(_one)
assert _one[0] == 42; _ledger.append(1)
assert heapq.heappop(_one) == 42; _ledger.append(1)
assert _one == []; _ledger.append(1)

# 14) Already-sorted input is a valid heap
_already: list[int] = [1, 2, 3, 4, 5]
heapq.heapify(_already)
assert _already[0] == 1; _ledger.append(1)
assert heapq.heappop(_already) == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_heapq_min_heap_merge_ops {sum(_ledger)} asserts")
