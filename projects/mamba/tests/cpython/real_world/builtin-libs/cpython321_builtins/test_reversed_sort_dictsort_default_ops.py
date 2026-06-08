# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_reversed_sort_dictsort_default_ops"
# subject = "cpython321.test_reversed_sort_dictsort_default_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_reversed_sort_dictsort_default_ops.py"
# status = "filled"
# ///
"""cpython321.test_reversed_sort_dictsort_default_ops: execute CPython 3.12 seed test_reversed_sort_dictsort_default_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed extending the sort/min/max
# surface in directions not already covered by
# test_sorted_key_ops.py / test_list_sort_key_ops.py /
# lang_any_all_min_max_key_ops.py. This seed asserts:
# reversed() over list / range / tuple / str returning an
# iterator that materialises to the reversed sequence;
# list.reverse() in-place mutation (including the empty-list
# no-op); sorted() over dict.items() by value-projection via
# `lambda kv: kv[1]`; sorted() over the bare dict iterating
# keys by external-table lookup; multi-key tuple sort
# combining ascending and descending axes via `(-t[0], t[1])`;
# stable-sort retention of original input order on equal keys
# under both sorted() and list.sort(); list.sort() in-place
# with key= and reverse= combined; min()/max() multi-arg
# positional form with key=; min()/max() default= kwarg
# returning the default when the iterable is empty.
_ledger: list[int] = []

# reversed() over list — iterator materialises to reversed list
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed([42])) == [42]; _ledger.append(1)
assert list(reversed([])) == []; _ledger.append(1)
assert list(reversed([1, 2, 3, 4, 5])) == [5, 4, 3, 2, 1]; _ledger.append(1)

# reversed() over range
assert list(reversed(range(5))) == [4, 3, 2, 1, 0]; _ledger.append(1)
assert list(reversed(range(3))) == [2, 1, 0]; _ledger.append(1)
assert list(reversed(range(0))) == []; _ledger.append(1)

# reversed() over tuple
assert list(reversed((1, 2, 3))) == [3, 2, 1]; _ledger.append(1)
assert list(reversed(("a", "b", "c"))) == ["c", "b", "a"]; _ledger.append(1)

# reversed() over str
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)
assert list(reversed("a")) == ["a"]; _ledger.append(1)
assert list(reversed("")) == []; _ledger.append(1)

# list.reverse() in-place
xs = [1, 2, 3]
xs.reverse()
assert xs == [3, 2, 1]; _ledger.append(1)

xs2 = [1]
xs2.reverse()
assert xs2 == [1]; _ledger.append(1)

xs3: list[int] = []
xs3.reverse()
assert xs3 == []; _ledger.append(1)

xs4 = ["a", "b", "c", "d"]
xs4.reverse()
assert xs4 == ["d", "c", "b", "a"]; _ledger.append(1)

# sorted() over dict.items() by value-projection
d = {"a": 3, "b": 1, "c": 2}
assert sorted(d.items(), key=lambda kv: kv[1]) == [("b", 1), ("c", 2), ("a", 3)]; _ledger.append(1)

# sorted() over bare dict iterating keys by external-table lookup
d2 = {"x": 30, "y": 10, "z": 20}
assert sorted(d2, key=lambda k: d2[k]) == ["y", "z", "x"]; _ledger.append(1)

# Multi-key tuple sort — ascending then descending via negated axis
data = [(1, "b"), (2, "a"), (1, "a"), (2, "b")]
assert sorted(data, key=lambda t: (-t[0], t[1])) == [(2, "a"), (2, "b"), (1, "a"), (1, "b")]; _ledger.append(1)
# Multi-key tuple sort — pure ascending tuple
assert sorted(data, key=lambda t: (t[0], t[1])) == [(1, "a"), (1, "b"), (2, "a"), (2, "b")]; _ledger.append(1)

# Stable sort via sorted() — equal keys keep input order
items = [(1, "x"), (2, "y"), (1, "z"), (2, "w")]
assert sorted(items, key=lambda t: t[0]) == [(1, "x"), (1, "z"), (2, "y"), (2, "w")]; _ledger.append(1)

# Stable sort via list.sort() — equal keys keep input order
items2 = [(1, "x"), (2, "y"), (1, "z"), (2, "w")]
items2.sort(key=lambda t: t[0])
assert items2 == [(1, "x"), (1, "z"), (2, "y"), (2, "w")]; _ledger.append(1)

# list.sort() in-place with key= and reverse=
words = ["banana", "fig", "apple", "kiwi"]
words.sort(key=len, reverse=True)
assert words == ["banana", "apple", "kiwi", "fig"]; _ledger.append(1)

nums = [3, -1, -4, 1, 2]
nums.sort(key=abs)
assert nums == [-1, 1, 2, 3, -4]; _ledger.append(1)

nums2 = [3, 1, 4, 1, 5]
nums2.sort(reverse=True)
assert nums2 == [5, 4, 3, 1, 1]; _ledger.append(1)

# list.sort() on empty list
empty: list[int] = []
empty.sort()
assert empty == []; _ledger.append(1)

# min()/max() multi-arg positional form
assert min(3, 1, 4) == 1; _ledger.append(1)
assert max(3, 1, 4) == 4; _ledger.append(1)
assert min(3, 1, 4, 1, 5, 9, 2, 6) == 1; _ledger.append(1)
assert max(3, 1, 4, 1, 5, 9, 2, 6) == 9; _ledger.append(1)

# min()/max() multi-arg positional form with key=
assert min(-3, 1, -4, key=abs) == 1; _ledger.append(1)
assert max(-3, 1, -4, key=abs) == -4; _ledger.append(1)
assert min("apple", "fig", "banana", key=len) == "fig"; _ledger.append(1)
assert max("apple", "fig", "banana", key=len) == "banana"; _ledger.append(1)

# min()/max() default= kwarg over empty iterable
assert min([], default=99) == 99; _ledger.append(1)
assert max([], default=99) == 99; _ledger.append(1)
assert min([], default="sentinel") == "sentinel"; _ledger.append(1)
assert max([], default=-1) == -1; _ledger.append(1)
assert min([], default=None) is None; _ledger.append(1)

# min()/max() over non-empty iterable still ignore default=
assert min([3, 1, 4], default=99) == 1; _ledger.append(1)
assert max([3, 1, 4], default=99) == 4; _ledger.append(1)

# sorted() returns a new list, leaves input untouched
src = [3, 1, 2]
out = sorted(src)
assert out == [1, 2, 3]; _ledger.append(1)
assert src == [3, 1, 2]; _ledger.append(1)

# reversed() returns a fresh iterator each call
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_reversed_sort_dictsort_default_ops {sum(_ledger)} asserts")
