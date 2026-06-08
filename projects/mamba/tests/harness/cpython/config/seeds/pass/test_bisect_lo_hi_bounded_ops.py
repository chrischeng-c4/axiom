# Operational AssertionPass seed for `bisect.bisect_left` /
# `bisect.bisect_right` / `bisect.insort_left` / `bisect.insort_right`
# / `bisect.insort` with the `lo` / `hi` SEARCH-WINDOW BOUNDS — both
# positional AND keyword forms. The existing bisect seeds
# (test_bisect, test_bisect_insort_ops, test_bisect_ops,
# test_bisect_search_insort_ops) cover the 2-argument `(L, x)` form
# extensively but leave the 3-/4-argument `(L, x, lo)`, `(L, x, lo,
# hi)`, `(L, x, lo=lo, hi=hi)` window-bounded forms — plus the
# corresponding insort variants — effectively unverified for both
# runtimes.
#
# Surface (the matching subset between mamba and CPython):
#   • bisect.bisect_left(L, x, lo) — positional lo, when the natural
#     insertion point is >= lo so the bound doesn't clamp the answer;
#   • bisect.bisect_left(L, x, lo, hi) — positional lo + hi, when
#     lo <= natural index <= hi;
#   • bisect.bisect_left(L, x, lo=lo) — keyword lo only;
#   • bisect.bisect_left(L, x, hi=hi) — keyword hi only;
#   • bisect.bisect_left(L, x, lo=lo, hi=hi) — keyword lo + hi;
#   • the same five forms for bisect.bisect_right;
#   • bisect.insort_left/insort_right/insort with positional and
#     keyword lo/hi.
#
# Empty-window edge cases (lo == hi at the natural insertion point)
# degenerate to the single insertion-index = lo.
#
# The MISBEHAVED case (lo > natural insertion point — CPython clamps to
# lo, mamba silently ignores lo and returns the unclamped natural
# index; same dual for hi < natural) is covered in the spec seed
# lang_bisect_heapq_key_silent.py.
import bisect

_ledger: list[int] = []

# 1) bisect_left — positional lo + hi (natural index inside the window)
# Search [1, 3, 5, 7, 9] for 4, starting at index 2 → still finds insertion at 2
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, 2) == 2; _ledger.append(1)
# Restrict window to [0, 3) — search [1, 3, 5] for 4 → 2
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, 0, 3) == 2; _ledger.append(1)
# Search [1, 3, 5, 7, 9] for 6, starting at index 1 → 3 (between 5 and 7)
assert bisect.bisect_left([1, 3, 5, 7, 9], 6, 1) == 3; _ledger.append(1)
# Search [1, 3, 5, 7, 9] for 8, window [0, 5) → 4
assert bisect.bisect_left([1, 3, 5, 7, 9], 8, 0, 5) == 4; _ledger.append(1)

# 2) bisect_right — positional lo + hi
assert bisect.bisect_right([1, 3, 5, 7, 9], 4, 1) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 4, 0, 3) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 6, 0, 5) == 3; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 5, 0, 5) == 3; _ledger.append(1)

# 3) bisect_left — keyword lo only (lo <= natural index)
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, lo=2) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 6, lo=1) == 3; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 6, lo=0) == 3; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 8, lo=4) == 4; _ledger.append(1)

# 4) bisect_left — keyword hi only (natural index <= hi)
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, hi=3) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 2, hi=2) == 1; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 0, hi=2) == 0; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 6, hi=4) == 3; _ledger.append(1)

# 5) bisect_left — keyword lo + hi (lo <= natural index <= hi)
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, lo=1, hi=4) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, lo=0, hi=5) == 2; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 6, lo=2, hi=5) == 3; _ledger.append(1)
assert bisect.bisect_left([1, 3, 5, 7, 9], 8, lo=3, hi=5) == 4; _ledger.append(1)

# 6) bisect_right — keyword forms
assert bisect.bisect_right([1, 3, 5, 7, 9], 4, lo=1) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 6, lo=0, hi=5) == 3; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 5, lo=0, hi=5) == 3; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 8, lo=3, hi=5) == 4; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 2, hi=3) == 1; _ledger.append(1)

# 7) Empty-window degenerate cases (lo == hi == natural insertion)
assert bisect.bisect_left([1, 3, 5, 7, 9], 4, lo=2, hi=2) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 4, lo=2, hi=2) == 2; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 0, lo=0, hi=0) == 0; _ledger.append(1)

# 8) Duplicate-element handling within window
assert bisect.bisect_left([1, 2, 2, 2, 3], 2, lo=0, hi=5) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 2, 2, 2, 3], 2, lo=0, hi=5) == 4; _ledger.append(1)
assert bisect.bisect_left([1, 2, 2, 2, 3], 2, lo=0, hi=3) == 1; _ledger.append(1)
assert bisect.bisect_right([1, 2, 2, 2, 3], 2, lo=1, hi=5) == 4; _ledger.append(1)

# 9) insort_left with positional + keyword lo/hi (full-list windows)
_a = [1, 3, 5, 7]
bisect.insort_left(_a, 4, 0, 4)
assert _a == [1, 3, 4, 5, 7]; _ledger.append(1)

_b = [1, 3, 5, 7]
bisect.insort_left(_b, 4, lo=0, hi=4)
assert _b == [1, 3, 4, 5, 7]; _ledger.append(1)

_c = [1, 3, 5, 7]
bisect.insort_left(_c, 4, lo=1)
assert _c == [1, 3, 4, 5, 7]; _ledger.append(1)

# 10) insort_right with positional + keyword lo/hi
_d = [1, 3, 5, 7]
bisect.insort_right(_d, 4, 0, 4)
assert _d == [1, 3, 4, 5, 7]; _ledger.append(1)

_e = [1, 3, 5, 7]
bisect.insort_right(_e, 4, lo=0, hi=4)
assert _e == [1, 3, 4, 5, 7]; _ledger.append(1)

_f = [1, 3, 5, 7]
bisect.insort_right(_f, 4, lo=1)
assert _f == [1, 3, 4, 5, 7]; _ledger.append(1)

# 11) insort (alias for insort_right) with lo/hi
_g = [1, 3, 5, 7]
bisect.insort(_g, 4, 0, 4)
assert _g == [1, 3, 4, 5, 7]; _ledger.append(1)

_h = [1, 3, 5, 7]
bisect.insort(_h, 4, lo=0, hi=4)
assert _h == [1, 3, 4, 5, 7]; _ledger.append(1)

# 12) Duplicate insertion preserves bisect_left/bisect_right semantics
_i = [1, 2, 2, 3]
bisect.insort_left(_i, 2, lo=0, hi=4)
assert _i == [1, 2, 2, 2, 3]; _ledger.append(1)
# insort_left inserts BEFORE existing dupes — first 2 stays at index 1
assert _i.index(2) == 1; _ledger.append(1)

_j = [1, 2, 2, 3]
bisect.insort_right(_j, 2, lo=0, hi=4)
assert _j == [1, 2, 2, 2, 3]; _ledger.append(1)
# insort_right inserts AFTER existing dupes — last 2 at index 3
assert _j[3] == 2; _ledger.append(1)

# 13) Idempotent: bisect_left/right without bounds == with full-list bounds
_data = [10, 20, 30, 40, 50]
assert bisect.bisect_left(_data, 30) == bisect.bisect_left(_data, 30, lo=0, hi=5); _ledger.append(1)
assert bisect.bisect_right(_data, 30) == bisect.bisect_right(_data, 30, lo=0, hi=5); _ledger.append(1)
assert bisect.bisect_left(_data, 30) == bisect.bisect_left(_data, 30, 0, 5); _ledger.append(1)
assert bisect.bisect_right(_data, 30) == bisect.bisect_right(_data, 30, 0, 5); _ledger.append(1)

# 14) Single-element list edge cases (natural inside window)
assert bisect.bisect_left([5], 5, lo=0, hi=1) == 0; _ledger.append(1)
assert bisect.bisect_right([5], 5, lo=0, hi=1) == 1; _ledger.append(1)
assert bisect.bisect_left([5], 0, lo=0, hi=1) == 0; _ledger.append(1)

# 15) Two-element window inside a longer list (natural inside window)
_long = [10, 20, 30, 40, 50, 60, 70]
# Window [2, 5) -> indices [30, 40, 50]; find 35 → 3 (between 30 and 40)
assert bisect.bisect_left(_long, 35, lo=2, hi=5) == 3; _ledger.append(1)
assert bisect.bisect_right(_long, 35, lo=2, hi=5) == 3; _ledger.append(1)
# find 40 within same window
assert bisect.bisect_left(_long, 40, lo=2, hi=5) == 3; _ledger.append(1)
assert bisect.bisect_right(_long, 40, lo=2, hi=5) == 4; _ledger.append(1)

# 16) Window containing the natural insertion point at the boundary
# Insertion exactly at lo — i.e., natural == lo
assert bisect.bisect_left([1, 3, 5, 7, 9], 5, lo=2, hi=5) == 2; _ledger.append(1)
# Insertion exactly at hi - 1
assert bisect.bisect_left([1, 3, 5, 7, 9], 9, lo=4, hi=5) == 4; _ledger.append(1)
assert bisect.bisect_right([1, 3, 5, 7, 9], 9, lo=4, hi=5) == 5; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_bisect_lo_hi_bounded_ops {sum(_ledger)} asserts")
