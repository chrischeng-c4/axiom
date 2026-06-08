# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_heapq_heap_ops"
# subject = "cpython321.test_heapq_heap_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_heapq_heap_ops.py"
# status = "filled"
# ///
"""cpython321.test_heapq_heap_ops: execute CPython 3.12 seed test_heapq_heap_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `heapq` module — the
# stdlib binary-heap (min-heap) implementation used by priority
# queues, scheduling, top-k retrieval (`nlargest` / `nsmallest`),
# k-way merge (`merge`), and any algorithm that needs O(log n)
# insert + O(1) min-access. Surface focuses on the matching subset
# between mamba and CPython on numeric / float scalar elements:
# `heapify` / `heappush` / `heappop` / `heappushpop` /
# `heapreplace` produce identical heap arrays and identical
# pop sequences. Mamba's `heapify` on tuples diverges (CPython
# orders by tuple-compare, mamba leaves the tuple list near-
# unchanged), and `nsmallest` / `nlargest` with the `key=`
# keyword argument is also divergent (mamba ignores `key` and
# returns the heap-order top-k, not the key-mapped top-k) —
# both are left to a spec fixture and not exercised here. No
# fixture coverage yet for heapq.
#
# Surface (the matching subset):
#   • heapq.heapify(L: list[T]) → None
#       — rearranges L in-place into a min-heap (L[0] is the min);
#       — empty list stays empty;
#       — heapify is idempotent on already-heapified input;
#   • heapq.heappush(L: list[T], item: T) → None
#       — inserts item; L[0] remains the min;
#       — heap invariant preserved: L[i] <= L[2*i+1], L[2*i+2];
#   • heapq.heappop(L: list[T]) → T
#       — removes and returns the smallest item;
#       — repeated popping yields sorted order ascending;
#   • heapq.heappushpop(L: list[T], item: T) → T
#       — push then pop, but in a single op; returns the smaller of
#         (item, L[0]);
#   • heapq.heapreplace(L: list[T], item: T) → T
#       — pop then push; returns the smaller of (item, original L[0])
#         after the swap;
#   • heapq.nsmallest(n: int, iterable, key=None) → list[T]
#       — returns the n smallest items (without `key` only);
#       — `nsmallest(0, ...) == []`;
#       — `nsmallest(n, []) == []`;
#       — `nsmallest(n, L) == sorted(L)[:n]` for the matching subset;
#   • heapq.nlargest(n: int, iterable, key=None) → list[T]
#       — returns the n largest items (without `key` only);
#       — `nlargest(0, ...) == []`;
#       — `nlargest(n, []) == []`;
#       — `nlargest(n, L) == sorted(L, reverse=True)[:n]`;
#   • heapq.merge(*iterables) → iterator
#       — k-way merge of sorted inputs;
#       — `list(merge()) == []`;
#       — `list(merge([])) == []`.
import heapq
_ledger: list[int] = []

# heapify — empty list
_empty: list[int] = []
assert heapq.heapify(_empty) is None; _ledger.append(1)
assert _empty == []; _ledger.append(1)

# heapify — single element
_one = [42]
heapq.heapify(_one)
assert _one == [42]; _ledger.append(1)

# heapify — standard small heap
_h = [5, 3, 8, 1, 9, 2]
heapq.heapify(_h)
assert _h == [1, 3, 2, 5, 9, 8]; _ledger.append(1)
assert _h[0] == 1; _ledger.append(1)

# heapify — already in heap order is idempotent
_pre = [1, 3, 2, 5, 9, 8]
heapq.heapify(_pre)
assert _pre == [1, 3, 2, 5, 9, 8]; _ledger.append(1)

# heappush — insert preserves min at index 0
_h2 = [1, 3, 2, 5, 9, 8]
heapq.heappush(_h2, 0)
assert _h2[0] == 0; _ledger.append(1)
assert len(_h2) == 7; _ledger.append(1)

# heappush — into empty
_h3: list[int] = []
heapq.heappush(_h3, 5)
assert _h3 == [5]; _ledger.append(1)
heapq.heappush(_h3, 3)
assert _h3[0] == 3; _ledger.append(1)
heapq.heappush(_h3, 1)
assert _h3[0] == 1; _ledger.append(1)

# heappop — pop yields min
_h4 = [1, 3, 2, 5, 9, 8]
heapq.heapify(_h4)
assert heapq.heappop(_h4) == 1; _ledger.append(1)
assert heapq.heappop(_h4) == 2; _ledger.append(1)
assert heapq.heappop(_h4) == 3; _ledger.append(1)

# heappop — full sort sequence
_h5 = [5, 3, 8, 1, 9, 2, 7, 4, 6]
heapq.heapify(_h5)
_sorted: list[int] = []
while _h5:
    _sorted.append(heapq.heappop(_h5))
assert _sorted == [1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

# heappushpop — push 0 to [1,2,3] returns 0 (smaller of new and existing min)
assert heapq.heappushpop([1, 2, 3], 0) == 0; _ledger.append(1)
# heappushpop — push 5 to [1,2,3] returns 1 (existing min)
assert heapq.heappushpop([1, 2, 3], 5) == 1; _ledger.append(1)

# heapreplace — pop+push, returns original min
assert heapq.heapreplace([1, 2, 3], 4) == 1; _ledger.append(1)
assert heapq.heapreplace([1, 2, 3], 0) == 1; _ledger.append(1)

# nsmallest — without key
assert heapq.nsmallest(3, [5, 3, 8, 1, 9, 2]) == [1, 2, 3]; _ledger.append(1)
assert heapq.nsmallest(1, [5, 3, 8, 1, 9, 2]) == [1]; _ledger.append(1)
assert heapq.nsmallest(0, [1, 2, 3]) == []; _ledger.append(1)
assert heapq.nsmallest(3, []) == []; _ledger.append(1)
assert heapq.nsmallest(10, [1, 2, 3]) == [1, 2, 3]; _ledger.append(1)
assert heapq.nsmallest(5, [9, 8, 7, 6, 5, 4, 3, 2, 1]) == [1, 2, 3, 4, 5]; _ledger.append(1)

# nlargest — without key
assert heapq.nlargest(3, [5, 3, 8, 1, 9, 2]) == [9, 8, 5]; _ledger.append(1)
assert heapq.nlargest(1, [5, 3, 8, 1, 9, 2]) == [9]; _ledger.append(1)
assert heapq.nlargest(0, [1, 2, 3]) == []; _ledger.append(1)
assert heapq.nlargest(3, []) == []; _ledger.append(1)
assert heapq.nlargest(10, [1, 2, 3]) == [3, 2, 1]; _ledger.append(1)
assert heapq.nlargest(5, [1, 2, 3, 4, 5, 6, 7, 8, 9]) == [9, 8, 7, 6, 5]; _ledger.append(1)

# merge — empty input
assert list(heapq.merge()) == []; _ledger.append(1)
assert list(heapq.merge([])) == []; _ledger.append(1)
assert list(heapq.merge([], [])) == []; _ledger.append(1)

# merge — single iterable
assert list(heapq.merge([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# merge — two sorted iterables
assert list(heapq.merge([1, 3, 5], [2, 4, 6])) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)

# merge — three sorted iterables
assert list(heapq.merge([1, 4, 7], [2, 5, 8], [3, 6, 9])) == [1, 2, 3, 4, 5, 6, 7, 8, 9]; _ledger.append(1)

# merge — with duplicates
assert list(heapq.merge([1, 2, 2, 3], [2, 3, 4])) == [1, 2, 2, 2, 3, 3, 4]; _ledger.append(1)

# Float heap — same protocol
_hf = [3.14, 1.5, 2.7, 0.1]
heapq.heapify(_hf)
assert _hf[0] == 0.1; _ledger.append(1)
_sorted_f: list[float] = []
while _hf:
    _sorted_f.append(heapq.heappop(_hf))
assert _sorted_f == [0.1, 1.5, 2.7, 3.14]; _ledger.append(1)

# Return type discipline
assert isinstance(heapq.nsmallest(2, [1, 2, 3]), list); _ledger.append(1)
assert isinstance(heapq.nlargest(2, [1, 2, 3]), list); _ledger.append(1)

# Module-level attribute discipline
assert hasattr(heapq, "heapify"); _ledger.append(1)
assert hasattr(heapq, "heappush"); _ledger.append(1)
assert hasattr(heapq, "heappop"); _ledger.append(1)
assert hasattr(heapq, "heappushpop"); _ledger.append(1)
assert hasattr(heapq, "heapreplace"); _ledger.append(1)
assert hasattr(heapq, "nsmallest"); _ledger.append(1)
assert hasattr(heapq, "nlargest"); _ledger.append(1)
assert hasattr(heapq, "merge"); _ledger.append(1)
for _name in ["heapify", "heappush", "heappop", "heappushpop",
              "heapreplace", "nsmallest", "nlargest", "merge"]:
    assert callable(getattr(heapq, _name)); _ledger.append(1)

# Idempotence
assert heapq.nsmallest(3, [5, 3, 8, 1, 9, 2]) == heapq.nsmallest(3, [5, 3, 8, 1, 9, 2]); _ledger.append(1)
assert heapq.nlargest(3, [5, 3, 8, 1, 9, 2]) == heapq.nlargest(3, [5, 3, 8, 1, 9, 2]); _ledger.append(1)

# Length invariants
_h6 = [3, 1, 2]
heapq.heapify(_h6)
assert len(_h6) == 3; _ledger.append(1)
heapq.heappush(_h6, 0)
assert len(_h6) == 4; _ledger.append(1)
_ = heapq.heappop(_h6)
assert len(_h6) == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_heapq_heap_ops {sum(_ledger)} asserts")
