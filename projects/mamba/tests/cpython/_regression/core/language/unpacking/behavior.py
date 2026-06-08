"""Behavior contract for language unpacking.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: Unpacking assigns values left-to-right
_a, _b, _c = 10, 20, 30
assert _a == 10, f"a = {_a!r}"
assert _b == 20, f"b = {_b!r}"
assert _c == 30, f"c = {_c!r}"

# Rule 2: Wrong number of values raises ValueError
_raised = False
try:
    _x, _y = (1, 2, 3)
except ValueError:
    _raised = True
assert _raised, "too many values should raise ValueError"

_raised2 = False
try:
    _x, _y, _z = (1, 2)
except ValueError:
    _raised2 = True
assert _raised2, "not enough values should raise ValueError"

# Rule 3: Starred expression captures remaining items as list
_first, *_rest = range(5)
assert _first == 0, f"first = {_first!r}"
assert _rest == [1, 2, 3, 4], f"rest = {_rest!r}"
assert isinstance(_rest, list), f"*rest type = {type(_rest)!r}"

# Rule 4: Starred with nothing gets empty list
_only, *_empty = [42]
assert _only == 42, f"only = {_only!r}"
assert _empty == [], f"empty = {_empty!r}"

# Rule 5: Augmented tuple unpacking with single element requires trailing comma
_tup = (99,)
(_val,) = _tup
assert _val == 99, f"single unpack = {_val!r}"

# Rule 6: Nested unpacking mirrors structure
((_aa, _bb), _cc) = ((1, 2), 3)
assert _aa == 1 and _bb == 2 and _cc == 3, f"nested = {_aa},{_bb},{_cc}"

# Rule 7: Unpacking works with any iterable
_p, _q, _r = "abc"
assert _p == "a" and _q == "b" and _r == "c", f"str unpack = {_p},{_q},{_r}"

_i, _j = {10: "x", 20: "y"}  # iterates over keys
assert {_i, _j} == {10, 20}, f"dict key unpack = {_i},{_j}"

# Rule 8: List literal unpacking in for
_result = []
for _n, _m in [(1, 2), (3, 4), (5, 6)]:
    _result.append(_n + _m)
assert _result == [3, 7, 11], f"loop unpack = {_result!r}"

# Rule 9: *args in function pass-through
def _sum3(a: int, b: int, c: int) -> int:
    return a + b + c

_vals = (4, 5, 6)
assert _sum3(*_vals) == 15, f"*args sum = {_sum3(*_vals)!r}"

# Rule 10: **kwargs in function pass-through
def _fmt(name: str, age: int) -> str:
    return f"{name}:{age}"

_kw = {"name": "Bob", "age": 30}
assert _fmt(**_kw) == "Bob:30", f"**kw = {_fmt(**_kw)!r}"

# Rule 11: Mixed *args and **kwargs
def _mixed(a: int, b: int, c: int = 0) -> int:
    return a + b + c

assert _mixed(*[1, 2], **{"c": 3}) == 6, f"mixed = {_mixed(*[1,2], **{'c':3})!r}"

print("behavior OK")
