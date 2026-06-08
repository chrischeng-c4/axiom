"""Surface contract for language walrus operator (:=).

# type-regime: monomorphic

Probes: assignment expression in while, if, comprehension, return.
CPython 3.12 is the oracle. Walrus operator is PEP 572.
"""

# Basic walrus in while loop
_data = [1, 2, 3, 4, 5]
_idx = 0
_seen = []
while (_val := _data[_idx] if _idx < len(_data) else None) is not None:
    _seen.append(_val)
    _idx += 1
assert _seen == [1, 2, 3, 4, 5], f"while walrus = {_seen!r}"

# Walrus in if condition
_text = "hello world"
if (_pos := _text.find("world")) >= 0:
    assert _pos == 6, f"find pos = {_pos!r}"
else:
    raise AssertionError("find should succeed")

# Walrus in if with side effect
_log = []
def _compute(x: int) -> int:
    _log.append(x)
    return x * 2

if (_result := _compute(5)) > 5:
    assert _result == 10, f"result = {_result!r}"
assert _log == [5], f"log = {_log!r}"

# Walrus in list comprehension (filter+transform)
_raw = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
_doubled_evens = [_d for _n in _raw if (_d := _n * 2) > 6]
assert _doubled_evens == [8, 10, 12, 14, 16, 18, 20], f"doubled = {_doubled_evens!r}"

# Walrus returns the assigned value
_x = None
_y = (_x := 42)
assert _x == 42, f"x = {_x!r}"
assert _y == 42, f"y = {_y!r}"

# Walrus in nested condition
def _first_positive(lst: list) -> int:
    return next((_w for _w in lst if (_r := _w) > 0 and _r == _w), -1)

assert _first_positive([-1, -2, 3, 4]) == 3, f"first_pos = {_first_positive([-1,-2,3,4])!r}"

print("surface OK")
