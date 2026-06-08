"""Behavior contract for builtins.repr.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: repr of scalars
assert repr(0) == "0", f"repr(0) = {repr(0)!r}"
assert repr(-1) == "-1", f"repr(-1) = {repr(-1)!r}"
assert repr(3.14) == "3.14", f"repr(3.14) = {repr(3.14)!r}"
assert repr(True) == "True", f"repr(True) = {repr(True)!r}"
assert repr(False) == "False", f"repr(False) = {repr(False)!r}"
assert repr(None) == "None", f"repr(None) = {repr(None)!r}"

# Rule 2: repr of str — adds quotes and escapes
assert repr("hello") == "'hello'", f"repr('hello') = {repr('hello')!r}"
_its = repr("it's")
assert _its in ('"it\'s"', '"it\'s"', "\"it's\""), f"repr(\"it's\") = {_its!r}"
assert repr("a\nb") == "'a\\nb'", f"repr('a\\nb') = {repr('a' + chr(10) + 'b')!r}"

# Rule 3: repr of bytes
assert repr(b"abc") == "b'abc'", f"repr(b'abc') = {repr(b'abc')!r}"
assert repr(b"\xff") == "b'\\xff'", f"repr(b'\\xff') = {repr(b'\\xff')!r}"

# Rule 4: repr of list / tuple / dict / set
assert repr([1, 2, 3]) == "[1, 2, 3]", f"repr([1,2,3]) = {repr([1,2,3])!r}"
assert repr((1, 2)) == "(1, 2)", f"repr((1,2)) = {repr((1,2))!r}"
assert repr((1,)) == "(1,)", f"repr((1,)) = {repr((1,))!r}"
assert repr({}) == "{}", f"repr({{}}) = {repr({})!r}"

# Rule 5: custom __repr__
class _MyObj:
    def __repr__(self):
        return "MyObj()"
assert repr(_MyObj()) == "MyObj()", f"custom __repr__ = {repr(_MyObj())!r}"

# Rule 6: repr of nested structures
assert repr([[1, 2], [3, 4]]) == "[[1, 2], [3, 4]]", \
    f"repr(nested) = {repr([[1,2],[3,4]])!r}"

# Rule 7: repr(float) special values
import math
assert repr(float("inf")) == "inf", f"repr(inf) = {repr(float('inf'))!r}"
assert repr(float("-inf")) == "-inf", f"repr(-inf) = {repr(float('-inf'))!r}"
assert repr(float("nan")) == "nan", f"repr(nan) = {repr(float('nan'))!r}"

print("behavior OK")
