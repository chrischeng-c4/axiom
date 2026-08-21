# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_builtin_iter_aggregate_itertools_deep_value_ops"
# subject = "cpython321.test_builtin_iter_aggregate_itertools_deep_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_builtin_iter_aggregate_itertools_deep_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_builtin_iter_aggregate_itertools_deep_value_ops: execute CPython 3.12 seed test_builtin_iter_aggregate_itertools_deep_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 248 pass conformance — built-in type introspection (dir list type/
# dir non-empty / repr int/str/list / ascii nonascii/basic / ord A/0 /
# chr 65/0x41) + built-in call ops (eval simple / compile basic / callable
# fn/int/type) + built-in iter helpers (zip basic/uneven / map basic /
# filter basic/None / enumerate basic/start / reversed list/str) +
# built-in sort/aggregate (sorted basic/reverse/key / sum basic/start /
# all true/false / any true/false / min basic/key / max basic/key / range
# basic/start_stop/step/neg) + itertools deeper (chain / repeat n /
# takewhile / dropwhile / groupby / product 2-arity / combinations /
# permutations / tee / starmap / islice basic / zip_longest) that match
# between CPython 3.12 and mamba.
import itertools as it


_ledger: list[int] = []

# 1) built-in type introspection
assert type(dir([])).__name__ == "list"; _ledger.append(1)
assert len(dir([])) > 0; _ledger.append(1)
assert repr(42) == "42"; _ledger.append(1)
assert repr("hi") == "'hi'"; _ledger.append(1)
assert repr([1, 2]) == "[1, 2]"; _ledger.append(1)
assert ascii("café") == "'caf\\xe9'"; _ledger.append(1)
assert ascii("hi") == "'hi'"; _ledger.append(1)
assert ord("A") == 65; _ledger.append(1)
assert ord("0") == 48; _ledger.append(1)
assert chr(65) == "A"; _ledger.append(1)
assert chr(0x41) == "A"; _ledger.append(1)

# 2) built-in eval / compile / callable
assert eval("1+2") == 3; _ledger.append(1)
assert type(compile("1+2", "<s>", "eval")).__name__ == "code"; _ledger.append(1)
assert callable(lambda: 1) == True; _ledger.append(1)
assert callable(42) == False; _ledger.append(1)
assert callable(list) == True; _ledger.append(1)

# 3) zip / map / filter / enumerate / reversed
assert list(zip([1, 2, 3], ["a", "b", "c"])) == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
assert list(zip([1, 2, 3], ["a", "b"])) == [(1, "a"), (2, "b")]; _ledger.append(1)
assert list(map(str, [1, 2, 3])) == ["1", "2", "3"]; _ledger.append(1)
assert list(filter(lambda x: x > 1, [0, 1, 2, 3])) == [2, 3]; _ledger.append(1)
assert list(filter(None, [0, 1, "", "a", None])) == [1, "a"]; _ledger.append(1)
assert list(enumerate(["a", "b", "c"])) == [(0, "a"), (1, "b"), (2, "c")]; _ledger.append(1)
assert list(enumerate(["a", "b"], start=10)) == [(10, "a"), (11, "b")]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)

# 4) sorted
assert sorted([3, 1, 2]) == [1, 2, 3]; _ledger.append(1)
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]; _ledger.append(1)
assert sorted(["bb", "ccc", "a"], key=len) == ["a", "bb", "ccc"]; _ledger.append(1)

# 5) sum int
assert sum([1, 2, 3]) == 6; _ledger.append(1)
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)

# 6) all / any
assert all([1, 2, 3]) == True; _ledger.append(1)
assert all([1, 0, 3]) == False; _ledger.append(1)
assert any([0, 1, 0]) == True; _ledger.append(1)
assert any([0, 0, 0]) == False; _ledger.append(1)

# 7) min / max
assert min([3, 1, 2]) == 1; _ledger.append(1)
assert min(["bb", "ccc", "a"], key=len) == "a"; _ledger.append(1)
assert max([3, 1, 2]) == 3; _ledger.append(1)
assert max(["bb", "ccc", "a"], key=len) == "ccc"; _ledger.append(1)

# 8) range
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(range(2, 6)) == [2, 3, 4, 5]; _ledger.append(1)
assert list(range(0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(range(5, 0, -1)) == [5, 4, 3, 2, 1]; _ledger.append(1)

# 9) itertools deeper — chain / repeat
assert list(it.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
assert list(it.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)

# 10) itertools deeper — takewhile / dropwhile
assert list(it.takewhile(lambda x: x < 3, [1, 2, 3, 1])) == [1, 2]; _ledger.append(1)
assert list(it.dropwhile(lambda x: x < 3, [1, 2, 3, 1])) == [3, 1]; _ledger.append(1)

# 11) itertools deeper — groupby
assert [(k, list(g)) for k, g in it.groupby([1, 1, 2, 3, 3, 3])] == [(1, [1, 1]), (2, [2]), (3, [3, 3, 3])]; _ledger.append(1)

# 12) itertools deeper — product / combinations / permutations
assert list(it.product([1, 2], ["a", "b"])) == [(1, "a"), (1, "b"), (2, "a"), (2, "b")]; _ledger.append(1)
assert list(it.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
assert list(it.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)

# 13) itertools deeper — tee
_a, _b = it.tee([1, 2, 3], 2)
assert (list(_a), list(_b)) == ([1, 2, 3], [1, 2, 3]); _ledger.append(1)

# 14) itertools deeper — starmap / islice / zip_longest
assert list(it.starmap(lambda x, y: x + y, [(1, 2), (3, 4)])) == [3, 7]; _ledger.append(1)
assert list(it.islice([0, 1, 2, 3, 4], 2, 4)) == [2, 3]; _ledger.append(1)
assert list(it.zip_longest([1, 2, 3], ["a", "b"], fillvalue="*")) == [(1, "a"), (2, "b"), (3, "*")]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_builtin_iter_aggregate_itertools_deep_value_ops {sum(_ledger)} asserts")
