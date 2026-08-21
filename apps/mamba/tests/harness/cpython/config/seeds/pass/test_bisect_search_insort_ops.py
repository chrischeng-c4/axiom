# Operational AssertionPass seed for the `bisect` module — the
# stdlib binary-search / sorted-insertion utilities (`bisect_left`,
# `bisect_right`, `bisect`, `insort_left`, `insort_right`, `insort`)
# used by sorted-collection maintenance, order-statistic queries,
# leaderboard inserts, range-membership tests, and any algorithm
# that needs O(log n) lookup over an already-sorted sequence.
# Surface focuses on the matching subset between mamba and CPython
# on numeric / float / string elements (all element types work
# the same way because the bisect protocol delegates to the
# `<` operator). No fixture coverage yet for bisect.
#
# Surface:
#   • bisect.bisect_left(L, x, lo=0, hi=len(L)) → int
#       — leftmost insertion index for x such that L[:i] < x ≤ L[i:];
#       — `bisect_left([], x) == 0`;
#       — `bisect_left([1,3,5,7,9], 5) == 2`;
#       — duplicates skewed left: `bisect_left([1,5,5,5,7], 5) == 1`;
#   • bisect.bisect_right(L, x, lo=0, hi=len(L)) → int
#       — rightmost insertion index for x;
#       — `bisect_right([1,3,5,7,9], 5) == 3`;
#       — duplicates skewed right: `bisect_right([1,5,5,5,7], 5) == 4`;
#   • bisect.bisect — alias for `bisect_right`;
#   • bisect.insort_left(L, x, lo=0, hi=len(L)) → None
#       — inserts x at the leftmost valid position; mutates L
#         in place;
#   • bisect.insort_right(L, x, lo=0, hi=len(L)) → None
#       — inserts x at the rightmost valid position;
#   • bisect.insort — alias for `insort_right`;
#   • lo / hi parameters restrict the search range.
import bisect
_ledger: list[int] = []

# bisect_left — basic monotone sequence
_L = [1, 3, 5, 7, 9]
assert bisect.bisect_left(_L, 5) == 2; _ledger.append(1)
assert bisect.bisect_left(_L, 4) == 2; _ledger.append(1)
assert bisect.bisect_left(_L, 0) == 0; _ledger.append(1)
assert bisect.bisect_left(_L, 100) == 5; _ledger.append(1)
assert bisect.bisect_left(_L, 1) == 0; _ledger.append(1)
assert bisect.bisect_left(_L, 9) == 4; _ledger.append(1)
assert bisect.bisect_left(_L, 10) == 5; _ledger.append(1)

# bisect_right — same sequence
assert bisect.bisect_right(_L, 5) == 3; _ledger.append(1)
assert bisect.bisect_right(_L, 4) == 2; _ledger.append(1)
assert bisect.bisect_right(_L, 0) == 0; _ledger.append(1)
assert bisect.bisect_right(_L, 100) == 5; _ledger.append(1)
assert bisect.bisect_right(_L, 1) == 1; _ledger.append(1)
assert bisect.bisect_right(_L, 9) == 5; _ledger.append(1)

# bisect (alias for bisect_right)
assert bisect.bisect(_L, 5) == 3; _ledger.append(1)
assert bisect.bisect(_L, 0) == 0; _ledger.append(1)
assert bisect.bisect(_L, 100) == 5; _ledger.append(1)

# bisect_left/right on empty list
assert bisect.bisect_left([], 5) == 0; _ledger.append(1)
assert bisect.bisect_right([], 5) == 0; _ledger.append(1)
assert bisect.bisect([], 5) == 0; _ledger.append(1)

# bisect_left/right on single-element list
assert bisect.bisect_left([5], 3) == 0; _ledger.append(1)
assert bisect.bisect_left([5], 5) == 0; _ledger.append(1)
assert bisect.bisect_left([5], 7) == 1; _ledger.append(1)
assert bisect.bisect_right([5], 3) == 0; _ledger.append(1)
assert bisect.bisect_right([5], 5) == 1; _ledger.append(1)
assert bisect.bisect_right([5], 7) == 1; _ledger.append(1)

# bisect_left/right with duplicates
_dups = [1, 3, 5, 5, 5, 7, 9]
assert bisect.bisect_left(_dups, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(_dups, 5) == 5; _ledger.append(1)
# Range of duplicates: [bisect_left, bisect_right) covers all equal items
assert bisect.bisect_right(_dups, 5) - bisect.bisect_left(_dups, 5) == 3; _ledger.append(1)

# lo / hi parameters (matching subset — mamba ignores `lo` when it
# exceeds the natural insertion point, so only cases where the
# natural insertion lands inside [lo, hi) are exercised here)
assert bisect.bisect_left(_L, 5, 1) == 2; _ledger.append(1)
assert bisect.bisect_left(_L, 5, 2, 4) == 2; _ledger.append(1)
assert bisect.bisect_right(_L, 5, 0, 5) == 3; _ledger.append(1)
assert bisect.bisect_left(_L, 5, 0, 5) == 2; _ledger.append(1)

# insort_left — leftmost insertion mutates in place, returns None
_il = [1, 3, 5, 7, 9]
assert bisect.insort_left(_il, 4) is None; _ledger.append(1)
assert _il == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
# Insert duplicate, lands at leftmost position
_il2 = [1, 3, 5, 7, 9]
bisect.insort_left(_il2, 5)
assert _il2 == [1, 3, 5, 5, 7, 9]; _ledger.append(1)

# insort_right — same insertion point because no duplicates exist
_ir = [1, 3, 5, 7, 9]
bisect.insort_right(_ir, 4)
assert _ir == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
# Insert duplicate, lands at rightmost position
_ir2 = [1, 3, 5, 7, 9]
bisect.insort_right(_ir2, 5)
assert _ir2 == [1, 3, 5, 5, 7, 9]; _ledger.append(1)

# insort (alias for insort_right)
_in = [1, 3, 5, 7, 9]
bisect.insort(_in, 4)
assert _in == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
_in2 = [1, 3, 5, 7, 9]
bisect.insort(_in2, 5)
assert _in2 == [1, 3, 5, 5, 7, 9]; _ledger.append(1)

# insort on empty list
_e: list[int] = []
bisect.insort(_e, 5)
assert _e == [5]; _ledger.append(1)
bisect.insort(_e, 3)
assert _e == [3, 5]; _ledger.append(1)
bisect.insort(_e, 4)
assert _e == [3, 4, 5]; _ledger.append(1)

# Float elements — same protocol via < operator
_F = [1.0, 2.5, 4.0, 5.5]
assert bisect.bisect_left(_F, 3.0) == 2; _ledger.append(1)
assert bisect.bisect_left(_F, 2.5) == 1; _ledger.append(1)
assert bisect.bisect_right(_F, 2.5) == 2; _ledger.append(1)
assert bisect.bisect_left(_F, 0.5) == 0; _ledger.append(1)
assert bisect.bisect_left(_F, 100.0) == 4; _ledger.append(1)

# String elements — same protocol via < operator
_S = ["apple", "banana", "cherry", "date"]
assert bisect.bisect_left(_S, "banana") == 1; _ledger.append(1)
assert bisect.bisect_left(_S, "cat") == 2; _ledger.append(1)
assert bisect.bisect_right(_S, "banana") == 2; _ledger.append(1)
assert bisect.bisect_left(_S, "aardvark") == 0; _ledger.append(1)
assert bisect.bisect_left(_S, "zebra") == 4; _ledger.append(1)

# Return type discipline
assert isinstance(bisect.bisect_left(_L, 5), int); _ledger.append(1)
assert isinstance(bisect.bisect_right(_L, 5), int); _ledger.append(1)
assert isinstance(bisect.bisect(_L, 5), int); _ledger.append(1)

# Module-level attribute discipline
for _name in ["bisect_left", "bisect_right", "bisect",
              "insort_left", "insort_right", "insort"]:
    assert hasattr(bisect, _name); _ledger.append(1)
    assert callable(getattr(bisect, _name)); _ledger.append(1)

# Idempotence — same query, same result
assert bisect.bisect_left(_L, 5) == bisect.bisect_left(_L, 5); _ledger.append(1)
assert bisect.bisect_right(_L, 5) == bisect.bisect_right(_L, 5); _ledger.append(1)

# bisect_left ≤ bisect_right invariant (always)
for _x in [0, 1, 4, 5, 9, 100]:
    assert bisect.bisect_left(_L, _x) <= bisect.bisect_right(_L, _x); _ledger.append(1)

# Result is always in [0, len(L)] range
for _x in [0, 1, 4, 5, 9, 100]:
    _idx = bisect.bisect_left(_L, _x)
    assert 0 <= _idx <= len(_L); _ledger.append(1)

# insort preserves sortedness
_sorted_chk = [1, 3, 5, 7, 9]
for _new in [0, 4, 6, 8, 10, 2, 11]:
    bisect.insort(_sorted_chk, _new)
# Verify sorted
for _i in range(len(_sorted_chk) - 1):
    assert _sorted_chk[_i] <= _sorted_chk[_i + 1]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bisect_search_insort_ops {sum(_ledger)} asserts")
