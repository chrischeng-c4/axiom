# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "repr_false_field_hidden"
# subject = "dataclasses.field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: field(repr=False) suppresses that field from the synthesized __repr__ while repr=True fields still appear"""
import dataclasses


@dataclasses.dataclass
class Hidden:
    visible: str
    secret: str = dataclasses.field(repr=False)


h = Hidden("show", "hide")
r = repr(h)
assert "visible='show'" in r, f"visible in repr = {r!r}"
assert "secret" not in r, f"secret hidden from repr = {r!r}"

print("repr_false_field_hidden OK")
