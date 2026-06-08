# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_builtin_eval_exec_map_sum_itertools_silent"
# subject = "cpython321.lang_builtin_eval_exec_map_sum_itertools_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_builtin_eval_exec_map_sum_itertools_silent.py"
# status = "filled"
# ///
"""cpython321.lang_builtin_eval_exec_map_sum_itertools_silent: execute CPython 3.12 seed lang_builtin_eval_exec_map_sum_itertools_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of `type(vars).__name__ ==
# "builtin_function_or_method"` (the documented "vars is a built-in
# function" type contract — mamba silently binds vars to a user-level
# function, so the type-name returns 'function'), `eval("x*2", {"x":
# 5}) == 10` (the documented "eval honours the globals dict" value
# contract — mamba silently returns None when a globals dict is
# supplied), `exec("y = 10", ns)` (the documented "exec mutates the
# supplied globals dict" value contract — mamba silently leaves the
# dict unmodified), `list(map(lambda a, b: a + b, [1, 2], [10, 20]))
# == [11, 22]` (the documented "multi-iterable map applies the
# func across positional args" value contract — mamba silently
# returns floats `[1.0, 2.0]`), `sum([1.5, 2.5]) == 4.0` (the
# documented "sum of floats is a float" value contract — mamba
# silently returns the boxed-handle int 4616189618054758400),
# `list(it.chain.from_iterable([[1, 2], [3, 4]])) == [1, 2, 3, 4]`
# (the documented "chain.from_iterable flattens one level" value
# contract — mamba silently returns []), `list(it.islice(it.cycle(
# [1, 2, 3]), 5)) == [1, 2, 3, 1, 2]` (the documented
# "cycle repeats indefinitely" value contract — mamba silently
# returns []), `list(it.islice(it.count(10), 4)) == [10, 11, 12,
# 13]` (the documented "count starts at the given value and
# increments by 1" value contract — mamba silently returns []),
# and `list(it.islice(it.count(0, 5), 4)) == [0, 5, 10, 15]` (the
# documented "count with step accepts start/step args" value
# contract — mamba silently returns []). Ten-pack pinned to atomic
# 248.
#
# Behavioral edges that CONFORM on mamba (dir list type/non-empty;
# repr int/str/list; ascii nonascii/basic; ord/chr; eval simple +
# compile basic + callable fn/int/type; zip basic/uneven + map
# single-iter + filter basic/None + enumerate basic/start +
# reversed list/str; sorted basic/reverse/key; sum int basic/start;
# all/any true/false; min/max basic/key; range basic/start_stop/
# step/neg; itertools chain/repeat n/takewhile/dropwhile/groupby/
# product/combinations/permutations/tee/starmap/islice basic/
# zip_longest) are covered in the matching pass fixture
# `test_builtin_iter_aggregate_itertools_deep_value_ops`.
from typing import Any
import itertools as _itertools_mod
import builtins as _builtins_mod

it_mod: Any = _itertools_mod
builtins_mod: Any = _builtins_mod


_ledger: list[int] = []

# 1) vars — type contract is 'builtin_function_or_method'
#    (mamba: silently binds to user-level function, type-name == 'function')
assert type(builtins_mod.vars).__name__ == "builtin_function_or_method"; _ledger.append(1)

# 2) eval with globals dict
#    (mamba: silently returns None)
assert eval("x*2", {"x": 5}) == 10; _ledger.append(1)

# 3) exec mutates supplied globals dict
#    (mamba: silently leaves the dict unmodified)
_ns: dict = {}
exec("y = 10", _ns)
assert _ns.get("y") == 10; _ledger.append(1)

# 4) map with two iterables — multi-arg func
#    (mamba: silently returns [1.0, 2.0])
assert list(map(lambda a, b: a + b, [1, 2], [10, 20])) == [11, 22]; _ledger.append(1)

# 5) sum of floats
#    (mamba: silently returns boxed-handle int 4616189618054758400)
assert sum([1.5, 2.5]) == 4.0; _ledger.append(1)

# 6) itertools.chain.from_iterable — flatten one level
#    (mamba: silently returns [])
assert list(it_mod.chain.from_iterable([[1, 2], [3, 4]])) == [1, 2, 3, 4]; _ledger.append(1)

# 7) itertools.cycle — indefinite repeat
#    (mamba: silently returns [] after islice)
assert list(it_mod.islice(it_mod.cycle([1, 2, 3]), 5)) == [1, 2, 3, 1, 2]; _ledger.append(1)

# 8) itertools.count default start=0 step=1
#    (mamba: silently returns [] after islice)
assert list(it_mod.islice(it_mod.count(10), 4)) == [10, 11, 12, 13]; _ledger.append(1)

# 9) itertools.count with step arg
#    (mamba: silently returns [] after islice)
assert list(it_mod.islice(it_mod.count(0, 5), 4)) == [0, 5, 10, 15]; _ledger.append(1)

# 10) itertools.count default start (no args)
#     (mamba: silently returns [] after islice)
assert list(it_mod.islice(it_mod.count(), 3)) == [0, 1, 2]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_builtin_eval_exec_map_sum_itertools_silent {sum(_ledger)} asserts")
