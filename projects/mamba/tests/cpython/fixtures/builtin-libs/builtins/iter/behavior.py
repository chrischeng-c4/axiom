"""Behavior contract for builtins.iter.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: iter(list) returns an iterator
_it = iter([1, 2, 3])
assert hasattr(_it, "__next__"), \
    f"iter(list) has no __next__: type={type(_it).__name__}"

# Rule 2: consuming iter(list)
_it = iter([10, 20, 30])
assert next(_it) == 10
assert next(_it) == 20
assert next(_it) == 30

# Rule 3: StopIteration when exhausted
_it = iter([1])
next(_it)
_raised = False
try:
    next(_it)
except StopIteration:
    _raised = True
assert _raised, "iter([1]) did not raise StopIteration when exhausted"

# Rule 4: iter(list) is not the list itself
_lst = [1, 2, 3]
assert iter(_lst) is not _lst, "iter(list) returned the list itself"

# Rule 5: iter(iter) returns the same object (idempotent)
_it = iter([1, 2])
assert iter(_it) is _it, "iter(iterator) is not idempotent"

# Rule 6: iter(tuple)
_it = iter((10, 20))
assert list(_it) == [10, 20], f"iter(tuple) contents = {list(iter((10,20)))!r}"

# Rule 7: iter(str)
_it = iter("abc")
assert list(_it) == ["a", "b", "c"], f"iter('abc') = {list(iter('abc'))!r}"

# Rule 8: iter(dict) yields keys
_it = iter({"a": 1, "b": 2})
assert sorted(_it) == ["a", "b"], f"iter(dict) keys = {sorted(iter({'a':1,'b':2}))!r}"

# Rule 9: iter(set)
_it = iter({1, 2, 3})
assert sorted(_it) == [1, 2, 3], f"iter(set) = {sorted(iter({1,2,3}))!r}"

# Rule 10: iter(non-iterable) raises TypeError
_raised = False
try:
    iter(42)
except TypeError:
    _raised = True
assert _raised, "iter(42) did not raise TypeError"

# Rule 11: two-arg form: iter(callable, sentinel)
_data = iter([1, 2, 3, 0, 5])
_result = list(iter(lambda: next(_data), 0))
assert _result == [1, 2, 3], f"iter(callable, sentinel) = {_result!r}"

print("behavior OK")
