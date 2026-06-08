# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "fields_returns_field_objects"
# subject = "dataclasses.fields"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.fields: fields() returns a tuple of Field objects in declaration order, each carrying name/type/default/default_factory attributes"""
import dataclasses


@dataclasses.dataclass
class Point:
    x: float
    y: float


p = Point(1.0, 2.0)
fs = dataclasses.fields(p)
assert isinstance(fs, tuple), f"fields() type = {type(fs)!r}"
assert len(fs) == 2, f"field count = {len(fs)!r}"
assert fs[0].name == "x", f"first field name = {fs[0].name!r}"
assert fs[1].name == "y", f"second field name = {fs[1].name!r}"

# Each Field object exposes name/type/default/default_factory.
f = fs[0]
assert hasattr(f, "name"), "Field has name"
assert hasattr(f, "type"), "Field has type"
assert hasattr(f, "default"), "Field has default"
assert hasattr(f, "default_factory"), "Field has default_factory"

print("fields_returns_field_objects OK")
