"""Behavior contract for builtins.len.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: len of list
result = len([1, 2, 3])
assert result == 3, f"len([1,2,3]) = {result!r}, expected 3"

# Rule 2: len of empty list
result = len([])
assert result == 0, f"len([]) = {result!r}, expected 0"

# Rule 3: len of tuple
result = len((10, 20, 30, 40))
assert result == 4, f"len((10,20,30,40)) = {result!r}, expected 4"

# Rule 4: len of str
result = len("hello")
assert result == 5, f"len('hello') = {result!r}, expected 5"

# Rule 5: len of empty str
result = len("")
assert result == 0, f"len('') = {result!r}, expected 0"

# Rule 6: len of bytes
result = len(b"abc")
assert result == 3, f"len(b'abc') = {result!r}, expected 3"

# Rule 7: len of dict
result = len({"a": 1, "b": 2})
assert result == 2, f"len({{'a':1,'b':2}}) = {result!r}, expected 2"

# Rule 8: len of set
result = len({1, 2, 3, 4})
assert result == 4, f"len({{1,2,3,4}}) = {result!r}, expected 4"

# Rule 9: len of frozenset
result = len(frozenset([1, 2, 3]))
assert result == 3, f"len(frozenset([1,2,3])) = {result!r}, expected 3"

# Rule 10: len returns int (not bool, not float)
result = len([1, 2])
assert type(result) is int, \
    f"type(len([1,2])) = {type(result).__name__!r}, expected 'int'"

# Rule 11: len of range
result = len(range(10))
assert result == 10, f"len(range(10)) = {result!r}, expected 10"

# Rule 12: len of range with step
result = len(range(0, 10, 2))
assert result == 5, f"len(range(0,10,2)) = {result!r}, expected 5"

# Rule 13: len raises TypeError for non-sequence (int has no len)
_raised = False
try:
    len(42)
except TypeError:
    _raised = True
assert _raised, "len(42) did not raise TypeError"

# Rule 14: len raises TypeError for None
_raised = False
try:
    len(None)
except TypeError:
    _raised = True
assert _raised, "len(None) did not raise TypeError"

# Rule 15: custom __len__ is respected
class _C:
    def __len__(self):
        return 7

result = len(_C())
assert result == 7, f"len(custom __len__) = {result!r}, expected 7"

# Rule 16: __len__ returning non-int raises TypeError in CPython
class _Bad:
    def __len__(self):
        return "not-an-int"

_raised = False
try:
    len(_Bad())
except TypeError:
    _raised = True
assert _raised, "len(obj with __len__ returning str) did not raise TypeError"

print("behavior OK")
