"""Surface contract for language unpacking.

# type-regime: monomorphic

Probes: tuple unpacking, list unpacking, starred assignment,
swap idiom, nested unpacking, function arg unpacking.
CPython 3.12 is the oracle.
"""

# Basic tuple unpacking
_a, _b = (1, 2)
assert _a == 1, f"a = {_a!r}"
assert _b == 2, f"b = {_b!r}"

# Basic list unpacking
_x, _y, _z = [10, 20, 30]
assert _x == 10 and _y == 20 and _z == 30, f"list unpack = {_x},{_y},{_z}"

# Swap idiom
_p, _q = 7, 3
_p, _q = _q, _p
assert _p == 3 and _q == 7, f"swap = {_p},{_q}"

# Starred assignment — capture rest
_first, *_rest = [1, 2, 3, 4, 5]
assert _first == 1, f"first = {_first!r}"
assert _rest == [2, 3, 4, 5], f"rest = {_rest!r}"

*_head, _last = [1, 2, 3, 4, 5]
assert _head == [1, 2, 3, 4], f"head = {_head!r}"
assert _last == 5, f"last = {_last!r}"

_f2, *_mid, _l2 = [1, 2, 3, 4, 5]
assert _f2 == 1 and _l2 == 5, f"ends = {_f2},{_l2}"
assert _mid == [2, 3, 4], f"mid = {_mid!r}"

# Nested unpacking
(_na, (_nb, _nc)) = (1, (2, 3))
assert _na == 1 and _nb == 2 and _nc == 3, f"nested = {_na},{_nb},{_nc}"

# Unpacking in for loop
_pairs = [(1, "a"), (2, "b"), (3, "c")]
_keys = []
_vals = []
for _k, _v in _pairs:
    _keys.append(_k)
    _vals.append(_v)
assert _keys == [1, 2, 3], f"keys = {_keys!r}"
assert _vals == ["a", "b", "c"], f"vals = {_vals!r}"

# Unpacking from range
_i, _j = range(2)
assert _i == 0 and _j == 1, f"range unpack = {_i},{_j}"

# Starred in function call
def _add3(a: int, b: int, c: int) -> int:
    return a + b + c

_args = [1, 2, 3]
assert _add3(*_args) == 6, f"*args unpack = {_add3(*_args)!r}"

# Double-star in function call
def _greet(name: str, greeting: str = "hi") -> str:
    return f"{greeting} {name}"

_kw = {"name": "Alice", "greeting": "hello"}
assert _greet(**_kw) == "hello Alice", f"**kwargs unpack = {_greet(**_kw)!r}"

print("surface OK")
