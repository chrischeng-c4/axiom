# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "namedtuple_class_form"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: a class-form NamedTuple is a real tuple subclass with named fields, defaults, and helpers: Point(NamedTuple) with x:int, y:int=0 gives Point(1)==(1,0), isinstance tuple, _fields==('x','y'), _field_defaults=={'y':0}, __annotations__=={'x':int,'y':int}, _replace(y=5)==(1,5), _asdict()=={'x':1,'y':0}"""
from typing import NamedTuple


# Class-form NamedTuple: a real tuple subclass with named fields and defaults.
class Point(NamedTuple):
    x: int
    y: int = 0


p = Point(1)
assert isinstance(p, tuple)
assert (p.x, p.y) == (1, 0)
assert p == (1, 0)
assert Point._fields == ("x", "y")
assert Point._field_defaults == {"y": 0}
assert Point.__annotations__ == {"x": int, "y": int}
assert p._replace(y=5) == (1, 5)
assert p._asdict() == {"x": 1, "y": 0}

print("namedtuple_class_form OK")
