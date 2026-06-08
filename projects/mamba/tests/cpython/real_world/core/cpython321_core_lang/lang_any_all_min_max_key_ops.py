# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_any_all_min_max_key_ops"
# subject = "cpython321.lang_any_all_min_max_key_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_any_all_min_max_key_ops.py"
# status = "filled"
# ///
"""cpython321.lang_any_all_min_max_key_ops: execute CPython 3.12 seed lang_any_all_min_max_key_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the truthiness aggregators
# `any` / `all` and the key-function form of `min` / `max`. Surface:
# `any(iterable)` returns True iff at least one element is truthy;
# the empty-iterable case returns False. `all(iterable)` returns True
# iff every element is truthy; the empty-iterable case returns True
# (vacuous truth). Both short-circuit at the first decisive element.
# `min(iter, key=fn)` and `max(iter, key=fn)` select the element that
# minimises/maximises `fn(elem)` rather than the element itself. The
# `key=` callable is invoked once per element. Slice assignment
# (`L[a:b] = iter`) replaces the slice in place: the slice can shrink
# (assign shorter iter), grow (assign longer iter), or be emptied
# (assign `[]`). Sequence multiplication `[1, 2] * 3` produces a
# fresh repeated list; the same for tuples. Both are zero-respecting:
# `seq * 0 == []`.
_ledger: list[int] = []

# any — at least one truthy element
assert any([False, True, False]) == True; _ledger.append(1)
assert any([False, False]) == False; _ledger.append(1)
assert any([]) == False; _ledger.append(1)
assert any([0, 0, 1]) == True; _ledger.append(1)
assert any([0, "", None]) == False; _ledger.append(1)

# all — every element truthy
assert all([True, True, True]) == True; _ledger.append(1)
assert all([True, False, True]) == False; _ledger.append(1)
assert all([]) == True; _ledger.append(1)  # vacuous truth
assert all([1, 2, 3]) == True; _ledger.append(1)
assert all([1, 0, 3]) == False; _ledger.append(1)

# min / max with key= selector
assert min(["abc", "de", "f"], key=len) == "f"; _ledger.append(1)
assert max(["abc", "de", "f"], key=len) == "abc"; _ledger.append(1)
assert min([-3, 1, -4, 2], key=abs) == 1; _ledger.append(1)
assert max([-3, 1, -4, 2], key=abs) == -4; _ledger.append(1)

# Slice assignment — equal-size replacement
L = [1, 2, 3, 4, 5]
L[1:4] = [20, 30, 40]
assert L == [1, 20, 30, 40, 5]; _ledger.append(1)

# Slice assignment — shrink (assign empty list)
L[1:4] = []
assert L == [1, 5]; _ledger.append(1)

# Slice assignment — grow (assign longer iter)
L2 = [1, 5]
L2[1:1] = [2, 3, 4]
assert L2 == [1, 2, 3, 4, 5]; _ledger.append(1)

# Sequence multiplication on lists
assert [1, 2] * 3 == [1, 2, 1, 2, 1, 2]; _ledger.append(1)
assert [1, 2, 3] * 0 == []; _ledger.append(1)

# Sequence multiplication on tuples
assert (1, 2) * 3 == (1, 2, 1, 2, 1, 2); _ledger.append(1)
assert (1, 2) * 0 == (); _ledger.append(1)

# List concatenation +
assert [1, 2, 3] + [4, 5] == [1, 2, 3, 4, 5]; _ledger.append(1)
assert [] + [1] == [1]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_any_all_min_max_key_ops {sum(_ledger)} asserts")
