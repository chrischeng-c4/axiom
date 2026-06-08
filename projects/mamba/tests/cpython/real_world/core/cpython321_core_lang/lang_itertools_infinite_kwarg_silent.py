# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_itertools_infinite_kwarg_silent"
# subject = "cpython321.lang_itertools_infinite_kwarg_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_itertools_infinite_kwarg_silent.py"
# status = "filled"
# ///
"""cpython321.lang_itertools_infinite_kwarg_silent: execute CPython 3.12 seed lang_itertools_infinite_kwarg_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in
# `itertools` — infinite-generator family, keyword-argument forms on
# accumulate/product, and chain.from_iterable. The matching subset
# (finite combinators) is covered by
# `test_itertools_finite_combinatorics_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • itertools.count(N) — should be an infinite int generator
#     starting at N (mamba returns an empty iterator);
#   • itertools.count(N, step) — same with step (mamba: empty);
#   • itertools.cycle(iterable) — should repeat the iterable forever
#     (mamba: empty);
#   • itertools.chain.from_iterable — should flatten one level (mamba:
#     empty);
#   • itertools.accumulate(iter, initial=N) — should include the
#     initial value (mamba ignores `initial`, returns wrong shape);
#   • itertools.product(iter, repeat=N) — should be the N-fold
#     Cartesian product (mamba ignores `repeat`, returns the
#     single-iterable form);
#   • itertools.product across 3+ positional iterables — should yield
#     N-tuples (mamba silently truncates to 2-tuples);
#   • itertools.accumulate(iter, max) — built-in `max` should work as
#     the binary fn (mamba: TypeError "object is not iterable", since
#     `max` is dispatched as varargs not (a, b)).
import itertools
from typing import Any

_ledger: list[int] = []

# 1) itertools.count — infinite int generator
_c5: Any = list(itertools.islice(itertools.count(5), 5))
assert _c5 == [5, 6, 7, 8, 9]; _ledger.append(1)
_c0: Any = list(itertools.islice(itertools.count(0), 4))
assert _c0 == [0, 1, 2, 3]; _ledger.append(1)
_c_neg: Any = list(itertools.islice(itertools.count(-3), 5))
assert _c_neg == [-3, -2, -1, 0, 1]; _ledger.append(1)

# 2) itertools.count(start, step) — with custom step
_c_step: Any = list(itertools.islice(itertools.count(0, 2), 5))
assert _c_step == [0, 2, 4, 6, 8]; _ledger.append(1)
_c_step_neg: Any = list(itertools.islice(itertools.count(10, -1), 4))
assert _c_step_neg == [10, 9, 8, 7]; _ledger.append(1)
_c_step_skip: Any = list(itertools.islice(itertools.count(0, 5), 4))
assert _c_step_skip == [0, 5, 10, 15]; _ledger.append(1)

# 3) itertools.cycle — repeat forever
_cyc_abc: Any = list(itertools.islice(itertools.cycle("abc"), 7))
assert _cyc_abc == ["a", "b", "c", "a", "b", "c", "a"]; _ledger.append(1)
_cyc_int: Any = list(itertools.islice(itertools.cycle([1, 2, 3]), 7))
assert _cyc_int == [1, 2, 3, 1, 2, 3, 1]; _ledger.append(1)
_cyc_one: Any = list(itertools.islice(itertools.cycle(["only"]), 3))
assert _cyc_one == ["only", "only", "only"]; _ledger.append(1)

# 4) itertools.chain.from_iterable — flattens one level
_cfi_int: Any = list(itertools.chain.from_iterable([[1, 2], [3, 4]]))
assert _cfi_int == [1, 2, 3, 4]; _ledger.append(1)
_cfi_str: Any = list(itertools.chain.from_iterable(["AB", "CD"]))
assert _cfi_str == ["A", "B", "C", "D"]; _ledger.append(1)
_cfi_3: Any = list(itertools.chain.from_iterable([[1], [2, 3], [4, 5, 6]]))
assert _cfi_3 == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
_cfi_empty: Any = list(itertools.chain.from_iterable([]))
assert _cfi_empty == []; _ledger.append(1)
_cfi_mixed_empty: Any = list(itertools.chain.from_iterable([[], [1], []]))
assert _cfi_mixed_empty == [1]; _ledger.append(1)

# 5) itertools.accumulate with initial= kwarg
_ai_10: Any = list(itertools.accumulate([1, 2, 3], initial=10))
assert _ai_10 == [10, 11, 13, 16]; _ledger.append(1)
_ai_0: Any = list(itertools.accumulate([5, 3, 2], initial=0))
assert _ai_0 == [0, 5, 8, 10]; _ledger.append(1)
_ai_empty: Any = list(itertools.accumulate([], initial=99))
assert _ai_empty == [99]; _ledger.append(1)

# 6) itertools.product with repeat= kwarg
_pr_2: Any = list(itertools.product([0, 1], repeat=2))
assert _pr_2 == [(0, 0), (0, 1), (1, 0), (1, 1)]; _ledger.append(1)
_pr_3: Any = list(itertools.product([0, 1], repeat=3))
assert _pr_3 == [(0, 0, 0), (0, 0, 1), (0, 1, 0), (0, 1, 1), (1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)]; _ledger.append(1)
_pr_AB: Any = list(itertools.product("AB", repeat=2))
assert _pr_AB == [("A", "A"), ("A", "B"), ("B", "A"), ("B", "B")]; _ledger.append(1)

# 7) itertools.product across 3+ positional iterables — yields N-tuples
_p_3: Any = list(itertools.product([1], [2], [3]))
assert _p_3 == [(1, 2, 3)]; _ledger.append(1)
_p_3_multi: Any = list(itertools.product([1, 2], [3, 4], [5]))
assert _p_3_multi == [(1, 3, 5), (1, 4, 5), (2, 3, 5), (2, 4, 5)]; _ledger.append(1)
_p_4: Any = list(itertools.product([1], [2], [3], [4]))
assert _p_4 == [(1, 2, 3, 4)]; _ledger.append(1)

# 8) itertools.accumulate(iter, max) — built-in max as binary fn
_am: Any = list(itertools.accumulate([1, 5, 3, 7, 2], max))
assert _am == [1, 5, 5, 7, 7]; _ledger.append(1)
_am_min: Any = list(itertools.accumulate([5, 3, 7, 2], min))
assert _am_min == [5, 3, 3, 2]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_itertools_infinite_kwarg_silent {sum(_ledger)} asserts")
