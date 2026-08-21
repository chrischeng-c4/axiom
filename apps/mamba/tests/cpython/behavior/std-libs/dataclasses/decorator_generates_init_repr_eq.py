# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "decorator_generates_init_repr_eq"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass synthesizes __init__ (positional fields), __repr__ (ClassName(x=1.0, y=2.0)), and __eq__ (field-wise) for a two-field class"""
import dataclasses


@dataclasses.dataclass
class Point:
    x: float
    y: float


p = Point(1.0, 2.0)
# __init__ binds positional fields by declaration order.
assert p.x == 1.0, f"x = {p.x!r}"
assert p.y == 2.0, f"y = {p.y!r}"

# __repr__ is auto-generated as ClassName(field=value, ...).
r = repr(p)
assert "Point" in r, f"repr has class = {r!r}"
assert "x=1.0" in r, f"repr has x = {r!r}"
assert "y=2.0" in r, f"repr has y = {r!r}"

# __eq__ compares field-wise.
assert p == Point(1.0, 2.0), "equal points"
assert p != Point(1.0, 3.0), "unequal points"

print("decorator_generates_init_repr_eq OK")
