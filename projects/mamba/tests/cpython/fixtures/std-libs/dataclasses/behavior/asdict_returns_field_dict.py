# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "asdict_returns_field_dict"
# subject = "dataclasses.asdict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.asdict: asdict() converts a flat dataclass instance into a plain {field: value} dict"""
import dataclasses


@dataclasses.dataclass
class Point:
    x: float
    y: float


p = Point(1.0, 2.0)
d = dataclasses.asdict(p)
assert isinstance(d, dict), f"asdict type = {type(d)!r}"
assert d == {"x": 1.0, "y": 2.0}, f"asdict = {d!r}"

print("asdict_returns_field_dict OK")
