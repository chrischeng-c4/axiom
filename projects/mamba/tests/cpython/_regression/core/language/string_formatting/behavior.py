"""Behavior contract for language string formatting (f-strings, %, .format).

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: f-string evaluates expression at runtime
_a = 3
_b = 4
assert f"{_a}+{_b}={_a + _b}" == "3+4=7", f"expr = {f'{_a}+{_b}={_a + _b}'!r}"

# Rule 2: Format spec after colon
assert f"{42:>10}" == "        42", f"right-align = {f'{42:>10}'!r}"
assert f"{42:<10}" == "42        ", f"left-align = {f'{42:<10}'!r}"
assert f"{42:^10}" == "    42    ", f"center = {f'{42:^10}'!r}"
_fill_result = f"{'hi':*^6}"
assert _fill_result == "**hi**", f"fill = {_fill_result!r}"

# Rule 3: Conversion specifiers
_obj = [1, 2, 3]
_obj_r = repr(_obj)
assert f"{_obj!r}" == _obj_r, f"!r = {f'{_obj!r}'!r}"
assert f"{_obj!s}" == str(_obj), f"!s = {f'{_obj!s}'!r}"

# Rule 4: f-string with dict access
_d = {"key": "value"}
_dict_access = f"{_d['key']}"
assert _dict_access == "value", f"dict access = {_dict_access!r}"

# Rule 5: Nested f-string — dynamic format spec
_width = 8
_dyn_width = f"{'hi':{_width}}"
assert _dyn_width == "hi      ", f"dynamic width = {_dyn_width!r}"

# Rule 6: .format() positional and keyword
assert "{0} < {1}".format(1, 2) == "1 < 2", ".format positional"
assert "{x} + {y}".format(x=3, y=4) == "3 + 4", ".format keyword"
assert "{:>5}".format(42) == "   42", ".format with spec"

# Rule 7: % formatting — string and int
assert "name: %s" % "Bob" == "name: Bob", "% str"
assert "val: %05d" % 42 == "val: 00042", "% int pad"
assert "%.4f" % 2.71828 == "2.7183", "% float"
assert "%x %X" % (255, 255) == "ff FF", "% hex"

# Rule 8: str.format_map
class _Default(dict):
    def __missing__(self, key: str) -> str:
        return f"<{key}>"

_res = "{name} {age}".format_map(_Default(name="Alice"))
assert _res == "Alice <age>", f"format_map = {_res!r}"

# Rule 9: f-string with __format__
class _Angle:
    def __init__(self, deg: float):
        self.deg = deg
    def __format__(self, spec: str) -> str:
        if spec == "rad":
            import math
            return f"{self.deg * math.pi / 180:.4f}"
        return f"{self.deg:.1f}°"

_ang = _Angle(90.0)
assert f"{_ang}" == "90.0°", f"__format__ default = {f'{_ang}'!r}"
_rad_str = f"{_ang:rad}"
assert _rad_str == "1.5708", f"__format__ rad = {_rad_str!r}"

# Rule 10: Empty f-string
assert f"" == "", "empty f-string"

print("behavior OK")
