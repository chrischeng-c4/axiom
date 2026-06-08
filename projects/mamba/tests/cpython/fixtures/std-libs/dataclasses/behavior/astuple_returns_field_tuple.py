# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "astuple_returns_field_tuple"
# subject = "dataclasses.astuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.astuple: astuple() converts a flat dataclass instance into a field-ordered tuple"""
import dataclasses


@dataclasses.dataclass
class Point:
    x: float
    y: float


p = Point(1.0, 2.0)
t = dataclasses.astuple(p)
assert isinstance(t, tuple), f"astuple type = {type(t)!r}"
assert t == (1.0, 2.0), f"astuple = {t!r}"

print("astuple_returns_field_tuple OK")
