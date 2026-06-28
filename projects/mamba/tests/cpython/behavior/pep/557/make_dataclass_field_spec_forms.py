# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "557"
# dimension = "behavior"
# case = "make_dataclass_field_spec_forms"
# subject = "dataclasses.make_dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "make_dataclass.py"
# status = "filled"
# ///
"""dataclasses.make_dataclass: make_dataclass field specs may be a bare name, (name, type), or (name, type, field(...))"""
from dataclasses import make_dataclass, field

D = make_dataclass(
    "D",
    [
        "a",  # bare name
        ("b", int),  # name + type
        ("c", int, field(default=9)),  # name + type + field
    ],
)
d = D(1, 2)
assert (d.a, d.b, d.c) == (1, 2, 9)
print("make_dataclass_field_spec_forms OK")
