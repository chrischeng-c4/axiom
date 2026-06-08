# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_bisect_heapq_key_silent"
# subject = "cpython321.lang_bisect_heapq_key_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_bisect_heapq_key_silent.py"
# status = "filled"
# ///
"""cpython321.lang_bisect_heapq_key_silent: execute CPython 3.12 seed lang_bisect_heapq_key_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Spec seed for CPython contracts on the `bisect` / `heapq` /
# `string.Template` keyword-argument family where mamba silently
# IGNORES the keyword instead of honoring the documented semantics.
# The keyword-effect divergence is silent — the call returns a
# numerically-shaped but functionally-WRONG answer — masking the
# call-site bug where the caller passed `key=`, `reverse=`, `lo=`,
# or `hi=` expecting the search/merge to honor it.
#
# Surface (CPython honors the kwarg, mamba silently ignores):
#   (1) `bisect.bisect_left(L, x, lo=lo)` where `lo > natural index`
#       — CPython clamps the result to `lo`; mamba returns the
#       unclamped natural index;
#   (2) `bisect.bisect_left(L, x, hi=hi)` / `bisect_right(L, x, hi=hi)`
#       where `hi < natural index` — CPython clamps to `hi`; mamba
#       returns the unclamped natural index;
#   (3) `bisect.bisect_left(L, x, key=fn)` / `bisect_right(L, x,
#       key=fn)` — CPython compares `fn(L[i])` against `x`; mamba
#       silently ignores `key=` and compares the raw element against
#       `x` (cross-type comparison may produce a wrong but non-error
#       answer);
#   (4) `heapq.nlargest(n, iter, key=fn)` / `heapq.nsmallest(n, iter,
#       key=fn)` — CPython picks the top-/bottom-K by `key(item)`;
#       mamba silently ignores `key=` and ranks by raw element;
#   (5) `heapq.merge(*iters, reverse=True)` — CPython performs a
#       descending merge; mamba silently performs ascending merge;
#   (6) `heapq.merge(*iters, key=fn)` — CPython merges by `key(item)`;
#       mamba silently ignores `key=`;
#   (7) `string.Template(s)` — CPython returns a `Template` instance
#       with `.substitute(...)` / `.safe_substitute(...)` methods;
#       mamba returns a plain `dict`, so any attribute access fails
#       with `AttributeError` (wrong-type / missing-method divergence).
#
# Mamba behavior:
#   • `bisect.bisect_left([1,3,5,7,9], 0, lo=2)` → 0 (silent — lo ignored,
#     should be 2);
#   • `bisect.bisect_right([1,3,5,7,9], 99, lo=3, hi=3)` → 5 (silent —
#     hi ignored, should be 3);
#   • `bisect.bisect_left([(1,'a'),(3,'b'),(5,'c')], 4, key=lambda
#     p: p[0])` → 0 (silent — key ignored, should be 2);
#   • `heapq.nlargest(2, [9,3,7,1,8], key=lambda x: -x)` → [9, 8]
#     (silent — key ignored, should be [1, 3]);
#   • `heapq.nsmallest(2, [9,3,7,1,8], key=lambda x: -x)` → [1, 3]
#     (silent — key ignored, should be [9, 8]);
#   • `list(heapq.merge([5,3,1], [6,4,2], reverse=True))` → [1, 2, 3,
#     4, 5, 6] (silent — reverse ignored, should be [6, 5, 4, 3, 2, 1]);
#   • `list(heapq.merge([3,1], [4,2], key=lambda x: -x))` → [1, 2, 3,
#     4] (silent — key ignored, should be [4, 3, 2, 1]);
#   • `string.Template("$x")` → `{}` (the `Template` constructor
#     silently returns an empty dict; subsequent `.substitute(...)`
#     raises `AttributeError`).
#
# CPython contract:
#   bisect.bisect_left(L, x, lo=lo) with lo > natural index → result == lo;
#   bisect.bisect_*(L, x, hi=hi) with hi < natural index → result == hi;
#   bisect.bisect_*(L, x, key=fn) → uses fn(L[i]) for comparison;
#   heapq.nlargest/nsmallest(n, it, key=fn) → ranks by key(item);
#   heapq.merge(*it, reverse=True) → descending merge;
#   heapq.merge(*it, key=fn) → merges by key(item);
#   string.Template(s) → instance with `.substitute` / `.safe_substitute`
#     producing the substituted string.
import bisect
import heapq
from string import Template

_ledger: list[int] = []

# (1) bisect.bisect_left(L, x, lo=lo) clamps when lo > natural index
# Natural insertion of 0 in [1, 3, 5, 7, 9] is 0; lo=2 should clamp to 2
_r = bisect.bisect_left([1, 3, 5, 7, 9], 0, lo=2)
assert _r == 2, f"bisect_left with lo=2 must clamp 0's natural index 0 → 2, got {_r!r}"
_ledger.append(1)

# Same for positional lo
_r = bisect.bisect_left([1, 3, 5, 7, 9], 0, 3)
assert _r == 3, f"bisect_left with positional lo=3 must clamp 0's index 0 → 3, got {_r!r}"
_ledger.append(1)

# bisect_right with lo > natural
_r = bisect.bisect_right([1, 3, 5, 7, 9], 0, lo=2)
assert _r == 2, f"bisect_right with lo=2 must clamp → 2, got {_r!r}"
_ledger.append(1)

# (2) bisect_*(L, x, hi=hi) clamps when hi < natural index
# Natural insertion of 99 in [1, 3, 5, 7, 9] is 5; hi=3 should clamp to 3
_r = bisect.bisect_left([1, 3, 5, 7, 9], 99, hi=3)
assert _r == 3, f"bisect_left with hi=3 must clamp 99's index 5 → 3, got {_r!r}"
_ledger.append(1)

# bisect_right with hi clamping
_r = bisect.bisect_right([1, 3, 5, 7, 9], 99, hi=2)
assert _r == 2, f"bisect_right with hi=2 must clamp → 2, got {_r!r}"
_ledger.append(1)

# bisect_right([1, 2, 2, 2, 3], 2, hi=3) — natural is 4, hi=3 clamps to 3
_r = bisect.bisect_right([1, 2, 2, 2, 3], 2, hi=3)
assert _r == 3, f"bisect_right with hi=3 must clamp natural 4 → 3, got {_r!r}"
_ledger.append(1)

# bisect_left with lo+hi both at the same point (empty window)
# lo=hi=3 means insertion is forced at 3 regardless of search value
_r = bisect.bisect_left([1, 3, 5, 7, 9], 99, lo=3, hi=3)
assert _r == 3, f"bisect_left with empty window lo=hi=3 → 3, got {_r!r}"
_ledger.append(1)

_r = bisect.bisect_right([1, 3, 5, 7, 9], 99, lo=3, hi=3)
assert _r == 3, f"bisect_right with empty window lo=hi=3 → 3, got {_r!r}"
_ledger.append(1)

# (3) bisect_left(L, x, key=fn) — key is honored when computing comparison
# L is a list of (k, v) tuples sorted by k; with key=p: p[0] the search
# value 4 is compared against L[i][0] yielding insertion at index 2.
_pairs = [(1, "a"), (3, "b"), (5, "c")]
_r = bisect.bisect_left(_pairs, 4, key=lambda p: p[0])
assert _r == 2, f"bisect_left key=first must place 4 between (3,b) and (5,c), got {_r!r}"
_ledger.append(1)

# bisect_right key=first
_r = bisect.bisect_right(_pairs, 4, key=lambda p: p[0])
assert _r == 2, f"bisect_right key=first must place 4 → 2, got {_r!r}"
_ledger.append(1)

# bisect_left key=len on strings sorted by length
_strings = ["a", "bb", "ccc", "dddd"]
_r = bisect.bisect_left(_strings, 2, key=len)
assert _r == 1, f"bisect_left key=len searching for length-2 must land at index 1, got {_r!r}"
_ledger.append(1)

# bisect_right key=len — places target AFTER existing matches
_r = bisect.bisect_right(_strings, 2, key=len)
assert _r == 2, f"bisect_right key=len for length-2 must land at index 2, got {_r!r}"
_ledger.append(1)

# (4) heapq.nlargest(n, iter, key=fn) — key is honored when ranking
# With key=neg, the "largest" values are the most-negative-of-the-negated
# i.e. the smallest actual values. nlargest(2, ...) by key=-x → [1, 3]
_r = heapq.nlargest(2, [9, 3, 7, 1, 8], key=lambda x: -x)
assert _r == [1, 3], f"nlargest key=neg must return [1,3], got {_r!r}"
_ledger.append(1)

# heapq.nsmallest with key=neg → returns largest actual values
_r = heapq.nsmallest(2, [9, 3, 7, 1, 8], key=lambda x: -x)
assert _r == [9, 8], f"nsmallest key=neg must return [9,8], got {_r!r}"
_ledger.append(1)

# nlargest key=len on strings
_r = heapq.nlargest(2, ["a", "ccc", "bb", "dddd"], key=len)
assert _r == ["dddd", "ccc"], f"nlargest key=len must return [dddd, ccc], got {_r!r}"
_ledger.append(1)

# (5) heapq.merge(*iters, reverse=True) — descending merge
# Inputs must each be PRE-SORTED DESCENDING when reverse=True
_r = list(heapq.merge([5, 3, 1], [6, 4, 2], reverse=True))
assert _r == [6, 5, 4, 3, 2, 1], f"merge reverse=True must descend, got {_r!r}"
_ledger.append(1)

# Single stream with reverse=True
_r = list(heapq.merge([5, 4, 3, 2, 1], reverse=True))
assert _r == [5, 4, 3, 2, 1], f"merge single reverse=True, got {_r!r}"
_ledger.append(1)

# (6) heapq.merge(*iters, key=fn) — merges by key(item)
# Inputs must each be SORTED BY key(item) ascending
_r = list(heapq.merge([3, 1], [4, 2], key=lambda x: -x))
assert _r == [4, 3, 2, 1], f"merge key=neg must descend, got {_r!r}"
_ledger.append(1)

# merge key=len — sort by length
_r = list(heapq.merge(["bb", "dddd"], ["a", "ccc"], key=len))
assert _r == ["a", "bb", "ccc", "dddd"], f"merge key=len must sort by length, got {_r!r}"
_ledger.append(1)

# (7) string.Template — construction must yield Template instance
_t = Template("Hello $name")
assert isinstance(_t, Template), f"Template(...) must return Template instance, got {type(_t).__name__}"
_ledger.append(1)

# .substitute() must produce the substituted string
try:
    _r = _t.substitute(name="X")
    assert _r == "Hello X", f"Template.substitute must produce 'Hello X', got {_r!r}"
    _ledger.append(1)
except AttributeError as e:
    raise AssertionError(f"Template.substitute must be callable, got AttributeError: {e}")

# .safe_substitute() with missing key leaves the placeholder intact
try:
    _t2 = Template("Hi $a and $b")
    _r = _t2.safe_substitute(a="X")
    assert _r == "Hi X and $b", f"safe_substitute must leave $b alone, got {_r!r}"
    _ledger.append(1)
except AttributeError as e:
    raise AssertionError(f"Template.safe_substitute must be callable, got AttributeError: {e}")

# $$ → literal $ on a properly-constructed Template
try:
    _t3 = Template("$$ literal")
    _r = _t3.substitute()
    assert _r == "$ literal", f"Template $$ must produce literal $, got {_r!r}"
    _ledger.append(1)
except AttributeError as e:
    raise AssertionError(f"Template literal $$ must work, got AttributeError: {e}")

print(f"MAMBA_ASSERTION_PASS: lang_bisect_heapq_key_silent {sum(_ledger)} asserts")
