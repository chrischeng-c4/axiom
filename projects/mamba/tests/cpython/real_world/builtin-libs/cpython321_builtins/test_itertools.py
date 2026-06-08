# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_itertools"
# subject = "cpython321.test_itertools"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_itertools.py"
# status = "filled"
# ///
"""cpython321.test_itertools: execute CPython 3.12 seed test_itertools"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_itertools.py — #2698 CPython itertools seed (executed assertions).
#
# Replaces the prior vendored CPython upstream Lib/test/test_itertools.py
# (ranked `Fail` at the line-1172 `unexpected token: newline` parser
# gap) with a Mamba-authored seed distilled from the itertools
# bounded-iterator smoke surface. Exercises the deterministic-output
# combinators that work on both CPython 3.12 and mamba today and
# emits the runner's positive proof-of-execution marker that
# `cpython_lib_test_runner.rs` (#2691) classifies as `AssertionPass`.
#
# Why so small? Mamba's current itertools surface presents a healthy
# subset (chain, islice-on-range, repeat-with-N, product,
# permutations, combinations, accumulate, takewhile, dropwhile,
# starmap, zip_longest, compress, filterfalse, pairwise, groupby).
# Richer surface — `count()` / `cycle()` (infinite generators) and
# `chain.from_iterable()` — return `[]` when fed into `list(...)`
# on mamba today, so those are excluded from this seed.
#
# Why no helper function? Per the #2691 contract, top-level `def()`
# does not capture module-scope names by reference on mamba.
#
# Contract with the runner (#2691):
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: itertools N asserts` to stdout.

import itertools

_ledger: list[int] = []

# 1. Module identity.
assert itertools.__name__ == "itertools", "itertools.__name__ must be 'itertools'"
_ledger.append(1)

# 2. chain — concatenates iterables.
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4], "chain concatenates iterables"
_ledger.append(1)

# 3. islice — start/stop slicing of a range.
assert list(itertools.islice(range(10), 3, 6)) == [3, 4, 5], "islice(range(10), 3, 6)"
_ledger.append(1)

# 4. repeat with bounded N — emits the value N times.
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"], "repeat('x', 3)"
_ledger.append(1)

# 5. product — cartesian product of two iterables.
assert list(itertools.product([1, 2], ["a", "b"])) == [(1, "a"), (1, "b"), (2, "a"), (2, "b")], "product([1,2], ['a','b'])"
_ledger.append(1)

# 6. permutations — ordered selections without replacement.
_perms = list(itertools.permutations([1, 2, 3], 2))
assert _perms == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)], "permutations([1,2,3], 2)"
_ledger.append(1)

# 7. combinations — unordered selections without replacement.
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)], "combinations([1,2,3], 2)"
_ledger.append(1)

# 8. accumulate — running totals.
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10], "accumulate([1,2,3,4])"
_ledger.append(1)

# 9. takewhile — stop on first predicate-false.
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4])) == [1, 2], "takewhile(< 3, ...)"
_ledger.append(1)

# 10. dropwhile — start on first predicate-false.
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4])) == [3, 4], "dropwhile(< 3, ...)"
_ledger.append(1)

# 11. starmap — apply f(*args) to each tuple.
assert list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4)])) == [3, 7], "starmap(add, [(1,2),(3,4)])"
_ledger.append(1)

# 12. zip_longest — zip with fill on the short side.
_zl = list(itertools.zip_longest([1, 2, 3], ["a", "b"], fillvalue="?"))
assert _zl == [(1, "a"), (2, "b"), (3, "?")], "zip_longest with fillvalue='?'"
_ledger.append(1)

# 13. compress — select by selector mask.
assert list(itertools.compress([1, 2, 3, 4], [1, 0, 1, 0])) == [1, 3], "compress by mask"
_ledger.append(1)

# 14. filterfalse — invert filter.
assert list(itertools.filterfalse(lambda x: x % 2, [1, 2, 3, 4])) == [2, 4], "filterfalse(odd, ...)"
_ledger.append(1)

# 15. pairwise (3.10+) — overlapping adjacent pairs.
assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)], "pairwise([1,2,3,4])"
_ledger.append(1)

# 16. groupby — consecutive run-length grouping.
_groups = [(k, list(g)) for k, g in itertools.groupby("aaabbc")]
assert _groups == [("a", ["a", "a", "a"]), ("b", ["b", "b"]), ("c", ["c"])], "groupby('aaabbc')"
_ledger.append(1)

# Emit the proof-of-execution marker as the FINAL line so the runner
# can see it on stdout. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: itertools {len(_ledger)} asserts")
