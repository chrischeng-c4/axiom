# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_list_sort_key_ops"
# subject = "cpython321.test_list_sort_key_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_list_sort_key_ops.py"
# status = "filled"
# ///
"""cpython321.test_list_sort_key_ops: execute CPython 3.12 seed test_list_sort_key_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for sorted / list.sort / min / max
# with a `key=` callable, plus any / all.
# Surface: sorted(key=) orders by callable output; list.sort(reverse=)
# mutates in place and respects reverse; min(key=) / max(key=)
# select the extreme element under the key; any/all collapse a
# boolean iterable.
_ledger: list[int] = []
words = ["banana", "fig", "apple"]
# sorted with key=len orders by length, stably
assert sorted(words, key=len) == ["fig", "apple", "banana"]; _ledger.append(1)
# default sorted on strings is lexicographic
assert sorted(words) == ["apple", "banana", "fig"]; _ledger.append(1)
# list.sort mutates in place
nums = [3, 1, 4, 1, 5]
nums.sort()
assert nums == [1, 1, 3, 4, 5]; _ledger.append(1)
# sort(reverse=True) inverts the order in place
nums.sort(reverse=True)
assert nums == [5, 4, 3, 1, 1]; _ledger.append(1)
# min / max with key= select by callable output
assert min(words, key=len) == "fig"; _ledger.append(1)
assert max(words, key=len) == "banana"; _ledger.append(1)
# any returns True iff at least one truthy element
assert any([0, 0, 1]); _ledger.append(1)
assert not any([0, 0, 0]); _ledger.append(1)
# any of an empty iterable is False
assert not any([]); _ledger.append(1)
# all returns True iff every element is truthy
assert all([1, 1, 1]); _ledger.append(1)
assert not all([1, 0, 1]); _ledger.append(1)
# all of an empty iterable is True (vacuous truth)
assert all([]); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_list_sort_key_ops {sum(_ledger)} asserts")
