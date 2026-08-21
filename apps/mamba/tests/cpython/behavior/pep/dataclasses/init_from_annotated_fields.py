# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "dataclasses"
# dimension = "behavior"
# case = "init_from_annotated_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = "mamba does not synthesize __init__ from annotated fields (repo memory project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = "behavior.py"
# status = "filled"
# ///
"""dataclasses.dataclass: @dataclass generates __init__ from annotated fields, applying declared defaults; positional and keyword construction both work"""
from dataclasses import dataclass


@dataclass
class Vec:
    x: int
    y: int
    z: int = 0


# Positional, with the trailing default applied.
v = Vec(1, 2)
assert v.x == 1 and v.y == 2 and v.z == 0, f"vec = {v.x},{v.y},{v.z}"

# Positional, overriding the default.
v2 = Vec(1, 2, 3)
assert v2.z == 3, f"z = {v2.z!r}"

# Keyword construction.
v3 = Vec(y=20, x=10, z=30)
assert (v3.x, v3.y, v3.z) == (10, 20, 30), f"vec3 = {v3.x},{v3.y},{v3.z}"

print("init_from_annotated_fields OK")
