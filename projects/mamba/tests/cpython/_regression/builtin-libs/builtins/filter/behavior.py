"""Behavior contract for builtins.filter.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: filter with predicate
result = list(filter(lambda x: x > 2, [1, 2, 3, 4, 5]))
assert result == [3, 4, 5], f"filter(x>2, [1..5]) = {result!r}"

# Rule 2: filter with None keeps truthy values
result = list(filter(None, [0, 1, 2, 0, 3, False, True]))
assert result == [1, 2, 3, True], f"filter(None, ...) = {result!r}"

# Rule 3: filter of empty returns empty
result = list(filter(lambda x: x > 0, []))
assert result == [], f"filter(empty) = {result!r}, expected []"

# Rule 4: filter returns iterator (has __next__)
_it = filter(lambda x: x > 0, [1, 2])
assert hasattr(_it, "__next__"), \
    f"filter() result has no __next__: type={type(_it).__name__}"

# Rule 5: filter is lazy — predicate not called until iterated
# (predicate always returns True so laziness is isolated from filtering logic)
_calls = []
def _pred(x):
    _calls.append(x)
    return True
_f = filter(_pred, [1, 2, 3])
assert _calls == [], f"filter eager-evaluated before iteration: {_calls!r}"
next(_f)
assert len(_calls) >= 1, f"filter called wrong number of times: {_calls!r}"

# Rule 6: all elements pass (nothing filtered)
result = list(filter(lambda x: True, [1, 2, 3]))
assert result == [1, 2, 3], f"filter keep-all = {result!r}"

# Rule 7: no elements pass (all filtered)
result = list(filter(lambda x: False, [1, 2, 3]))
assert result == [], f"filter remove-all = {result!r}"

# Rule 8: filter over strings
result = list(filter(lambda s: len(s) > 3, ["hi", "hello", "hey", "world"]))
assert result == ["hello", "world"], f"filter strings = {result!r}"

# Rule 9: filter over range
result = list(filter(lambda x: x % 2 == 0, range(10)))
assert result == [0, 2, 4, 6, 8], f"filter range evens = {result!r}"

# Rule 10: filter with None over string (keeps non-empty chars)
result = list(filter(None, ["", "a", "", "b", ""]))
assert result == ["a", "b"], f"filter(None, strs) = {result!r}"

print("behavior OK")
