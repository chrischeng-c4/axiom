"""Behavior contract for builtins.reversed.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: reversed of list
result = list(reversed([1, 2, 3, 4, 5]))
assert result == [5, 4, 3, 2, 1], \
    f"list(reversed([1,2,3,4,5])) = {result!r}, expected [5,4,3,2,1]"

# Rule 2: reversed of tuple
result = list(reversed((10, 20, 30)))
assert result == [30, 20, 10], \
    f"list(reversed((10,20,30))) = {result!r}, expected [30,20,10]"

# Rule 3: reversed of range
result = list(reversed(range(5)))
assert result == [4, 3, 2, 1, 0], \
    f"list(reversed(range(5))) = {result!r}, expected [4,3,2,1,0]"

# Rule 4: reversed of string
result = list(reversed("abc"))
assert result == ["c", "b", "a"], \
    f"list(reversed('abc')) = {result!r}, expected ['c','b','a']"

# Rule 5: reversed of empty list
result = list(reversed([]))
assert result == [], f"list(reversed([])) = {result!r}, expected []"

# Rule 6: reversed returns an iterator (not a list directly)
_it = reversed([1, 2, 3])
assert hasattr(_it, "__next__"), \
    f"reversed([1,2,3]) has no __next__: type={type(_it).__name__}"

# Rule 7: reversed iterator is exhausted after full consumption
_it2 = reversed([1, 2])
next(_it2)
next(_it2)
_raised = False
try:
    next(_it2)
except StopIteration:
    _raised = True
assert _raised, "reversed iterator did not raise StopIteration when exhausted"

# Rule 8: reversed of dict yields keys in insertion order reversed (Python 3.8+)
result = list(reversed({"a": 1, "b": 2, "c": 3}))
assert result == ["c", "b", "a"], \
    f"list(reversed(dict)) = {result!r}, expected ['c','b','a']"

# Rule 9: custom __reversed__ is respected
class _C:
    def __reversed__(self):
        return iter([99, 88, 77])

result = list(reversed(_C()))
assert result == [99, 88, 77], \
    f"list(reversed(custom __reversed__)) = {result!r}, expected [99,88,77]"

# Rule 10: reversed of single-element list
result = list(reversed([42]))
assert result == [42], f"list(reversed([42])) = {result!r}, expected [42]"

print("behavior OK")
