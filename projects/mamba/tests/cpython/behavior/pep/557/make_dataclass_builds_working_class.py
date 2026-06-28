# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "make_dataclass_builds_working_class"
# subject = "dataclasses.make_dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "make_dataclass.py"
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass builds a working dataclass at runtime (init/eq/repr/fields all synthesized)"""
from dataclasses import make_dataclass, fields, is_dataclass

C = make_dataclass("C", [("x", int), ("y", int)])
assert type(C).__name__ == "type"
assert C.__name__ == "C"
obj = C(1, 2)
assert is_dataclass(C)
assert (obj.x, obj.y) == (1, 2)
assert obj == C(1, 2)
assert repr(obj) == "C(x=1, y=2)"
assert [f.name for f in fields(C)] == ["x", "y"]
print("make_dataclass_builds_working_class OK")
